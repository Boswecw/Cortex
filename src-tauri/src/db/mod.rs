mod schema;
pub mod operations;

pub use schema::*;
pub use operations::*;

use crate::error::{CortexError, Result};
use rusqlite::{Connection, params};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Clone for Database {
    fn clone(&self) -> Self {
        // Create a new connection to the same database
        // This is safe with SQLite WAL mode (multiple readers allowed)
        let db_path = Self::get_db_path().expect("Failed to get database path");
        let conn = Connection::open(&db_path).expect("Failed to clone database connection");

        // Apply same performance settings
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA cache_size=-64000;
             PRAGMA temp_store=MEMORY;
             PRAGMA mmap_size=30000000000;"
        ).expect("Failed to configure cloned connection");

        Self { conn }
    }
}

impl Database {
    pub async fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;

        // Configure SQLite for performance
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA cache_size=-64000;
             PRAGMA temp_store=MEMORY;
             PRAGMA mmap_size=30000000000;"
        )?;

        let mut db = Self { conn };
        db.initialize_schema().await?;

        Ok(db)
    }

    fn get_db_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| CortexError::Internal {
                message: "Cannot determine home directory".to_string(),
            })?;

        Ok(PathBuf::from(home).join(".cortex").join("db.sqlite"))
    }

    async fn initialize_schema(&mut self) -> Result<()> {
        schema::create_tables(&self.conn)?;
        Ok(())
    }

    pub fn get_connection(&self) -> &Connection {
        &self.conn
    }
}
