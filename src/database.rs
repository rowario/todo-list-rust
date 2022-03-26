use rusqlite::{Connection, Result};

pub struct Todo {
    pub id: i64,
    pub text: String,
    pub completed: bool,
}

impl Todo {
    pub fn get_text(&self) -> String {
        if self.completed {
            format!("{} {}", "[x]", self.text)
        } else {
            format!("{} {}", "[ ]", self.text)
        }
    }

    pub fn toggle(&mut self) {
        self.completed = !self.completed;
    }
}

pub fn init(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY,
            text TEXT NOT NULL,
            completed INTEGER NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

pub fn get_todos(conn: &Connection) -> Result<Vec<Todo>> {
    let mut stmt = conn.prepare("SELECT id, text, completed FROM todos")?;
    let todos: Vec<Todo> = stmt.query_map([], |row| {
        Ok(Todo {
            id: row.get(0)?,
            text: row.get(1)?,
            completed: row.get(2)?,
        })
    })?
    .filter_map(Result::ok)
    .collect();
    Ok(todos)
}

pub fn new_todo(conn: &Connection, text: &str) -> Result<Todo> {
    conn.execute(
        "INSERT INTO todos (text, completed) VALUES (?1, 0)",
        &[&text],
    )?;
    Ok(Todo {
        id: conn.last_insert_rowid(),
        text: text.to_string(),
        completed: false,
    })
}

pub fn toggle_todo(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE todos SET completed = 1 - completed WHERE id = ?1",
        &[&id],
    )?;
    Ok(())
}

pub fn delete_todo(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM todos WHERE id = ?1", &[&id])?;
    Ok(())
}
