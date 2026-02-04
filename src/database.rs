use rusqlite::{Connection, params};
use std::collections::HashSet;
use std::convert::From;
use std::path::PathBuf;

pub enum TaskStatus {
    NotDone = 0,
    Passed = 1,
    Failed = 2,
}

impl From<i32> for TaskStatus {
    fn from(value: i32) -> Self {
        match value {
            0 => TaskStatus::NotDone,
            1 => TaskStatus::Passed,
            2 => TaskStatus::Failed,
            _ => TaskStatus::NotDone,
        }
    }
}

pub struct Database {
    file: PathBuf,
}

impl Database {
    pub fn new(file: PathBuf) -> rusqlite::Result<Self> {
        if !file.exists() {
            let conn = Connection::open(&file)?;
            conn.execute_batch(
                "
            BEGIN;
            CREATE TABLE tasks (
                    id        INTEGER PRIMARY KEY,
                    file_name TEXT,
                    task_name INTEGER NOT NULL,
                    passed    INTEGER DEFAULT 0,
                );
                CREATE TABLE weeks (
                    id INTEGER PRIMARY KEY AUTO_INCREMENT,
                    week TEXT NOT NULL,
                    task_id INTEGER,
                    FOREIGN KEY (task_id) REFERENCES tasks(id)
                );
                COMMIT;",
            )?;
        }
        Ok(Database { file })
    }

    pub fn add_task(&self, task_id: &i32, task_name: &str, week: &str) -> rusqlite::Result<()> {
        let conn = Connection::open(&self.file)?;

        conn.execute(
            "INSERT INTO tasks (id, task_name) VALUES (?1,?2);",
            params![task_id, task_name],
        )?;

        conn.execute(
            "INSERT INTO weeks (week, task_id) VALUES (?1,?2);",
            params![week, task_id],
        )?;

        Ok(())
    }

    pub fn get_passed_by_id(&self, task_id: &i32) -> rusqlite::Result<TaskStatus> {
        let conn = Connection::open(&self.file)?;
        let passed: i32 = conn.query_row(
            "SELECT passed FROM tasks WHERE id = ?1;",
            params![task_id],
            |row| row.get(0),
        )?;
        Ok(passed.into())
    }

    pub fn get_id_by_file_name(&self, file_name: &str) -> rusqlite::Result<i32> {
        let conn = Connection::open(&self.file)?;
        let id: i32 = conn.query_row(
            "SELECT id FROM tasks WHERE file_name = ?1;",
            params![file_name],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    pub fn get_week_by_file_name(&self, file_name: &str) -> rusqlite::Result<String> {
        let conn = Connection::open(&self.file)?;
        let week: String = conn.query_row(
            "SELECT w.week FROM tasks AS t JOIN weeks AS w ON t.id = w.task_id WHERE t.file_name = ?1;",
            params![file_name],
            |row| row.get(0)
        )?;
        Ok(week)
    }

    pub fn get_all_task_names(&self) -> rusqlite::Result<HashSet<String>> {
        let conn = Connection::open(&self.file)?;
        let mut stmt = conn.prepare("SELECT DISTINCT task_name FROM tasks;")?;
        let task_names: HashSet<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<_, _>>()?;
        Ok(task_names)
    }

    pub fn get_all_weeks(&self) -> rusqlite::Result<HashSet<String>> {
        let conn = Connection::open(&self.file)?;
        let mut stmt = conn.prepare("SELECT DISTINCT week FROM weeks;")?;
        let weeks: HashSet<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<_, _>>()?;
        Ok(weeks)
    }

    pub fn set_passed_by_id(&self, passed: TaskStatus, task_id: &i32) -> rusqlite::Result<()> {
        let conn = Connection::open(&self.file)?;
        conn.execute(
            "UPDATE tasks set passed = ?1 WHERE id = ?2;",
            params![passed as i32, task_id],
        )?;

        Ok(())
    }

    pub fn set_file_name_by_id(&self, file_name: &str, task_id: &i32) -> rusqlite::Result<()> {
        let conn = Connection::open(&self.file)?;
        conn.execute(
            "UPDATE tasks set file_name = ?1 WHERE id = ?2;",
            params![file_name, task_id],
        )?;
        Ok(())
    }
}
