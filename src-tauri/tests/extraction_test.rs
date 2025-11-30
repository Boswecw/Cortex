use cortex_lib::db::{Database, insert_file, upsert_file_content, search_files_fts};
use cortex_lib::indexer::{FileScanner, ContentExtractor};
use cortex_lib::error::Result;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test the full pipeline: scan → extract → index → search
#[tokio::test]
async fn test_full_extraction_pipeline() -> Result<()> {
    // Setup test environment
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    // Create test files
    create_test_file(dir_path, "doc1.txt", "Rust programming language tutorial");
    create_test_file(dir_path, "doc2.md", "# Markdown Guide\n\nLearn **markdown** syntax.");
    create_test_file(dir_path, "doc3.txt", "JavaScript modern web development");

    // Initialize database
    let db = Database::new().await?;
    let conn = db.get_connection();

    // Step 1: Scan directory
    let scanner = FileScanner::new();
    let jobs = scanner.scan_directory(dir_path)?;

    println!("Found {} files to index", jobs.len());
    assert_eq!(jobs.len(), 3);

    // Step 2: For each file, extract content and index
    for job in jobs {
        // Extract content
        let extracted = ContentExtractor::extract(&job.path)?;

        println!(
            "Extracted {} words from {}",
            extracted.word_count,
            job.path.display()
        );

        // Insert file metadata
        let file_id = insert_file(
            conn,
            &job.path.to_string_lossy(),
            job.path.file_name().unwrap().to_str().unwrap(),
            "txt",
            job.size as i64,
            "2025-11-29T00:00:00Z",
            "2025-11-29T00:00:00Z",
            None,
            dir_path.to_str().unwrap(),
        )?;

        // Insert content
        upsert_file_content(
            conn,
            file_id,
            Some(&extracted.text),
            extracted.summary.as_deref(),
        )?;
    }

    // Step 3: Search for content
    let rust_results = search_files_fts(conn, "rust", 10)?;
    println!("Search 'rust': {} results", rust_results.len());
    assert_eq!(rust_results.len(), 1);
    assert!(rust_results[0].filename.contains("doc1"));

    let markdown_results = search_files_fts(conn, "markdown", 10)?;
    println!("Search 'markdown': {} results", markdown_results.len());
    assert_eq!(markdown_results.len(), 1);
    assert!(markdown_results[0].filename.contains("doc2"));

    let js_results = search_files_fts(conn, "javascript", 10)?;
    println!("Search 'javascript': {} results", js_results.len());
    assert_eq!(js_results.len(), 1);
    assert!(js_results[0].filename.contains("doc3"));

    // Search across all documents
    let dev_results = search_files_fts(conn, "development", 10)?;
    println!("Search 'development': {} results", dev_results.len());
    assert_eq!(dev_results.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_markdown_extraction_integration() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    // Create markdown file with rich formatting
    let markdown_content = r#"# Project Documentation

## Overview
This is a **comprehensive** guide to our project.

### Features
- Fast performance
- Easy to use
- Well documented

```rust
fn main() {
    println!("Hello, world!");
}
```

## Getting Started
Use `cargo run` to start the application.
"#;

    create_test_file(dir_path, "README.md", markdown_content);

    // Scan and extract
    let scanner = FileScanner::new();
    let jobs = scanner.scan_directory(dir_path)?;
    assert_eq!(jobs.len(), 1);

    let extracted = ContentExtractor::extract(&jobs[0].path)?;

    // Verify markdown was converted to text
    assert!(extracted.text.contains("Project Documentation"));
    assert!(extracted.text.contains("comprehensive"));
    assert!(extracted.text.contains("Features"));
    assert!(extracted.text.contains("Fast performance"));
    assert!(extracted.text.contains("cargo run"));

    // Markdown formatting should be removed
    assert!(!extracted.text.contains("**"));
    assert!(!extracted.text.contains("##"));
    assert!(!extracted.text.contains("```"));

    println!("Extracted text:\n{}", extracted.text);
    println!("Word count: {}", extracted.word_count);

    Ok(())
}

#[tokio::test]
async fn test_text_extraction_with_encoding() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    // Create text file with UTF-8 content including special characters
    let content = "Hello! Café résumé naïve 你好 🦀";
    create_test_file(dir_path, "unicode.txt", content);

    let scanner = FileScanner::new();
    let jobs = scanner.scan_directory(dir_path)?;

    let extracted = ContentExtractor::extract(&jobs[0].path)?;

    // Should preserve unicode content
    assert!(extracted.text.contains("Café"));
    assert!(extracted.text.contains("你好"));
    assert!(extracted.text.contains("🦀"));

    Ok(())
}

#[test]
fn test_extraction_error_handling() {
    // Try to extract from non-existent file
    let result = ContentExtractor::extract(&PathBuf::from("/nonexistent/file.txt"));
    assert!(result.is_err());
}

fn create_test_file(dir: &std::path::Path, name: &str, content: &str) {
    let path = dir.join(name);
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}
