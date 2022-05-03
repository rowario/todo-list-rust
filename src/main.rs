mod database;
mod ui;

use database::*;
use ui::*;

use rusqlite::Connection;
use std::{
    io::{
        self,
        Result,
    }
};
use chrono::Utc;
use tui::{Frame, Terminal, backend::Backend, backend::CrosstermBackend};
use crossterm::{
    execute,
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers, Event::{Key}},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

enum Screen {
    NewDay,
    NewTodo,
    Todos,
    Stats,
}

pub struct App {
    index: usize,
    input: String,
    screen: Screen,
    db: Connection,
    day: Day,
}

impl App {
    fn new(db: Connection) -> Self {
        let days = get_days(&db).unwrap();
        let day = if !days.is_empty() {
            let result = days.last().unwrap();
            get_day(&db, result.id).expect("Error: Cannot load todos.")
        } else {
            new_day(&db, Utc::today().format("%Y-%m-%d").to_string().as_str())
                .expect("Error: Cannot create new day.")
        };
        Self {
            screen: if day.todos.is_empty() { Screen::NewTodo } else { Screen::Todos },
            input: String::new(),
            index: 0,
            day,
            db,
        }
    }

    fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
    }

    fn swap(&mut self, index: usize) {
        self.day.todos.swap(self.index, index);
        update_todos_positions(&self.db, &self.day.todos).expect("Error: Cannot update positions.")
    }

    fn next(&mut self, modifiers: KeyModifiers) {
        if !self.day.todos.is_empty() && self.index < self.day.todos.len() - 1 {
            if modifiers == KeyModifiers::SHIFT {
                self.swap(self.index + 1);
            }
            self.index += 1;
        }
    }

    fn previous(&mut self, modifiers: KeyModifiers) {
        if !self.day.todos.is_empty() && self.index > 0 {
            if modifiers == KeyModifiers::SHIFT {
                self.swap(self.index - 1);
            }
            self.index -= 1;
        }
    }

    fn toggle(&mut self) {
        if let Some(todo) = self.day.todos.get_mut(self.index) {
            if toggle_todo(&self.db, todo.id).is_ok() {
                todo.toggle();
            }
        }
    }

    fn create(&mut self) {
        if let Ok(todo) = new_todo(&self.db, self.input.as_str(), self.day.id) {
            self.input.clear();
            self.day.todos.push(todo);
        }
    }

    fn delete(&mut self) {
        if let Some(todo) = self.day.todos.get(self.index) {
            if delete_todo(&self.db, todo.id).is_ok() {
                self.day.todos.remove(self.index);
            }
        }
    }

    fn ui<B: Backend>(&self, f: &mut Frame<B>) {
        match self.screen {
            Screen::NewDay => {}
            Screen::NewTodo => {
                todos_screen(self, f, false);
                new_screen(self, f);
            }
            Screen::Todos => todos_screen(self, f, true),
            Screen::Stats => stats_screen(self, f),
        }
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
                Screen::NewDay => {}
                Screen::NewTodo => {
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
                            'n' => app.set_screen(Screen::NewTodo),
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
