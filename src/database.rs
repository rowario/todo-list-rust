use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct Todo {
    pub id: i64,
    pub position: i64,
    pub day_id: i64,
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

#[derive(Debug)]
pub struct Day {
    pub id: i64,
    pub count_todos: i64,
    pub done_todos: i64,
    pub notes: String,
    pub date: String,
    pub todos: Vec<Todo>,
}

pub struct DayShort {
    pub id: i64,
    pub date: String,
}

pub fn init(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY,
            position INTEGER,
            day_id INTEGER,
            text TEXT NOT NULL,
            completed INTEGER NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS days (
            id INTEGER PRIMARY KEY,
            count_todos INTEGER NOT NULL,
            done_todos INTEGER NOT NULL,
            notes TEXT NOT NULL,
            date TEXT NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

pub fn get_days(conn: &Connection) -> Result<Vec<DayShort>> {
    let mut stmt = conn.prepare("SELECT id, date FROM days")?;
    let days: Vec<DayShort> = stmt.query_map([], |r| {
        Ok(DayShort {
            id: r.get(0)?,
            date: r.get(1)?,
        })
    })?
        .filter_map(Result::ok)
        .collect();
    Ok(days)
}

pub fn get_day(conn: &Connection, day_id: i64) -> Result<Day> {
    let mut stmt = conn.prepare("SELECT id, count_todos, done_todos, notes, date FROM days WHERE id = ?1 LIMIT 1")?;
    let day = stmt.query_row([day_id], |r| {
        let id = r.get(0)?;
        let todos = get_todos(conn, id)?;
        Ok(Day {
            id,
            count_todos: r.get(1)?,
            done_todos: r.get(2)?,
            notes: r.get(3)?,
            date: r.get(4)?,
            todos,
        })
    })?;
    Ok(day)
}

pub fn new_day(conn: &Connection, date: &str) -> Result<Day> {
    conn.execute(
        "INSERT INTO days (count_todos, done_todos, notes, date) VALUES (0,0,'',?1)",
        &[date],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Day {
        id,
        count_todos: 0,
        done_todos: 0,
        notes: String::new(),
        date: String::from(date),
        todos: vec![],
    })
}

fn get_todos(conn: &Connection, day_id: i64) -> Result<Vec<Todo>> {
    let mut stmt = conn.prepare("SELECT id, day_id, position, text, completed FROM todos WHERE day_id = ?1 ORDER BY position ASC")?;
    let todos: Vec<Todo> = stmt.query_map([day_id], |row| {
        Ok(Todo {
            id: row.get(0)?,
            day_id: row.get(1)?,
            position: row.get(2)?,
            text: row.get(3)?,
            completed: row.get(4)?,
        })
    })?
        .filter_map(Result::ok)
        .collect();
    Ok(todos)
}

pub fn new_todo(conn: &Connection, text: &str, day_id: i64) -> Result<Todo> {
    conn.execute(
        "INSERT INTO todos (text, completed, day_id) VALUES (?1, 0, ?2)",
        &[text, day_id.to_string().as_str()],
    )?;
    let last_id = conn.last_insert_rowid();
    conn.execute(
        "UPDATE todos SET position = ?1 WHERE id = ?2",
        &[&last_id, &last_id],
    )?;
    Ok(Todo {
        id: conn.last_insert_rowid(),
        day_id,
        position: last_id,
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

pub fn update_todos_positions(conn: &Connection, todos: &[Todo]) -> Result<()> {
    for (i, todo) in todos.iter().enumerate() {
        conn.execute(
            "UPDATE todos SET position = ?1 WHERE id = ?2",
            &[&i.to_string().as_str(), &todo.id.to_string().as_str()],
        )?;
    }
    Ok(())
}

