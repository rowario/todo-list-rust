mod database;

use database::*;
use rusqlite::Connection;
use unicode_width::UnicodeWidthStr;
use crossterm::event::{KeyCode, KeyModifiers};
use std::{
    io::{
        self,
        Result,
    }
};
use tui::{Frame, Terminal, backend::Backend, style::{Style, Color}, backend::CrosstermBackend, layout::{Rect, Constraint, Direction, Layout}, widgets::{Block, Borders, Clear, List, ListItem, Paragraph}, symbols};
use crossterm::{
    execute,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event::{Key}},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use tui::widgets::{BarChart};

enum Screen {
    New,
    Todos,
    Stats,
}

struct App {
    day: usize,
    index: usize,
    input: String,
    screen: Screen,
    db: Connection,
    todos: Vec<Todo>,
}

impl App {
    fn new(db: Connection) -> Self {
        let todos = get_todos(&db).expect("Error: Cannot load todos.");
        Self {
            screen: if todos.is_empty() { Screen::New } else { Screen::Todos },
            input: String::new(),
            index: 0,
            day: 0,
            todos,
            db,
        }
    }

    fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
    }

    fn swap(&mut self, index: usize) {
        self.todos.swap(self.index, index);
        update_todos_positions(&self.db, &self.todos).expect("Error: Cannot update positions.")
    }

    fn next(&mut self, modifiers: KeyModifiers) {
        if !self.todos.is_empty() && self.index < self.todos.len() - 1 {
            if modifiers == KeyModifiers::SHIFT {
                self.swap(self.index + 1);
            }
            self.index += 1;
        }
    }

    fn previous(&mut self, modifiers: KeyModifiers) {
        if !self.todos.is_empty() && self.index > 0 {
            if modifiers == KeyModifiers::SHIFT {
                self.swap(self.index - 1);
            }
            self.index -= 1;
        }
    }

    fn toggle(&mut self) {
        if let Some(todo) = self.todos.get_mut(self.index) {
            if toggle_todo(&self.db, todo.id).is_ok() {
                todo.toggle();
            }
        }
    }

    fn create(&mut self) {
        if let Ok(todo) = new_todo(&self.db, self.input.as_str()) {
            self.input.clear();
            self.todos.push(todo);
        }
    }

    fn delete(&mut self) {
        if let Some(todo) = self.todos.get(self.index) {
            if delete_todo(&self.db, todo.id).is_ok() {
                self.todos.remove(self.index);
            }
        }
    }

    fn ui<B: Backend>(&self, f: &mut Frame<B>) {
        match self.screen {
            Screen::Todos => self.todos_screen(f),
            Screen::Stats => self.stats_screen(f),
            Screen::New => {
                self.todos_screen(f);
                self.new_screen(f);
            }
        }
    }

    fn todos_screen<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(70)
                ].as_ref()
            )
            .split(f.size());
        let block = Block::default()
            .title("TODOs")
            .borders(Borders::ALL);
        let list = List::new(self.get_todos_list()).block(block);
        f.render_widget(list, chunks[0]);
        let block = Block::default()
            .title("Notes")
            .borders(Borders::ALL);
        f.render_widget(block, chunks[1]);
    }

    fn stats_screen<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(70)
                ].as_ref()
            )
            .split(f.size());
        let block = Block::default()
            .title("Days")
            .borders(Borders::ALL);
        let data: [(&str, u64); 5] = [("*27.04.2022*", 100), ("-28.04.2022-", 90), ("29.04.2022", 80), ("30.04.2022", 20), ("01.05.2022", 40)];
        let chart = BarChart::default()
            .bar_width(12)
            .bar_style(Style::default().fg(Color::Yellow))
            .value_style(Style::default().fg(Color::White).bg(Color::Yellow))
            .label_style(Style::default().fg(Color::White))
            .data(&data)
            .block(block);
        f.render_widget(chart, chunks[0]);
        let block = Block::default()
            .title("Notes")
            .borders(Borders::ALL);
        f.render_widget(block, chunks[1]);
    }

    fn new_screen<B: Backend>(&self, f: &mut Frame<B>) {
        let block = Paragraph::new(self.input.as_ref())
            .block(Block::default().title("New TODO").borders(Borders::ALL));
        let area = App::centered_input(60, f.size());
        f.render_widget(Clear, area);
        f.render_widget(block, area);
        f.set_cursor(area.x + self.input.width() as u16 + 1, area.y + 1);
    }

    fn get_todos_list(&self) -> Vec<ListItem> {
        self.todos.iter().enumerate().map(|(index, todo)| {
            ListItem::new(todo.get_text())
                .style(Style::default().fg(if index == self.index { Color::Yellow } else { Color::White }))
        }).collect()
    }

    fn centered_input(percent_x: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(r.height / 2 - 1),
                    Constraint::Min(3),
                    Constraint::Length(r.height / 2 - 1),
                ]
                    .as_ref(),
            )
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_x) / 2),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage((100 - percent_x) / 2),
                ]
                    .as_ref(),
            )
            .split(popup_layout[1])[1]
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut path = std::env::current_exe()?;
    path.pop();
    path.push("database.sqlite");
    let db = init(path.to_str().unwrap()).expect("Error: failed to initialize database");

    let mut app = App::new(db);

    loop {
        terminal.draw(|f| app.ui(f))?;

        if let Key(key) = event::read()? {
            match app.screen {
                Screen::New => {
                    match key.code {
                        KeyCode::Esc => {
                            app.input.clear();
                            app.set_screen(Screen::Todos);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Enter => {
                            app.set_screen(Screen::Todos);
                            app.create();
                        }
                        KeyCode::Char(c) => app.input.push(c),
                        _ => {}
                    }
                }
                Screen::Todos => {
                    if let KeyCode::Char(char) = key.code {
                        match char.to_ascii_lowercase() {
                            'q' => break,
                            'k' => app.previous(key.modifiers),
                            'j' => app.next(key.modifiers),
                            'x' => app.toggle(),
                            'd' => app.delete(),
                            'n' => app.set_screen(Screen::New),
                            's' => app.set_screen(Screen::Stats),
                            _ => {}
                        }
                    }
                }
                Screen::Stats => {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Esc => {
                            app.input.clear();
                            app.set_screen(Screen::Todos);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
