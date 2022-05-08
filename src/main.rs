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
    NewTodo,
    Todos,
    Notes,
    EditNotes,
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
        let days = DayShort::get_all(&db).unwrap();
        let day = if !days.is_empty() {
            let result = days.last().unwrap();
            Day::get(&db, result.id).unwrap()
        } else {
            Day::new(&db, Utc::today().format("%Y-%m-%d").to_string().as_str())
                .unwrap()
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

    fn new_day(&mut self) {
        let new_date = Utc::today().format("%Y-%m-%d").to_string();
        if new_date != self.day.date {
            self.day = Day::new(&self.db, new_date.as_str()).unwrap();
            self.index = 0;
        }
    }

    fn swap(&mut self, index: usize) {
        self.day.todos.swap(self.index, index);
        Todo::update_positions(&self.db, &self.day.todos).expect("Error: Cannot update positions.")
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

    fn create(&mut self) {
        if !self.input.trim().is_empty() {
            if let Ok(todo) = Todo::new(&self.db, self.input.trim(), self.day.id) {
                self.day.add_todo(&self.db, todo).expect("Error: Cannot add todo, to day.");
            }
        }
        self.input.clear();
    }

    fn delete(&mut self) {
        if let Some(todo) = self.day.todos.get(self.index) {
            if todo.delete(&self.db).is_ok() {
                self.day.remove_todo(&self.db, self.index).expect("Error: Cannot remove todo.");
                if self.index >= self.day.todos.len() && self.index != 0 {
                    self.index -= 1;
                }
            }
        }
    }

    fn ui<B: Backend>(&self, f: &mut Frame<B>) {
        match self.screen {
            Screen::NewTodo => {
                todos_screen(self, f, false);
                new_screen(self, f);
            }
            Screen::Todos => todos_screen(self, f, true),
            Screen::Notes => todos_screen(self, f, false),
            Screen::EditNotes => todos_screen(self, f, false),
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
    let db = init_connection(path.to_str().unwrap()).expect("Error: failed to initialize database");

    let mut app = App::new(db);

    loop {
        terminal.draw(|f| app.ui(f))?;

        if let Key(key) = event::read()? {
            match app.screen {
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
                            'l' => app.set_screen(Screen::Notes),
                            'x' => {
                                if let Some(todo) = app.day.todos.get_mut(app.index) {
                                    todo.toggle(&app.db).expect("Error: Cannot toggle todo.");
                                }
                            }
                            'd' => app.delete(),
                            'n' => {
                                if key.modifiers == KeyModifiers::SHIFT {
                                    app.new_day();
                                }
                                app.set_screen(Screen::NewTodo);
                            }
                            's' => app.set_screen(Screen::Stats),
                            _ => {}
                        }
                    }
                }
                Screen::Notes => {
                    if let KeyCode::Char(char) = key.code {
                        match char.to_ascii_lowercase() {
                            'q' => break,
                            'e' => app.set_screen(Screen::EditNotes),
                            'h' => app.set_screen(Screen::Todos),
                            _ => {}
                        }
                    }
                }
                Screen::EditNotes => {
                    match key.code {
                        KeyCode::Esc => {
                            app.set_screen(Screen::Notes);
                            app.day.set_notes(&app.db).expect("Error: Cannot save notes.");
                        }
                        KeyCode::Backspace => {
                            app.day.notes.pop();
                        }
                        KeyCode::Enter => app.day.notes.push('\n'),
                        KeyCode::Char(c) => app.day.notes.push(c),
                        _ => {}
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
