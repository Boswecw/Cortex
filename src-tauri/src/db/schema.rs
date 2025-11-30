use crate::error::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub id: i64,
    pub path: String,
    pub filename: String,
    pub file_type: String,
    pub size: i64,
    pub created_at: String,
    pub modified_at: String,
    pub last_indexed: String,
    pub hash: Option<String>,
    pub root_path: String,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub file_id: i64,
    pub text_content: Option<String>,
    pub word_count: Option<i64>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_id: i64,
    pub path: String,
    pub filename: String,
    pub snippet: String,
    pub score: f64,
}

pub fn create_tables(conn: &Connection) -> Result<()> {
    // Files table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT UNIQUE NOT NULL,
            filename TEXT NOT NULL,
            file_type TEXT NOT NULL,
            size INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            modified_at TEXT NOT NULL,
            last_indexed TEXT NOT NULL,
            hash TEXT,
            root_path TEXT NOT NULL,
            is_deleted INTEGER DEFAULT 0
        )",
        [],
    )?;

    // File content table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS file_content (
            file_id INTEGER PRIMARY KEY,
            text_content TEXT,
            word_count INTEGER,
            summary TEXT,
            FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // FTS5 virtual table
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
            filename,
            content,
            content='file_content',
            content_rowid='file_id',
            tokenize='porter'
        )",
        [],
    )?;

    // Triggers for FTS sync
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS files_fts_insert AFTER INSERT ON file_content BEGIN
            INSERT INTO files_fts(rowid, filename, content)
            SELECT f.id, f.filename, new.text_content
            FROM files f WHERE f.id = new.file_id;
         END",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS files_fts_update AFTER UPDATE ON file_content BEGIN
            UPDATE files_fts SET content = new.text_content
            WHERE rowid = new.file_id;
         END",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS files_fts_delete AFTER DELETE ON file_content BEGIN
            DELETE FROM files_fts WHERE rowid = old.file_id;
         END",
        [],
    )?;

    // Indexes for performance
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_files_path ON files(path)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_files_modified ON files(modified_at)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_files_type ON files(file_type)",
        [],
    )?;

    Ok(())
}
