use rusqlite::Connection;
use std::collections::HashSet;
use std::convert::From;
use std::path::PathBuf;

pub enum TaskStatus {
    NotDone = 0,
    Passed = 1,
    Failed = 2,
}

impl From<u8> for TaskStatus {
    fn from(value: u8) -> Self {
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

    pub fn add_task(&self, task_id: &str, task_name: &str, week: &str) -> rusqlite::Result<()> {
        let conn = Connection::open(&self.file)?;
        let id: i32 = task_id.parse().unwrap();

        conn.execute(
            "INSERT INTO tasks (id, task_name) VALUES (?1,?2);",
            (id, task_name),
        )?;

        conn.execute(
            "INSERT INTO weeks (week, task_id) VALUES (?1,?2);",
            (week, id),
        )?;

        Ok(())
    }

    pub fn get_passed_by_id(&self, task_id: &i32) -> rusqlite::Result<TaskStatus> {
        let conn = Connection::open(&self.file)?;
        let passed: u8 = conn.query_row(
            "SELECT passed FROM tasks WHERE id = ?1;",
            (task_id,),
            |row| row.get(0),
        )?;
        Ok(passed.into())
    }

    pub fn get_id_by_file_name(&self, file_name: &str) -> rusqlite::Result<String> {
        let conn = Connection::open(&self.file)?;
        let id: String = conn.query_row(
            "SELECT id FROM tasks WHERE file_name = ?1;",
            (file_name,),
            |row| row.get(0),
        )?;
        Ok(id)
    }

    pub fn get_week_by_file_name(&self, file_name: &str) -> rusqlite::Result<String> {
        let conn = Connection::open(&self.file)?;
        let week: String = conn.query_row(
            "SELECT w.week FROM tasks AS t JOIN weeks AS w ON t.id = w.task_id WHERE t.file_name = ?1;",
            (file_name,),
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

    pub fn set_passed_by_id(&self, passed: TaskStatus, task_id: &str) -> rusqlite::Result<()> {
        let conn = Connection::open(&self.file)?;
        let id: i32 = task_id.parse().unwrap();
        conn.execute(
            "UPDATE tasks set passed = ?1 WHERE id = ?2;",
            (passed as u8, id),
        )?;

        Ok(())
    }

    pub fn set_file_name_by_id(&self, file_name: &str, task_id: &str) -> rusqlite::Result<()> {
        let conn = Connection::open(&self.file)?;
        let id: i32 = task_id.parse().unwrap();
        conn.execute(
            "UPDATE tasks set file_name = ?1 WHERE id = ?2;",
            (file_name, id),
        )?;
        Ok(())
    }
}
