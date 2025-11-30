use cortex_lib::db::Database;
use cortex_lib::error::Result;

#[tokio::test]
async fn test_database_creation() -> Result<()> {
    // Create a test database
    let db = Database::new().await?;

    // Verify connection is valid
    let conn = db.get_connection();
    let version: String = conn.query_row("SELECT sqlite_version()", [], |row| row.get(0))?;

    assert!(!version.is_empty());
    println!("SQLite version: {}", version);

    Ok(())
}

#[tokio::test]
async fn test_schema_creation() -> Result<()> {
    let db = Database::new().await?;
    let conn = db.get_connection();

    // Check that files table exists
    let table_exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='files'",
        [],
        |row| {
            let count: i32 = row.get(0)?;
            Ok(count > 0)
        },
    )?;

    assert!(table_exists, "files table should exist");

    // Check that file_content table exists
    let table_exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='file_content'",
        [],
        |row| {
            let count: i32 = row.get(0)?;
            Ok(count > 0)
        },
    )?;

    assert!(table_exists, "file_content table should exist");

    // Check that FTS5 virtual table exists
    let table_exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='files_fts'",
        [],
        |row| {
            let count: i32 = row.get(0)?;
            Ok(count > 0)
        },
    )?;

    assert!(table_exists, "files_fts virtual table should exist");

    Ok(())
}
