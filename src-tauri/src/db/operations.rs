use crate::db::schema::{File, FileContent, SearchResult};
use crate::error::{CortexError, Result};
use rusqlite::{params, Connection};

/// Insert a new file record into the database
pub fn insert_file(
    conn: &Connection,
    path: &str,
    filename: &str,
    file_type: &str,
    size: i64,
    created_at: &str,
    modified_at: &str,
    hash: Option<&str>,
    root_path: &str,
) -> Result<i64> {
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO files (path, filename, file_type, size, created_at, modified_at, last_indexed, hash, root_path)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![path, filename, file_type, size, created_at, modified_at, now, hash, root_path],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get a file by ID
pub fn get_file_by_id(conn: &Connection, file_id: i64) -> Result<File> {
    let mut stmt = conn.prepare(
        "SELECT id, path, filename, file_type, size, created_at, modified_at, last_indexed, hash, root_path, is_deleted
         FROM files WHERE id = ?1"
    )?;

    let file = stmt.query_row(params![file_id], |row| {
        Ok(File {
            id: row.get(0)?,
            path: row.get(1)?,
            filename: row.get(2)?,
            file_type: row.get(3)?,
            size: row.get(4)?,
            created_at: row.get(5)?,
            modified_at: row.get(6)?,
            last_indexed: row.get(7)?,
            hash: row.get(8)?,
            root_path: row.get(9)?,
            is_deleted: row.get(10)?,
        })
    })?;

    Ok(file)
}

/// Get a file by path
pub fn get_file_by_path(conn: &Connection, path: &str) -> Result<Option<File>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, filename, file_type, size, created_at, modified_at, last_indexed, hash, root_path, is_deleted
         FROM files WHERE path = ?1"
    )?;

    let result = stmt.query_row(params![path], |row| {
        Ok(File {
            id: row.get(0)?,
            path: row.get(1)?,
            filename: row.get(2)?,
            file_type: row.get(3)?,
            size: row.get(4)?,
            created_at: row.get(5)?,
            modified_at: row.get(6)?,
            last_indexed: row.get(7)?,
            hash: row.get(8)?,
            root_path: row.get(9)?,
            is_deleted: row.get(10)?,
        })
    });

    match result {
        Ok(file) => Ok(Some(file)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Update file metadata
pub fn update_file(
    conn: &Connection,
    file_id: i64,
    size: Option<i64>,
    modified_at: Option<&str>,
    hash: Option<&str>,
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();

    // Build dynamic update query based on provided fields
    let mut updates = Vec::new();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(s) = size {
        updates.push("size = ?");
        params_vec.push(Box::new(s));
    }
    if let Some(m) = modified_at {
        updates.push("modified_at = ?");
        params_vec.push(Box::new(m.to_string()));
    }
    if let Some(h) = hash {
        updates.push("hash = ?");
        params_vec.push(Box::new(h.to_string()));
    }

    updates.push("last_indexed = ?");
    params_vec.push(Box::new(now));

    params_vec.push(Box::new(file_id));

    let query = format!(
        "UPDATE files SET {} WHERE id = ?",
        updates.join(", ")
    );

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

    conn.execute(&query, params_refs.as_slice())?;

    Ok(())
}

/// Mark a file as deleted (soft delete)
pub fn mark_file_deleted(conn: &Connection, file_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE files SET is_deleted = 1 WHERE id = ?1",
        params![file_id],
    )?;

    Ok(())
}

/// Delete a file permanently
pub fn delete_file(conn: &Connection, file_id: i64) -> Result<()> {
    conn.execute("DELETE FROM files WHERE id = ?1", params![file_id])?;
    Ok(())
}

/// Insert or update file content
pub fn upsert_file_content(
    conn: &Connection,
    file_id: i64,
    text_content: Option<&str>,
    summary: Option<&str>,
) -> Result<()> {
    let word_count = text_content.map(|c| c.split_whitespace().count() as i64);

    conn.execute(
        "INSERT INTO file_content (file_id, text_content, word_count, summary)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(file_id) DO UPDATE SET
            text_content = excluded.text_content,
            word_count = excluded.word_count,
            summary = excluded.summary",
        params![file_id, text_content, word_count, summary],
    )?;

    Ok(())
}

/// Get file content by file ID
pub fn get_file_content(conn: &Connection, file_id: i64) -> Result<Option<FileContent>> {
    let mut stmt = conn.prepare(
        "SELECT file_id, text_content, word_count, summary FROM file_content WHERE file_id = ?1"
    )?;

    let result = stmt.query_row(params![file_id], |row| {
        Ok(FileContent {
            file_id: row.get(0)?,
            text_content: row.get(1)?,
            word_count: row.get(2)?,
            summary: row.get(3)?,
        })
    });

    match result {
        Ok(content) => Ok(Some(content)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Get total file count
pub fn get_file_count(conn: &Connection) -> Result<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM files WHERE is_deleted = 0",
        [],
        |row| row.get(0),
    )?;

    Ok(count)
}

/// Get indexed file count (files with content)
pub fn get_indexed_file_count(conn: &Connection) -> Result<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT f.id) FROM files f
         INNER JOIN file_content fc ON f.id = fc.file_id
         WHERE f.is_deleted = 0",
        [],
        |row| row.get(0),
    )?;

    Ok(count)
}

