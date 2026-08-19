use std::path::{Path, PathBuf};
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use crate::database::schema::CREATE_HISTORY_TABLE;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub command: String,
    pub cwd: Option<String>,
    pub exit_code: Option<i32>,
    pub executed_at: i64,
    pub execution_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStats {
    pub total_records: usize,
    pub total_executions: usize,
    pub top_commands: Vec<(String, usize)>,
}

pub struct HistoryDb {
    conn: Connection,
}

impl HistoryDb {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        if let Some(parent) = path.as_ref().parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(CREATE_HISTORY_TABLE)?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(CREATE_HISTORY_TABLE)?;
        Ok(Self { conn })
    }

    pub fn default_db_path() -> PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "predictterm", "predictterm") {
            proj_dirs.data_dir().join("history.db")
        } else {
            PathBuf::from(".config/predictterm/history.db")
        }
    }

    pub fn open_default() -> Result<Self> {
        Self::open(Self::default_db_path())
    }

    pub fn record_command(&mut self, command: &str, cwd: Option<&str>, exit_code: Option<i32>) -> Result<i64> {
        let command = command.trim();
        if command.is_empty() {
            return Ok(0);
        }

        let now = Utc::now().timestamp();

        // Check if identical command already exists to increment count
        let mut check_stmt = self.conn.prepare("SELECT id, execution_count FROM command_history WHERE command = ?1 LIMIT 1")?;
        let existing = check_stmt
            .query_row(params![command], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })
            .ok();

        if let Some((id, count)) = existing {
            self.conn.execute(
                "UPDATE command_history SET cwd = ?1, exit_code = ?2, executed_at = ?3, execution_count = ?4 WHERE id = ?5",
                params![cwd, exit_code, now, count + 1, id],
            )?;
            Ok(id)
        } else {
            self.conn.execute(
                "INSERT INTO command_history (command, cwd, exit_code, executed_at, execution_count) VALUES (?1, ?2, ?3, ?4, 1)",
                params![command, cwd, exit_code, now],
            )?;
            Ok(self.conn.last_insert_rowid())
        }
    }

    pub fn search_prefix(&self, prefix: &str, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, command, cwd, exit_code, executed_at, execution_count 
             FROM command_history 
             WHERE command LIKE ?1 
             ORDER BY execution_count DESC, executed_at DESC 
             LIMIT ?2",
        )?;

        let pattern = format!("{}%", prefix);
        let rows = stmt.query_map(params![pattern, limit as i64], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                command: row.get(1)?,
                cwd: row.get(2)?,
                exit_code: row.get(3)?,
                executed_at: row.get(4)?,
                execution_count: row.get(5)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_recent(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, command, cwd, exit_code, executed_at, execution_count 
             FROM command_history 
             ORDER BY executed_at DESC 
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                command: row.get(1)?,
                cwd: row.get(2)?,
                exit_code: row.get(3)?,
                executed_at: row.get(4)?,
                execution_count: row.get(5)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_stats(&self) -> Result<HistoryStats> {
        let total_records: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM command_history",
            [],
            |r| r.get(0),
        )?;

        let total_executions: usize = self.conn.query_row(
            "SELECT COALESCE(SUM(execution_count), 0) FROM command_history",
            [],
            |r| r.get(0),
        )?;

        let mut top_stmt = self.conn.prepare(
            "SELECT command, execution_count FROM command_history ORDER BY execution_count DESC LIMIT 10",
        )?;
        let top_rows = top_stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, usize>(1)?))
        })?;

        let mut top_commands = Vec::new();
        for item in top_rows {
            top_commands.push(item?);
        }

        Ok(HistoryStats {
            total_records,
            total_executions,
            top_commands,
        })
    }

    pub fn delete_command(&mut self, command: &str) -> Result<usize> {
        self.conn.execute("DELETE FROM command_history WHERE command = ?1", params![command])
    }

    pub fn clear(&mut self) -> Result<usize> {
        self.conn.execute("DELETE FROM command_history", [])
    }
}
