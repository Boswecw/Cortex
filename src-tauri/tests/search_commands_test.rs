use cortex_lib::commands::search::{get_file_detail, get_search_stats, search_files, SearchFilters};
use cortex_lib::db::{insert_file, upsert_file_content};
use cortex_lib::state::AppState;
use std::sync::Arc;
use tempfile::TempDir;
use std::fs;
use std::io::Write;

/// Helper to create test files
fn create_test_file(dir: &std::path::Path, name: &str, content: &str) {
    let path = dir.join(name);
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}

#[tokio::test]
async fn test_search_files_basic() {
    let state = Arc::new(AppState::new().await.unwrap());
    let db = state.db.read().await;
    let conn = db.get_connection();

    // Insert test files
    let file_id1 = insert_file(
        conn,
        "/test/rust_tutorial.txt",
        "rust_tutorial.txt",
        "txt",
        1024,
        "2025-11-29T00:00:00Z",
        "2025-11-29T00:00:00Z",
        None,
        "/test",
    )
    .unwrap();

    upsert_file_content(
        conn,
        file_id1,
        Some("This is a comprehensive Rust programming tutorial"),
        Some("Rust tutorial"),
    )
    .unwrap();

    let file_id2 = insert_file(
        conn,
        "/test/javascript_guide.md",
        "javascript_guide.md",
        "md",
        2048,
        "2025-11-29T00:00:00Z",
        "2025-11-29T00:00:00Z",
        None,
        "/test",
    )
    .unwrap();

    upsert_file_content(
        conn,
        file_id2,
        Some("JavaScript development guide with modern frameworks"),
        Some("JS guide"),
    )
    .unwrap();

    drop(db); // Release lock before calling command

    // Search for "rust"
    let results = search_files(
        "rust".to_string(),
        None,
        None,
        None,
        tauri::State::from(state.as_ref()),
    )
    .await
    .unwrap();

    assert_eq!(results.total, 1);
    assert!(results.results[0].filename.contains("rust"));
    assert!(results.query_time_ms > 0);
}

#[tokio::test]
async fn test_search_files_with_filters() {
    let state = Arc::new(AppState::new().await.unwrap());
    let db = state.db.read().await;
    let conn = db.get_connection();

    // Insert test files with different types and sizes
    let file_id1 = insert_file(
        conn,
        "/test/small.txt",
        "small.txt",
        "txt",
        500,
        "2025-11-29T00:00:00Z",
        "2025-11-29T00:00:00Z",
        None,
        "/test",
    )
    .unwrap();

    upsert_file_content(conn, file_id1, Some("tutorial content"), None).unwrap();

    let file_id2 = insert_file(
        conn,
        "/test/large.md",
        "large.md",
        "md",
        5000,
        "2025-11-29T00:00:00Z",
        "2025-11-29T00:00:00Z",
        None,
        "/test",
    )
    .unwrap();

    upsert_file_content(conn, file_id2, Some("tutorial guide"), None).unwrap();

    drop(db); // Release lock

    // Search with file type filter
    let filters = Some(SearchFilters {
        file_type: Some("txt".to_string()),
        min_size: None,
        max_size: None,
        date_from: None,
        date_to: None,
    });

    let results = search_files(
        "tutorial".to_string(),
        filters,
        None,
        None,
        tauri::State::from(state.as_ref()),
    )
    .await
    .unwrap();

    assert_eq!(results.total, 1);
    assert_eq!(results.results[0].filename, "small.txt");
}

#[tokio::test]
async fn test_search_files_with_size_filter() {
    let state = Arc::new(AppState::new().await.unwrap());
    let db = state.db.read().await;
    let conn = db.get_connection();

    // Insert files with different sizes
    let file_id1 = insert_file(
        conn,
        "/test/small.txt",
        "small.txt",
        "txt",
        500,
        "2025-11-29T00:00:00Z",
        "2025-11-29T00:00:00Z",
        None,
        "/test",
    )
    .unwrap();

    upsert_file_content(conn, file_id1, Some("guide content"), None).unwrap();

    let file_id2 = insert_file(
        conn,
        "/test/large.txt",
        "large.txt",
        "txt",
        5000,
        "2025-11-29T00:00:00Z",
        "2025-11-29T00:00:00Z",
        None,
        "/test",
    )
    .unwrap();

    upsert_file_content(conn, file_id2, Some("guide content large"), None).unwrap();

    drop(db);

    // Filter by size: >= 1000 bytes
    let filters = Some(SearchFilters {
        file_type: None,
        min_size: Some(1000),
        max_size: None,
        date_from: None,
        date_to: None,
    });

    let results = search_files(
        "guide".to_string(),
        filters,
        None,
        None,
        tauri::State::from(state.as_ref()),
    )
    .await
    .unwrap();

    assert_eq!(results.total, 1);
    assert_eq!(results.results[0].filename, "large.txt");
}

