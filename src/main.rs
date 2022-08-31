mod database;
mod ui;

use database::{init_connection, DailyTodo, Day, DayShort, Todo};
use ui::{daily_todos_screen, new_daily_todo_screen, new_todo_screen, stats_screen, todos_screen};

use chrono::Local;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event::Key, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rusqlite::Connection;
use std::io;
use tui::{backend::Backend, backend::CrosstermBackend, Frame, Terminal};

enum Screen {
    NewTodo,
    Todos,
    Notes,
    EditNotes,
    NewDailyTodo,
    DailyTodos,
    Stats,
}

struct DailyTodoList {
    index: usize,
    input: String,
    list: Vec<DailyTodo>,
}

impl DailyTodoList {
    pub fn new(db: &Connection) -> io::Result<Self> {
        let list = DailyTodo::get_all(db).unwrap();
        Ok(Self {
            index: 0,
            input: String::new(),
            list,
        })
    }

    fn swap(&mut self, db: &Connection, index: usize) {
        self.list.swap(self.index, index);
        DailyTodo::update_positions(db, &self.list).expect("Error: Cannot update positions.")
    }

    fn next(&mut self, db: &Connection, modifiers: KeyModifiers) {
        if !self.list.is_empty() && self.index < self.list.len() - 1 {
            if modifiers == KeyModifiers::SHIFT {
                self.swap(db, self.index + 1);
            }
            self.index += 1;
        }
    }

    fn previous(&mut self, db: &Connection, modifiers: KeyModifiers) {
        if !self.list.is_empty() && self.index > 0 {
            if modifiers == KeyModifiers::SHIFT {
                self.swap(db, self.index - 1);
            }
            self.index -= 1;
        }
    }

    fn create(&mut self, db: &Connection) {
        if !self.input.trim().is_empty() {
            if let Ok(todo) = DailyTodo::new(db, self.input.trim()) {
                self.list.push(todo);
            }
        }
        self.input.clear();
    }

    fn delete(&mut self, db: &Connection) {
        if let Some(todo) = self.list.get(self.index) {
            if todo.delete(db).is_ok() {
                self.list.remove(self.index);
                if self.index >= self.list.len() && self.index != 0 {
                    self.index -= 1;
                }
            }
        }
    }
}

struct StatsList {
    index: usize,
    list: Vec<DayShort>,
}

impl StatsList {
    pub fn new(db: &Connection) -> io::Result<Self> {
        let list = DayShort::get_all(db).unwrap();
        Ok(Self {
            index: list.len() - 1,
            list,
        })
    }

    pub fn update(&mut self, db: &Connection) -> io::Result<()> {
        self.list = DayShort::get_all(db).unwrap();
        Ok(())
    }

    pub fn get_current(&self, db: &Connection) -> io::Result<Day> {
        if let Some(day) = self.list.get(self.index) {
            let day = Day::get(db, day.id).unwrap();
            Ok(day)
        } else {
            Ok(Day {
                id: 0,
                count_todos: 0,
                done_todos: 0,
                notes: String::new(),
                date: String::from("0000-00-00"),
                todos: vec![],
            })
        }
    }

    fn next(&mut self) {
        if !self.list.is_empty() && self.index < self.list.len() - 1 {
            self.index += 1;
        }
    }

    fn previous(&mut self) {
        if !self.list.is_empty() && self.index > 0 {
            self.index -= 1;
        }
    }
}

pub struct App {
    index: usize,
    input: String,
    screen: Screen,
    db: Connection,
    day: Day,
    daily_todos: DailyTodoList,
    stats_list: StatsList,
}

impl App {
    fn new(db: Connection) -> Self {
        let days = DayShort::get_all(&db).unwrap();
        let day = if !days.is_empty() {
            let result = days.last().unwrap();
            Day::get(&db, result.id).unwrap()
        } else {
            Day::new(&db, Local::today().format("%Y-%m-%d").to_string().as_str()).unwrap()
        };
        let daily_todos = DailyTodoList::new(&db).unwrap();
        let stats_list = StatsList::new(&db).unwrap();
        Self {
            screen: if day.todos.is_empty() {
                Screen::NewTodo
            } else {
                Screen::Todos
            },
            input: String::new(),
            index: 0,
            day,
            db,
            daily_todos,
            stats_list,
        }
    }

    fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
    }

    fn new_day(&mut self) {
        let new_date = Local::today().format("%Y-%m-%d").to_string();
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
                self.day
                    .add_todo(&self.db, todo)
                    .expect("Error: Cannot add todo, to day.");
                self.stats_list.update(&self.db).unwrap();
            }
        }
        self.input.clear();
    }

    fn delete(&mut self) {
        if let Some(todo) = self.day.todos.get(self.index) {
            if todo.delete(&self.db).is_ok() {
                self.day
                    .remove_todo(&self.db, self.index)
                    .expect("Error: Cannot remove todo.");
                self.stats_list.update(&self.db).unwrap();
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
                new_todo_screen(self, f);
            }
            Screen::Todos => todos_screen(self, f, true),
            Screen::Notes => todos_screen(self, f, false),
            Screen::EditNotes => todos_screen(self, f, false),
            Screen::NewDailyTodo => {
                todos_screen(self, f, false);
                daily_todos_screen(self, f, false);
                new_daily_todo_screen(self, f);
            }
            Screen::DailyTodos => {
                todos_screen(self, f, false);
                daily_todos_screen(self, f, true);
            }
            Screen::Stats => stats_screen(self, f),
        }
    }
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut path = std::env::current_exe()?;
    path.pop();
    path.push("database.sqlite");
    let db = init_connection(path.to_str().unwrap()).expect("Error: failed to initialize database");

    let mut app = App::new(db);

    enable_raw_mode()?;
    loop {
        terminal.draw(|f| app.ui(f))?;

        if let Key(key) = event::read()? {
            match app.screen {
                Screen::NewTodo => match key.code {
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
                },
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
                                    app.day.update_counts(&app.db).unwrap();
                                    app.stats_list.update(&app.db).unwrap();
                                }
                            }
                            'd' => app.delete(),
                            'n' => {
                                if key.modifiers == KeyModifiers::SHIFT {
                                    app.new_day();
                                } else {
                                    app.set_screen(Screen::NewTodo);
                                }
                            }
                            't' => app.set_screen(Screen::DailyTodos),
                            's' => app.set_screen(Screen::Stats),
                            _ => {}
                        }
                    }
                }
                Screen::NewDailyTodo => match key.code {
                    KeyCode::Esc => {
                        app.daily_todos.input.clear();
                        app.set_screen(Screen::DailyTodos);
                    }
                    KeyCode::Backspace => {
                        app.daily_todos.input.pop();
                    }
                    KeyCode::Enter => {
                        app.set_screen(Screen::DailyTodos);
                        app.daily_todos.create(&app.db);
                    }
                    KeyCode::Char(c) => app.daily_todos.input.push(c),
                    _ => {}
                },
                Screen::DailyTodos => match key.code {
                    KeyCode::Esc => {
                        app.set_screen(Screen::Todos);
                    }
                    KeyCode::Char(c) => match c.to_ascii_lowercase() {
                        'j' => app.daily_todos.next(&app.db, key.modifiers),
                        'k' => app.daily_todos.previous(&app.db, key.modifiers),
                        'n' => app.set_screen(Screen::NewDailyTodo),
                        'd' => app.daily_todos.delete(&app.db),
                        _ => {}
                    },
                    _ => {}
                },
                Screen::Notes => {
                    if let KeyCode::Char(char) = key.code {
                        match char.to_ascii_lowercase() {
                            'q' => break,
                            'e' => app.set_screen(Screen::EditNotes),
                            't' => app.set_screen(Screen::DailyTodos),
                            'h' => app.set_screen(Screen::Todos),
                            _ => {}
                        }
                    }
                }
                Screen::EditNotes => match key.code {
                    KeyCode::Esc => {
                        app.set_screen(Screen::Notes);
                        app.day
                            .set_notes(&app.db)
                            .expect("Error: Cannot save notes.");
                    }
                    KeyCode::Backspace => {
                        app.day.notes.pop();
                    }
                    KeyCode::Enter => app.day.notes.push('\n'),
                    KeyCode::Char(c) => app.day.notes.push(c),
                    _ => {}
                },
                Screen::Stats => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('h') => app.stats_list.previous(),
                    KeyCode::Char('l') => app.stats_list.next(),
                    KeyCode::Char('s') | KeyCode::Esc => {
                        app.input.clear();
                        app.set_screen(Screen::Todos);
                    }
                    _ => {}
                },
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
