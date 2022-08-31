use rusqlite::{Connection, Result};

pub struct Todo {
    pub id: i64,
    pub position: i64,
    pub day_id: i64,
    pub text: String,
    pub completed: bool,
}

impl Todo {
    pub fn new(db: &Connection, text: &str, day_id: i64) -> Result<Self> {
        db.execute(
            "INSERT INTO todos (text, completed, day_id) VALUES (?1, 0, ?2)",
            &[text, day_id.to_string().as_str()],
        )?;
        let last_id = db.last_insert_rowid();
        db.execute(
            "UPDATE todos SET position = ?1 WHERE id = ?2",
            &[&last_id, &last_id],
        )?;
        Ok(Self {
            id: db.last_insert_rowid(),
            day_id,
            position: last_id,
            text: text.to_string(),
            completed: false,
        })
    }

    pub fn get_all(db: &Connection, day_id: i64) -> Result<Vec<Self>> {
        let mut stmt = db.prepare("SELECT id, day_id, position, text, completed FROM todos WHERE day_id = ?1 ORDER BY position ASC")?;
        let todos: Vec<Self> = stmt
            .query_map([day_id], |row| {
                Ok(Self {
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

    pub fn get_text(&self) -> String {
        if self.completed {
            format!("{} {}", "[x]", self.text)
        } else {
            format!("{} {}", "[ ]", self.text)
        }
    }

    pub fn toggle(&mut self, db: &Connection) -> Result<()> {
        db.execute(
            "UPDATE todos SET completed = 1 - completed WHERE id = ?1",
            &[&self.id],
        )?;
        self.completed = !self.completed;
        Ok(())
    }

    pub fn delete(&self, db: &Connection) -> Result<()> {
        db.execute("DELETE FROM todos WHERE id = ?1", &[&self.id])?;
        Ok(())
    }

    pub fn update_positions(db: &Connection, todos: &[Self]) -> Result<()> {
        for (i, todo) in todos.iter().enumerate() {
            db.execute(
                "UPDATE todos SET position = ?1 WHERE id = ?2",
                &[&i.to_string().as_str(), &todo.id.to_string().as_str()],
            )?;
        }
        Ok(())
    }
}

pub struct DailyTodo {
    pub id: i64,
    pub position: i64,
    pub text: String,
}

impl DailyTodo {
    pub fn new(db: &Connection, text: &str) -> Result<Self> {
        db.execute("INSERT INTO daily_todos (text) VALUES (?1)", &[text])?;
        let id = db.last_insert_rowid();
        db.execute(
            "UPDATE daily_todos SET position = ?1 WHERE id = ?2",
            &[&id, &id],
        )?;
        Ok(Self {
            id,
            position: id,
            text: String::from(text),
        })
    }

    pub fn get_all(db: &Connection) -> Result<Vec<Self>> {
        let mut stmt =
            db.prepare("SELECT id, position, text FROM daily_todos ORDER BY position ASC")?;
        let days: Vec<Self> = stmt
            .query_map([], |r| {
                Ok(Self {
                    id: r.get(0)?,
                    position: r.get(1)?,
                    text: r.get(2)?,
                })
            })?
            .filter_map(Result::ok)
            .collect();
        Ok(days)
    }

    pub fn update_positions(db: &Connection, todos: &[Self]) -> Result<()> {
        for (i, todo) in todos.iter().enumerate() {
            db.execute(
                "UPDATE daily_todos SET position = ?1 WHERE id = ?2",
                &[&i.to_string().as_str(), &todo.id.to_string().as_str()],
            )?;
        }
        Ok(())
    }

    pub fn get_text(&self) -> String {
        self.text.to_string()
    }

    pub fn delete(&self, db: &Connection) -> Result<()> {
        db.execute("DELETE FROM daily_todos WHERE id = ?1", &[&self.id])?;
        Ok(())
    }
}

pub struct Day {
    pub id: i64,
    pub count_todos: i64,
    pub done_todos: i64,
    pub notes: String,
    pub date: String,
    pub todos: Vec<Todo>,
}

impl Day {
    pub fn new(db: &Connection, date: &str) -> Result<Self> {
        db.execute(
            "INSERT INTO days (count_todos, done_todos, notes, date) VALUES (0,0,'',?1)",
            &[date],
        )?;
        let id = db.last_insert_rowid();
        let daily_todos = DailyTodo::get_all(db)?;
        let todos: Vec<Todo> = daily_todos
            .iter()
            .map(|todo| Todo::new(db, &todo.text, id).unwrap())
            .collect();
        Ok(Self {
            id,
            count_todos: 0,
            done_todos: 0,
            notes: String::new(),
            date: String::from(date),
            todos,
        })
    }

    pub fn get(db: &Connection, day_id: i64) -> Result<Self> {
        let mut stmt = db.prepare(
            "SELECT id, count_todos, done_todos, notes, date FROM days WHERE id = ?1 LIMIT 1",
        )?;
        let day = stmt.query_row([day_id], |r| {
            let id = r.get(0)?;
            let todos = Todo::get_all(db, id)?;
            Ok(Self {
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

    pub fn set_notes(&mut self, db: &Connection) -> Result<()> {
        db.execute(
            "UPDATE days SET notes = ?1 WHERE id = ?2",
            &[&self.notes, self.id.to_string().as_str()],
        )?;
        Ok(())
    }

    pub fn update_counts(&mut self, db: &Connection) -> Result<()> {
        self.count_todos = self.todos.len() as i64;
        self.done_todos = self.todos.iter().filter(|t| t.completed).count() as i64;
        db.execute(
            "UPDATE days SET count_todos = ?1, done_todos = ?2 WHERE id = ?3",
            &[&self.count_todos, &self.done_todos, &self.id],
        )?;
        Ok(())
    }

    pub fn add_todo(&mut self, db: &Connection, item: Todo) -> Result<()> {
        self.todos.push(item);
        self.update_counts(db)?;
        Ok(())
    }

    pub fn remove_todo(&mut self, db: &Connection, index: usize) -> Result<()> {
        self.todos.remove(index);
        self.update_counts(db)?;
        Ok(())
    }
}

pub struct DayShort {
    pub id: i64,
    pub date: String,
    pub string: String,
    pub done: usize,
}

impl DayShort {
    pub fn get_all(db: &Connection) -> Result<Vec<Self>> {
        let mut stmt = db.prepare("SELECT id, date, count_todos, done_todos FROM days")?;
        let days: Vec<Self> = stmt
            .query_map([], |r| {
                let count: usize = r.get(2)?;
                let done: usize = r.get(3)?;
                Ok(Self {
                    id: r.get(0)?,
                    date: r.get(1)?,
                    string: format!("{}/{}", done, count),
                    done,
                })
            })?
            .filter_map(Result::ok)
            .collect();
        Ok(days)
    }
}

pub fn init_connection(path: &str) -> Result<Connection> {
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
    conn.execute(
        "CREATE TABLE IF NOT EXISTS daily_todos (
            id INTEGER PRIMARY KEY,
            position INTEGER,
            text TEXT NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}
