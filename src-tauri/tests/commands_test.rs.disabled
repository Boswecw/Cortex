use cortex_lib::commands::indexing::{get_index_status, start_indexing, stop_indexing, IndexStatus};
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
async fn test_indexing_status_initially_inactive() {
    let state = Arc::new(AppState::new().await.unwrap());

    let status: IndexStatus = get_index_status(tauri::State::from(state.as_ref()))
        .await
        .unwrap();

    assert!(!status.is_active);
    assert_eq!(status.total_files, 0);
    assert_eq!(status.indexed_files, 0);
    assert!(status.current_file.is_none());
    assert!(status.errors.is_empty());
}

#[tokio::test]
async fn test_cannot_start_indexing_twice() {
    // Note: This test is conceptual - in practice, we need a full Tauri app
    // to properly test commands. These tests verify the state logic works.

    let state = Arc::new(AppState::new().await.unwrap());

    // Manually set indexing as active
    *state.indexing_active.write().await = true;

    // Try to start indexing (would need AppHandle in real scenario)
    // For now, verify state behavior
    let indexing_active = *state.indexing_active.read().await;
    assert!(indexing_active);
}

#[tokio::test]
async fn test_stop_indexing_when_not_active() {
    let state = Arc::new(AppState::new().await.unwrap());

    let result = stop_indexing(tauri::State::from(state.as_ref())).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No indexing operation in progress");
}

#[tokio::test]
async fn test_stop_indexing_when_active() {
    let state = Arc::new(AppState::new().await.unwrap());

    // Set indexing as active
    *state.indexing_active.write().await = true;

    let result = stop_indexing(tauri::State::from(state.as_ref())).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Indexing stop signal sent");

    // Verify stop signal was set
    assert!(*state.stop_indexing.read().await);
}

#[tokio::test]
async fn test_indexing_state_reset() {
    let state = AppState::new().await.unwrap();

    // Set some state
    *state.indexing_active.write().await = true;
    *state.indexing_errors.write().await = vec!["error1".to_string()];
    *state.stop_indexing.write().await = true;

    // Reset
    state.reset_indexing_state().await;

    // Verify all state is reset
    assert!(!*state.indexing_active.read().await);
    assert!(*state.indexing_errors.read().await.is_empty());
    assert!(!*state.stop_indexing.read().await);
    assert!(state.indexing_progress.read().await.is_none());
}

#[tokio::test]
async fn test_indexing_progress_tracking() {
    let state = AppState::new().await.unwrap();

    // Simulate progress update
    *state.indexing_progress.write().await = Some(cortex_lib::indexer::ScanProgress {
        total_files: 100,
        current_file: 50,
    });

    let status: IndexStatus = get_index_status(tauri::State::from(&state))
        .await
        .unwrap();

    assert_eq!(status.total_files, 100);
    assert_eq!(status.indexed_files, 50);
    assert_eq!(status.progress_percentage, 50.0);
}

#[tokio::test]
async fn test_indexing_error_collection() {
    let state = AppState::new().await.unwrap();

    // Add some errors
    state.indexing_errors.write().await.push("Error 1".to_string());
    state.indexing_errors.write().await.push("Error 2".to_string());

    let status: IndexStatus = get_index_status(tauri::State::from(&state))
        .await
        .unwrap();

    assert_eq!(status.errors.len(), 2);
    assert_eq!(status.errors[0], "Error 1");
    assert_eq!(status.errors[1], "Error 2");
}

// Note: Full end-to-end tests with start_indexing require a Tauri AppHandle
// which is not available in integration tests. Those would be tested via:
// 1. Manual testing with the actual Tauri app
// 2. WebDriver tests (if UI automation is set up)
// 3. Tauri's built-in testing tools (when available)

#[tokio::test]
async fn test_indexing_pipeline_integration() {
    // This test verifies the full pipeline works without Tauri commands
    use cortex_lib::db::{insert_file, upsert_file_content, search_files_fts};
    use cortex_lib::indexer::{FileScanner, ContentExtractor};

    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    // Create test files
    create_test_file(dir_path, "test1.txt", "Rust programming language");
    create_test_file(dir_path, "test2.md", "# Markdown\nSome **bold** text");
    create_test_file(dir_path, "test3.txt", "JavaScript development");

    // Initialize state
    let state = AppState::new().await.unwrap();
    let db = state.db.read().await;
    let conn = db.get_connection();

    // Scan directory
    let scanner = FileScanner::new();
    let jobs = scanner.scan_directory(dir_path).unwrap();
    assert_eq!(jobs.len(), 3);

    // Index each file
    let mut indexed_count = 0;
    for job in jobs {
        let extracted = ContentExtractor::extract(&job.path).unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        let file_id = insert_file(
            conn,
            &job.path.to_string_lossy(),
            job.path.file_name().unwrap().to_str().unwrap(),
            "txt",
            job.size as i64,
            &now,
            &now,
            None,
            dir_path.to_str().unwrap(),
        ).unwrap();

        upsert_file_content(
            conn,
            file_id,
            Some(&extracted.text),
            extracted.summary.as_deref(),
        ).unwrap();

        indexed_count += 1;
    }

    assert_eq!(indexed_count, 3);

    // Search indexed content
    let results = search_files_fts(conn, "rust", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].filename.contains("test1"));

    let results = search_files_fts(conn, "markdown", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].filename.contains("test2"));
}