#[tokio::test]
async fn test_search_files_with_pagination() {
    let state = Arc::new(AppState::new().await.unwrap());
    let db = state.db.read().await;
    let conn = db.get_connection();

    // Insert multiple files
    for i in 1..=5 {
        let file_id = insert_file(
            conn,
            &format!("/test/doc{}.txt", i),
            &format!("doc{}.txt", i),
            "txt",
            1000,
            "2025-11-29T00:00:00Z",
            "2025-11-29T00:00:00Z",
            None,
            "/test",
        )
        .unwrap();

        upsert_file_content(conn, file_id, Some("programming tutorial"), None).unwrap();
    }

    drop(db);

    // Get first 2 results
    let results = search_files(
        "programming".to_string(),
        None,
        Some(2),
        Some(0),
        tauri::State::from(state.as_ref()),
    )
    .await
    .unwrap();

    assert_eq!(results.total, 2);

    // Get next 2 results
    let results = search_files(
        "programming".to_string(),
        None,
        Some(2),
        Some(2),
        tauri::State::from(state.as_ref()),
    )
    .await
    .unwrap();

    assert_eq!(results.total, 2);
}

#[tokio::test]
async fn test_search_files_empty_query() {
    let state = Arc::new(AppState::new().await.unwrap());

    let result = search_files(
        "".to_string(),
        None,
        None,
        None,
        tauri::State::from(state.as_ref()),
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Query cannot be empty"));
}

#[tokio::test]
async fn test_get_file_detail() {
    let state = Arc::new(AppState::new().await.unwrap());
    let db = state.db.read().await;
    let conn = db.get_connection();

    // Insert test file
    let file_id = insert_file(
        conn,
        "/test/document.txt",
        "document.txt",
        "txt",
        1500,
        "2025-11-28T10:00:00Z",
        "2025-11-29T15:30:00Z",
        None,
        "/test",
    )
    .unwrap();

    let long_content = "This is a very long document content. ".repeat(50);
    upsert_file_content(
        conn,
        file_id,
        Some(&long_content),
        Some("Document summary"),
    )
    .unwrap();

    drop(db);

    // Get detail without full content
    let detail = get_file_detail(
        file_id,
        Some(false),
        tauri::State::from(state.as_ref()),
    )
    .await
    .unwrap();

    assert_eq!(detail.id, file_id);
    assert_eq!(detail.filename, "document.txt");
    assert_eq!(detail.file_type, "txt");
    assert_eq!(detail.size, 1500);
    assert!(detail.content_preview.is_some());
    assert!(detail.full_content.is_none());
    assert_eq!(detail.summary, Some("Document summary".to_string()));

    // Get detail with full content
    let detail = get_file_detail(
        file_id,
        Some(true),
        tauri::State::from(state.as_ref()),
    )
    .await
    .unwrap();

    assert!(detail.full_content.is_some());
    assert_eq!(detail.full_content.unwrap().len(), long_content.len());
}

#[tokio::test]
async fn test_get_file_detail_not_found() {
    let state = Arc::new(AppState::new().await.unwrap());

    let result = get_file_detail(
        99999,
        Some(false),
        tauri::State::from(state.as_ref()),
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_get_search_stats() {
    let state = Arc::new(AppState::new().await.unwrap());
    let db = state.db.read().await;
    let conn = db.get_connection();

    // Insert files
    for i in 1..=3 {
        let file_id = insert_file(
            conn,
            &format!("/test/file{}.txt", i),
            &format!("file{}.txt", i),
            "txt",
            1000 * i as i64,
            "2025-11-29T00:00:00Z",
            "2025-11-29T00:00:00Z",
            None,
            "/test",
        )
        .unwrap();

        // Only index 2 of the 3 files
        if i <= 2 {
            upsert_file_content(conn, file_id, Some("content"), None).unwrap();
        }
    }

    drop(db);

    let stats = get_search_stats(tauri::State::from(state.as_ref()))
        .await
        .unwrap();

    assert_eq!(stats.total_files, 3);
    assert_eq!(stats.indexed_files, 2);
    assert_eq!(stats.total_size_bytes, 1000 + 2000 + 3000);
}

#[tokio::test]
async fn test_search_with_snippets() {
    let state = Arc::new(AppState::new().await.unwrap());
    let db = state.db.read().await;
    let conn = db.get_connection();

    let file_id = insert_file(
        conn,
        "/test/article.txt",
        "article.txt",
        "txt",
        2000,
        "2025-11-29T00:00:00Z",
        "2025-11-29T00:00:00Z",
        None,
        "/test",
    )
    .unwrap();

    upsert_file_content(
        conn,
        file_id,
        Some("Before text. The Rust programming language is amazing. After text."),
        None,
    )
    .unwrap();

    drop(db);

    let results = search_files(
        "rust".to_string(),
        None,
        None,
        None,
        tauri::State::from(state.as_ref()),
    )
    .await
    .unwrap();

    assert_eq!(results.total, 1);
    // Snippet should contain highlighted match
    assert!(results.results[0].snippet.contains("<mark>"));
    assert!(results.results[0].snippet.contains("</mark>"));
}