/// List files with pagination
pub fn list_files(
    conn: &Connection,
    limit: usize,
    offset: usize,
) -> Result<Vec<File>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, filename, file_type, size, created_at, modified_at, last_indexed, hash, root_path, is_deleted
         FROM files
         WHERE is_deleted = 0
         ORDER BY modified_at DESC
         LIMIT ?1 OFFSET ?2"
    )?;

    let files = stmt.query_map(params![limit, offset], |row| {
        Ok(File {
            id: row.get(0)?,
            path: row.get(1)?,
            filename: row.get(2)?,
            file_type: row.get(3)?,
            size: row.get(4)?,
            created_at: row.get(5)?,
            modified_at: row.get(6)?,
            last_indexed: row.get(7)?,
            hash: row.get(8)?,
            root_path: row.get(9)?,
            is_deleted: row.get(10)?,
        })
    })?
    .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(files)
}

/// Search files using FTS5
pub fn search_files_fts(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    // Validate query is not empty
    if query.trim().is_empty() {
        return Err(CortexError::InvalidQuery {
            query: query.to_string(),
            reason: "Query cannot be empty".to_string(),
        });
    }

    let mut stmt = conn.prepare(
        "SELECT f.id, f.path, f.filename,
                snippet(files_fts, 1, '<mark>', '</mark>', '...', 32) as snippet,
                rank
         FROM files_fts
         INNER JOIN files f ON files_fts.rowid = f.id
         WHERE files_fts MATCH ?1 AND f.is_deleted = 0
         ORDER BY rank
         LIMIT ?2"
    )?;

    let results = stmt.query_map(params![query, limit], |row| {
        Ok(SearchResult {
            file_id: row.get(0)?,
            path: row.get(1)?,
            filename: row.get(2)?,
            snippet: row.get(3)?,
            score: row.get(4)?,
        })
    })?
    .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(results)
}

