mod database;

use database::*;
use std::{io, thread};
use std::io::Result;
use std::time::Duration;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use tui::{
    style::{
        Style,
        Color,
    },
    layout::{Constraint, Direction, Layout}, backend::Backend, Frame, widgets::{Block, Borders}, Terminal};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use tui::backend::CrosstermBackend;
use tui::widgets::{List, ListItem};

enum Screen {
    Todos,
    Stats,
}

struct App {
    todos: Vec<Todo>,
    current_index: usize,
    current_screen: Screen,
}

impl App {
    fn new() -> Self {
        Self {
            todos: vec![],
            current_index: 0,
            current_screen: Screen::Todos,
        }
    }

    fn add_todos(&mut self, todos: Vec<Todo>) {
        for todo in todos {
            self.todos.push(todo);
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
                            Constraint::Percentage(20),
                            Constraint::Percentage(80)
                        ].as_ref()
                    )
                    .split(f.size());
                let block = Block::default()
                    .title("TODOs")
                    .borders(Borders::ALL);
                let list = List::new(self.get_todos_list()).block(block);
                f.render_widget(list, chunks[0]);
                let block = Block::default()
                    .title("About")
                    .borders(Borders::ALL);
                f.render_widget(block, chunks[1]);
            }
            Screen::Stats => {}
        }
    }

    fn get_todos_list(&self) -> Vec<ListItem> {
        self.todos.iter().enumerate().map(|(index, todo)| {
            let item = ListItem::new(todo.get_text());
            if index == self.current_index {
                item.style(Style::default().fg(Color::Black).bg(Color::White))
            }else {
                item
            }
        }).collect()
    }
}

fn main() -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    app.add_todos(vec![
        Todo {
            id: 0,
            position: 0,
            text: String::from("Rowario"),
            completed: false,
        },
        Todo {
            id: 1,
            position: 1,
            text: String::from("Rowario 1"),
            completed: false,
        },
    ]);

    terminal.draw(|f| app.draw_ui(f))?;

    thread::sleep(Duration::from_millis(5000));
    // TODO: controls and app loop

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
