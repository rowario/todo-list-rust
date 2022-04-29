mod database;

use database::*;
use std::{io};
use std::io::Result;
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
use tui::widgets::{List, ListItem};

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
}

impl App {
    fn new(db: Connection) -> Self {
        Self {
            db,
            todos: vec![],
            current_index: 0,
            current_screen: Screen::Todos,
        }
    }

    fn load_todos(&mut self) {
        let todos = get_todos(&self.db).expect("Cannot get todos");
        self.todos = todos;
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

    fn toggle_todo(&mut self) {
        if let Some(todo) = self.todos.get_mut(self.current_index) {
            if toggle_todo(&self.db, todo.id).is_ok() {
                todo.toggle();
            }
        }
    }

    fn draw_ui<B: Backend>(&self, f: &mut Frame<B>) {
        match self.current_screen {
            Screen::Todos => {
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
            Screen::Stats => {}
            Screen::New => {}
        }
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
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('j') => app.next(),
                KeyCode::Char('k') => app.previous(),
                KeyCode::Char('x') => app.toggle_todo(),
                _ => {}
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
