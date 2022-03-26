use std::io::{stdin, stdout};

use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand,
    Result,
    event::{KeyCode, KeyEvent, Event::Key, read},
    terminal::{Clear, ClearType},
};

struct TodoItem {
    text: String,
    completed: bool,
}

impl TodoItem {
    fn new(text: String) -> TodoItem {
        TodoItem {
            text,
            completed: false,
        }
    }

    fn toggle(&mut self) {
        self.completed = !self.completed;
    }
}

// TODO: Implement a local storage

fn main() -> Result<()> {
    stdout().execute(Clear(ClearType::All))?;
    let mut todo_items = vec![
        TodoItem::new("Buy milk".to_string()),
        TodoItem::new("Wash the dishes".to_string()),
        TodoItem::new("Learn to code".to_string()),
    ];

    let mut current = 0;

    loop {
        stdout()
            .execute(cursor::Hide)?
            .execute(cursor::MoveTo(0, 0))?
            .execute(Clear(ClearType::All))?;

        for (i, item) in todo_items.iter().enumerate() {
            let color = if item.completed {
                Color::Green
            } else {
                Color::White
            };

            let background_color = if i == current {
                Color::DarkGrey
            } else {
                Color::Reset
            };

            let text = if item.completed {
                format!("{} {}", "[x]", item.text)
            } else {
                format!("{} {}", "[ ]", item.text)
            };

            stdout()
                .execute(SetForegroundColor(color))?
                .execute(SetBackgroundColor(background_color))?
                .execute(Print(text))?
                .execute(ResetColor)?
                .execute(Print("\n"))?;
        }

        stdout()
            .execute(SetForegroundColor(Color::White))?
            .execute(SetBackgroundColor(Color::Black))?
            .execute(Print("[n]: New | [d]: Delete | [x]: Toggle | [q]: Quit".trim()))?
            .execute(ResetColor)?;

        let event = read()?;

        match event {
            Key(KeyEvent { code: KeyCode::Char('n'), modifiers: _ }) => {
                let mut text = String::new();
                stdout()
                    .execute(Clear(ClearType::All))?
                    .execute(cursor::Hide)?
                    .execute(cursor::MoveTo(0, 0))?
                    .execute(SetForegroundColor(Color::White))?
                    .execute(SetBackgroundColor(Color::Black))?
                    .execute(Print("Enter new item: "))?
                    .execute(ResetColor)?
                    .execute(cursor::Show)?;

                stdin().read_line(&mut text)?;
                stdout().execute(cursor::Hide)?;

                todo_items.push(TodoItem::new(text.trim().to_string()));
            }
            Key(KeyEvent { code: KeyCode::Char('q'), modifiers: _ }) => {
                stdout()
                    .execute(Clear(ClearType::All))?
                    .execute(cursor::MoveTo(0, 0))?
                    .execute(cursor::Show)?;
                break;
            }
            Key(KeyEvent { code: KeyCode::Up, modifiers: _ }) => {
                if !todo_items.is_empty() && current > 0 {
                    current -= 1;
                }
            }
            Key(KeyEvent { code: KeyCode::Down, modifiers: _ }) => {
                if !todo_items.is_empty() && current < todo_items.len() - 1 {
                    current += 1;
                }
            }
            Key(KeyEvent { code: KeyCode::Char('x'), modifiers: _ }) => {
                if let Some(item) = todo_items.get_mut(current) {
                    item.toggle();
                }
            }
            Key(KeyEvent { code: KeyCode::Char('d'), modifiers: _ }) => {
                if todo_items.get_mut(current).is_some() {
                    todo_items.remove(current);
                    if current > 0 {
                        current -= 1;
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}