/// Get database statistics
pub fn get_db_stats(conn: &Connection) -> Result<(i64, i64, i64)> {
    let total_files = get_file_count(conn)?;
    let indexed_files = get_indexed_file_count(conn)?;

    let total_size: i64 = conn.query_row(
        "SELECT COALESCE(SUM(size), 0) FROM files WHERE is_deleted = 0",
        [],
        |row| row.get(0),
    )?;

    Ok((total_files, indexed_files, total_size))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::create_tables;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;"
        ).unwrap();
        create_tables(&conn).unwrap();
        conn
    }

    #[test]
    fn test_insert_and_get_file() {
        let conn = setup_test_db();

        let file_id = insert_file(
            &conn,
            "/test/path/file.txt",
            "file.txt",
            "txt",
            1024,
            "2025-11-29T00:00:00Z",
            "2025-11-29T00:00:00Z",
            Some("abc123"),
            "/test/path",
        ).unwrap();

        assert!(file_id > 0);

        let file = get_file_by_id(&conn, file_id).unwrap();
        assert_eq!(file.path, "/test/path/file.txt");
        assert_eq!(file.filename, "file.txt");
        assert_eq!(file.size, 1024);
        assert_eq!(file.hash, Some("abc123".to_string()));
    }

    #[test]
    fn test_get_file_by_path() {
        let conn = setup_test_db();

        insert_file(
            &conn,
            "/test/file.txt",
            "file.txt",
            "txt",
            100,
            "2025-11-29T00:00:00Z",
            "2025-11-29T00:00:00Z",
            None,
            "/test",
        ).unwrap();

        let file = get_file_by_path(&conn, "/test/file.txt").unwrap();
        assert!(file.is_some());
        assert_eq!(file.unwrap().filename, "file.txt");

        let not_found = get_file_by_path(&conn, "/nonexistent.txt").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_update_file() {
        let conn = setup_test_db();

        let file_id = insert_file(
            &conn,
            "/test/file.txt",
            "file.txt",
            "txt",
            100,
            "2025-11-29T00:00:00Z",
            "2025-11-29T00:00:00Z",
            None,
            "/test",
        ).unwrap();

        update_file(&conn, file_id, Some(200), Some("2025-11-30T00:00:00Z"), Some("newhash")).unwrap();

        let file = get_file_by_id(&conn, file_id).unwrap();
        assert_eq!(file.size, 200);
        assert_eq!(file.modified_at, "2025-11-30T00:00:00Z");
        assert_eq!(file.hash, Some("newhash".to_string()));
    }

    #[test]
    fn test_soft_delete() {
        let conn = setup_test_db();

        let file_id = insert_file(
            &conn,
            "/test/file.txt",
            "file.txt",
            "txt",
            100,
            "2025-11-29T00:00:00Z",
            "2025-11-29T00:00:00Z",
            None,
            "/test",
        ).unwrap();

        mark_file_deleted(&conn, file_id).unwrap();

        let file = get_file_by_id(&conn, file_id).unwrap();
        assert_eq!(file.is_deleted, true);

        // Should not appear in counts
        let count = get_file_count(&conn).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_upsert_file_content() {
        let conn = setup_test_db();

        let file_id = insert_file(
            &conn,
            "/test/file.txt",
            "file.txt",
            "txt",
            100,
            "2025-11-29T00:00:00Z",
            "2025-11-29T00:00:00Z",
            None,
            "/test",
        ).unwrap();

        // Insert content
        upsert_file_content(&conn, file_id, Some("Hello world"), Some("A greeting")).unwrap();

        let content = get_file_content(&conn, file_id).unwrap().unwrap();
        assert_eq!(content.text_content, Some("Hello world".to_string()));
        assert_eq!(content.word_count, Some(2));
        assert_eq!(content.summary, Some("A greeting".to_string()));

        // Update content
        upsert_file_content(&conn, file_id, Some("Hello world again"), None).unwrap();

        let updated = get_file_content(&conn, file_id).unwrap().unwrap();
        assert_eq!(updated.text_content, Some("Hello world again".to_string()));
        assert_eq!(updated.word_count, Some(3));
    }

    #[test]
    fn test_file_counts() {
        let conn = setup_test_db();

        // Insert 3 files
        let id1 = insert_file(&conn, "/test/1.txt", "1.txt", "txt", 100, "2025-11-29T00:00:00Z", "2025-11-29T00:00:00Z", None, "/test").unwrap();
        let id2 = insert_file(&conn, "/test/2.txt", "2.txt", "txt", 200, "2025-11-29T00:00:00Z", "2025-11-29T00:00:00Z", None, "/test").unwrap();
        let id3 = insert_file(&conn, "/test/3.txt", "3.txt", "txt", 300, "2025-11-29T00:00:00Z", "2025-11-29T00:00:00Z", None, "/test").unwrap();

        // Add content to 2 files
        upsert_file_content(&conn, id1, Some("content 1"), None).unwrap();
        upsert_file_content(&conn, id2, Some("content 2"), None).unwrap();

        let total = get_file_count(&conn).unwrap();
        let indexed = get_indexed_file_count(&conn).unwrap();

        assert_eq!(total, 3);
        assert_eq!(indexed, 2);

        // Mark one deleted
        mark_file_deleted(&conn, id3).unwrap();

        let total_after = get_file_count(&conn).unwrap();
        assert_eq!(total_after, 2);
    }

    #[test]
    fn test_fts_search() {
        let conn = setup_test_db();

        let id1 = insert_file(&conn, "/test/rust.txt", "rust.txt", "txt", 100, "2025-11-29T00:00:00Z", "2025-11-29T00:00:00Z", None, "/test").unwrap();
        let id2 = insert_file(&conn, "/test/python.txt", "python.txt", "txt", 200, "2025-11-29T00:00:00Z", "2025-11-29T00:00:00Z", None, "/test").unwrap();

        upsert_file_content(&conn, id1, Some("Rust is a systems programming language"), None).unwrap();
        upsert_file_content(&conn, id2, Some("Python is a high-level programming language"), None).unwrap();

        // Search for "rust"
        let results = search_files_fts(&conn, "rust", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "rust.txt");

        // Search for "programming"
        let results = search_files_fts(&conn, "programming", 10).unwrap();
        assert_eq!(results.len(), 2);

        // Empty query should error
        let err = search_files_fts(&conn, "", 10);
        assert!(err.is_err());
    }

    #[test]
    fn test_list_files_pagination() {
        let conn = setup_test_db();

        // Insert 5 files
        for i in 1..=5 {
            insert_file(
                &conn,
                &format!("/test/{}.txt", i),
                &format!("{}.txt", i),
                "txt",
                100 * i as i64,
                "2025-11-29T00:00:00Z",
                "2025-11-29T00:00:00Z",
                None,
                "/test"
            ).unwrap();
        }

        // Get first page (2 items)
        let page1 = list_files(&conn, 2, 0).unwrap();
        assert_eq!(page1.len(), 2);

        // Get second page
        let page2 = list_files(&conn, 2, 2).unwrap();
        assert_eq!(page2.len(), 2);

        // Get third page
        let page3 = list_files(&conn, 2, 4).unwrap();
        assert_eq!(page3.len(), 1);
    }

    #[test]
    fn test_db_stats() {
        let conn = setup_test_db();

        insert_file(&conn, "/test/1.txt", "1.txt", "txt", 100, "2025-11-29T00:00:00Z", "2025-11-29T00:00:00Z", None, "/test").unwrap();
        insert_file(&conn, "/test/2.txt", "2.txt", "txt", 200, "2025-11-29T00:00:00Z", "2025-11-29T00:00:00Z", None, "/test").unwrap();

        let (total, indexed, size) = get_db_stats(&conn).unwrap();
        assert_eq!(total, 2);
        assert_eq!(indexed, 0); // No content yet
        assert_eq!(size, 300); // 100 + 200
    }
}
