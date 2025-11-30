use cortex_lib::db::{Database, insert_file, upsert_file_content, search_files_fts, get_file_by_id, get_db_stats};
use cortex_lib::error::Result;

#[tokio::test]
async fn test_full_indexing_pipeline() -> Result<()> {
    // Create a test database
    let db = Database::new().await?;
    let conn = db.get_connection();

    // Step 1: Insert a file
    let file_id = insert_file(
        conn,
        "/home/user/documents/rust_tutorial.md",
        "rust_tutorial.md",
        "md",
        5432,
        "2025-11-28T10:00:00Z",
        "2025-11-29T08:30:00Z",
        Some("abc123def456"),
        "/home/user/documents",
    )?;

    assert!(file_id > 0, "File should be inserted with a valid ID");

    // Step 2: Add content to the file
    let content = "# Rust Tutorial\n\nRust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.";
    upsert_file_content(
        conn,
        file_id,
        Some(content),
        Some("A tutorial about Rust programming language"),
    )?;

    // Step 3: Search for the content
    let results = search_files_fts(conn, "rust programming", 10)?;

    assert_eq!(results.len(), 1, "Should find exactly one result");
    assert_eq!(results[0].file_id, file_id);
    assert_eq!(results[0].filename, "rust_tutorial.md");
    assert!(results[0].snippet.contains("Rust"));

    // Step 4: Retrieve the file by ID
    let file = get_file_by_id(conn, file_id)?;
    assert_eq!(file.path, "/home/user/documents/rust_tutorial.md");
    assert_eq!(file.size, 5432);
    assert_eq!(file.file_type, "md");

    // Step 5: Check database statistics
    let (total_files, indexed_files, _total_size) = get_db_stats(conn)?;
    assert_eq!(total_files, 1);
    assert_eq!(indexed_files, 1);

    Ok(())
}

#[tokio::test]
async fn test_multiple_files_search() -> Result<()> {
    let db = Database::new().await?;
    let conn = db.get_connection();

    // Insert multiple files with different content
    let files = vec![
        ("rust.md", "Rust is a systems programming language focused on safety and performance."),
        ("python.md", "Python is a high-level programming language known for its simplicity."),
        ("javascript.md", "JavaScript is a versatile programming language for web development."),
        ("tutorial.md", "This tutorial covers Rust, Python, and JavaScript basics."),
    ];

    for (filename, content) in &files {
        let file_id = insert_file(
            conn,
            &format!("/docs/{}", filename),
            filename,
            "md",
            content.len() as i64,
            "2025-11-29T00:00:00Z",
            "2025-11-29T00:00:00Z",
            None,
            "/docs",
        )?;

        upsert_file_content(conn, file_id, Some(content), None)?;
    }

    // Search for "programming language"
    let results = search_files_fts(conn, "programming language", 10)?;
    assert!(results.len() >= 3, "Should find at least 3 files with 'programming language'");

    // Search for "rust"
    let rust_results = search_files_fts(conn, "rust", 10)?;
    assert!(rust_results.len() >= 2, "Should find files containing 'rust'");

    // Search for specific language
    let python_results = search_files_fts(conn, "python", 10)?;
    assert!(python_results.len() >= 2, "Should find files containing 'python'");

    Ok(())
}

#[tokio::test]
async fn test_update_and_reindex() -> Result<()> {
    let db = Database::new().await?;
    let conn = db.get_connection();

    // Insert initial file
    let file_id = insert_file(
        conn,
        "/test/document.md",
        "document.md",
        "md",
        100,
        "2025-11-29T00:00:00Z",
        "2025-11-29T00:00:00Z",
        None,
        "/test",
    )?;

    // Add initial content
    upsert_file_content(
        conn,
        file_id,
        Some("Initial content about databases"),
        None,
    )?;

    // Search for "databases"
    let results = search_files_fts(conn, "databases", 10)?;
    assert_eq!(results.len(), 1);

    // Update content
    upsert_file_content(
        conn,
        file_id,
        Some("Updated content about rust programming"),
        Some("Now about Rust"),
    )?;

    // Old search should return nothing
    let old_results = search_files_fts(conn, "databases", 10)?;
    assert_eq!(old_results.len(), 0, "Old content should not be found");

    // New search should work
    let new_results = search_files_fts(conn, "rust", 10)?;
    assert_eq!(new_results.len(), 1);
    assert!(new_results[0].snippet.to_lowercase().contains("rust"));

    Ok(())
}

#[tokio::test]
async fn test_large_batch_insert() -> Result<()> {
    let db = Database::new().await?;
    let conn = db.get_connection();

    let start = std::time::Instant::now();

    // Insert 100 files
    for i in 0..100 {
        let file_id = insert_file(
            conn,
            &format!("/test/file_{}.txt", i),
            &format!("file_{}.txt", i),
            "txt",
            1000 + i as i64,
            "2025-11-29T00:00:00Z",
            "2025-11-29T00:00:00Z",
            None,
            "/test",
        )?;

        upsert_file_content(
            conn,
            file_id,
            Some(&format!("Content for file number {} with some searchable text", i)),
            None,
        )?;
    }

    let duration = start.elapsed();
    println!("Inserted 100 files in {:?}", duration);

    // Verify count
    let (total, indexed, _) = get_db_stats(conn)?;
    assert_eq!(total, 100);
    assert_eq!(indexed, 100);

    // Test search performance
    let search_start = std::time::Instant::now();
    let results = search_files_fts(conn, "searchable", 100)?;
    let search_duration = search_start.elapsed();

    println!("Search completed in {:?}", search_duration);
    assert_eq!(results.len(), 100);

    // Performance assertion: search should be < 100ms
    assert!(search_duration.as_millis() < 100, "Search took too long: {:?}", search_duration);

    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let db = Database::new().await?;
    let conn = db.get_connection();

    // Test empty search query
    let empty_result = search_files_fts(conn, "", 10);
    assert!(empty_result.is_err(), "Empty query should return error");

    // Test whitespace-only query
    let whitespace_result = search_files_fts(conn, "   ", 10);
    assert!(whitespace_result.is_err(), "Whitespace-only query should return error");

    // Test getting non-existent file
    let non_existent = get_file_by_id(conn, 99999);
    assert!(non_existent.is_err(), "Getting non-existent file should error");

    Ok(())
}
