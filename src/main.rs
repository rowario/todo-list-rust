mod database;

use database::*;
use std::{io::{
    self,
    Result
}};
use crossterm::{event, execute};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use tui::{
    style::{
        Style,
        Color,
    },
    layout::{Constraint, Direction, Layout}, backend::Backend, Frame, widgets::{Block, Borders}, Terminal};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event::{Key}},
    terminal::{
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use crossterm::event::KeyCode;
use rusqlite::Connection;
use tui::backend::CrosstermBackend;
use tui::layout::Rect;
use tui::widgets::{Clear, List, ListItem, Paragraph};
use unicode_width::UnicodeWidthStr;

enum Screen {
    New,
    Todos,
    Stats,
}

struct App {
    db: Connection,
    todos: Vec<Todo>,
    current_index: usize,
    current_screen: Screen,
    input_value: String,
}

impl App {
    fn new(db: Connection) -> Self {
        Self {
            db,
            todos: vec![],
            current_index: 0,
            current_screen: Screen::Todos,
            input_value: String::new(),
        }
    }

    fn load_todos(&mut self) {
        let todos = get_todos(&self.db).expect("Cannot get todos");
        self.todos = todos;
    }

    fn set_screen(&mut self, screen: Screen) {
        self.current_screen = screen;
    }

    fn next(&mut self) {
        if !self.todos.is_empty() && self.current_index < self.todos.len() - 1 {
            self.current_index += 1;
        }
    }

    fn previous(&mut self) {
        if !self.todos.is_empty() && self.current_index > 0 {
            self.current_index -= 1;
        }
    }

    fn toggle(&mut self) {
        if let Some(todo) = self.todos.get_mut(self.current_index) {
            if toggle_todo(&self.db, todo.id).is_ok() {
                todo.toggle();
            }
        }
    }

    fn create(&mut self) {
        if let Ok(todo) = new_todo(&self.db, self.input_value.as_str()) {
            self.input_value.clear();
            self.todos.push(todo);
        }
    }

    fn delete(&mut self) {
        if let Some(todo) = self.todos.get(self.current_index) {
            if delete_todo(&self.db, todo.id).is_ok() {
                self.todos.remove(self.current_index);
            }
        }
    }

    fn draw_ui<B: Backend>(&self, f: &mut Frame<B>) {
        match self.current_screen {
            Screen::Todos => self.draw_todos(f),
            Screen::Stats => {}
            Screen::New => {
                self.draw_todos(f);
                self.draw_new(f);
            }
        }
    }

    fn draw_todos<B: Backend>(&self, f: &mut Frame<B>) {
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

    fn draw_new<B: Backend>(&self, f: &mut Frame<B>) {
        let block = Paragraph::new(self.input_value.as_ref())
            .block(Block::default().title("New TODO").borders(Borders::ALL));
        let area = input_rect(60, f.size());
        f.render_widget(Clear, area);
        f.render_widget(block, area);
        f.set_cursor(area.x + self.input_value.width() as u16 + 1, area.y + 1);
    }

    fn get_todos_list(&self) -> Vec<ListItem> {
        self.todos.iter().enumerate().map(|(index, todo)| {
            ListItem::new(todo.get_text())
                .style(Style::default().fg(if index == self.current_index { Color::Yellow } else { Color::White }))
        }).collect()
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

    app.load_todos();

    loop {
        terminal.draw(|f| app.draw_ui(f))?;

        if let Key(key) = event::read()? {
            match app.current_screen {
                Screen::New => {
                    match key.code {
                        KeyCode::Char(c) => app.input_value.push(c),
                        KeyCode::Esc => {
                            app.input_value.clear();
                            app.set_screen(Screen::Todos);
                        }
                        KeyCode::Backspace => {
                            app.input_value.pop();
                        },
                        KeyCode::Enter => {
                            app.set_screen(Screen::Todos);
                            app.create();
                        },
                        _ => {}
                    }
                }
                Screen::Todos => {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('k') => app.previous(),
                        KeyCode::Char('j') => app.next(),
                        KeyCode::Char('x') => app.toggle(),
                        KeyCode::Char('d') => app.delete(),
                        KeyCode::Char('n') => app.set_screen(Screen::New),
                        _ => {}
                    }
                }
                Screen::Stats => {}
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

fn input_rect(percent_x: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - Constraint::Length(3).apply(3)) / 2),
                Constraint::Length(3),
                Constraint::Percentage((100 - Constraint::Length(3).apply(3)) / 2),
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
