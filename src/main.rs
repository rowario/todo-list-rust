mod database;

use database::*;
use std::io::{stdin, stdout};
use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand,
    Result,
    event::{KeyCode::{self, Char}, Event::Key, read},
    terminal::{Clear, ClearType},
};
use crossterm::event::KeyModifiers;

fn main() -> Result<()> {
    let mut path = std::env::current_exe()?;
    path.pop();
    path.push("database.sqlite");
    let db = init(path.to_str().unwrap()).expect("Error: failed to initialize database");

    let mut todo_items = get_todos(&db).expect("Error: failed to get todos");

    stdout().execute(Clear(ClearType::All))?;

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

            stdout()
                .execute(SetForegroundColor(color))?
                .execute(SetBackgroundColor(background_color))?
                .execute(Print(item.get_text().as_str()))?
                .execute(ResetColor)?
                .execute(Print("\n"))?;
        }

        stdout()
            .execute(Print("\n"))?
            .execute(SetForegroundColor(Color::White))?
            .execute(SetBackgroundColor(Color::Black))?
            .execute(Print("[n]: New | [d]: Delete | [x]: Toggle | [q]: Quit".trim()))?
            .execute(ResetColor)?;

        if let Key(key_event) = read()? {
            match key_event.code {
                KeyCode::Up => {
                    if !todo_items.is_empty() && current > 0 {
                        if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                            todo_items.swap(current, current - 1);
                            update_todos_positions(&db, &todo_items).expect("Error: failed to update todos positions");
                        }
                        current -= 1;
                    }
                }
                KeyCode::Down => {
                    if !todo_items.is_empty() && current < todo_items.len() - 1 {
                        if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                            todo_items.swap(current, current + 1);
                            update_todos_positions(&db, &todo_items).expect("Error: failed to update todos positions");
                        }
                        current += 1;
                    }
                }
                Char(char) => {
                    match char {
                        'n' | 'т' => {
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

                            let new_todo = new_todo(&db, text.trim()).expect("Error: failed to create new todo");

                            todo_items.push(new_todo);
                        }
                        'q' | 'й' => {
                            stdout()
                                .execute(Clear(ClearType::All))?
                                .execute(cursor::MoveTo(0, 0))?
                                .execute(cursor::Show)?;
                            break;
                        }

                        'x' | 'ч' => {
                            if let Some(item) = todo_items.get_mut(current) {
                                toggle_todo(&db, item.id).expect("Error: Could not toggle todo");
                                item.toggle();
                            }
                        }
                        'd' | 'в' => {
                            if let Some(todo) = todo_items.get(current) {
                                delete_todo(&db, todo.id).expect("Error: Could not delete todo");
                                todo_items.remove(current);
                                if current > 0 {
                                    current -= 1;
                                }
                                update_todos_positions(&db, &todo_items).expect("Error: failed to update todos positions");
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
