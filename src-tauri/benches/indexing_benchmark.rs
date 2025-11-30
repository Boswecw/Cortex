use cortex_lib::db::{insert_file, upsert_file_content, Database};
use cortex_lib::indexer::{ContentExtractor, FileScanner};
use std::fs;
use std::io::Write;
use std::time::Instant;
use tempfile::TempDir;

/// Create test files for benchmarking
fn create_test_files(dir: &std::path::Path, count: usize) {
    println!("Creating {} test files...", count);

    for i in 0..count {
        let content = match i % 4 {
            0 => format!(
                "# Rust Programming Tutorial {}\n\nThis is a comprehensive guide to Rust programming language. \
                It covers ownership, borrowing, lifetimes, and advanced concepts. {}",
                i,
                "The quick brown fox jumps over the lazy dog. ".repeat(50)
            ),
            1 => format!(
                "JavaScript Development Guide {}\n\nModern JavaScript with ES6+ features. \
                Covers async/await, promises, modules, and more. {}",
                i,
                "Lorem ipsum dolor sit amet consectetur adipiscing elit. ".repeat(50)
            ),
            2 => format!(
                "Database Design Principles {}\n\nSQL, NoSQL, indexing, normalization, and performance tuning. {}",
                i,
                "The five boxing wizards jump quickly. ".repeat(50)
            ),
            _ => format!(
                "Python Data Science {}\n\nPandas, NumPy, SciPy, machine learning with scikit-learn. {}",
                i,
                "Pack my box with five dozen liquor jugs. ".repeat(50)
            ),
        };

        let extension = match i % 4 {
            0 => "md",
            1 => "txt",
            2 => "md",
            _ => "txt",
        };

        let filename = format!("doc_{}.{}", i, extension);
        let path = dir.join(&filename);

        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    println!("✓ Created {} test files", count);
}

#[tokio::main]
async fn main() {
    println!("\n=== Cortex Indexing Performance Benchmark ===\n");

    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path();

    // Test configurations
    let test_sizes = vec![100, 500, 1000];

    for &file_count in &test_sizes {
        println!("\n--- Benchmarking with {} files ---\n", file_count);

        // Create test files
        create_test_files(test_dir, file_count);

        // Initialize fresh database
        let db = Database::new().await.expect("Failed to create database");
        let conn = db.get_connection();

        // Benchmark 1: Directory Scanning
        println!("1. Directory Scanning Benchmark");
        let start = Instant::now();
        let scanner = FileScanner::new();
        let jobs = scanner.scan_directory(test_dir).expect("Scan failed");
        let scan_duration = start.elapsed();

        println!("  ✓ Scanned {} files in {:?}", jobs.len(), scan_duration);
        println!("  Rate: {:.2} files/sec", file_count as f64 / scan_duration.as_secs_f64());
        println!("  Per file: {:.2}ms", scan_duration.as_millis() as f64 / file_count as f64);
        println!();

        // Benchmark 2: Content Extraction
        println!("2. Content Extraction Benchmark");
        let start = Instant::now();
        let mut extracted_contents = Vec::new();

        for job in &jobs {
            let content = ContentExtractor::extract(&job.path).expect("Extraction failed");
            extracted_contents.push(content);
        }

        let extract_duration = start.elapsed();

        println!("  ✓ Extracted {} files in {:?}", jobs.len(), extract_duration);
        println!("  Rate: {:.2} files/sec", file_count as f64 / extract_duration.as_secs_f64());
        println!("  Per file: {:.2}ms", extract_duration.as_millis() as f64 / file_count as f64);

        // Calculate total words
        let total_words: usize = extracted_contents.iter().map(|c| c.word_count).sum();
        println!("  Total words extracted: {}", total_words);
        println!();

        // Benchmark 3: Database Insertion
        println!("3. Database Insertion Benchmark");
        let start = Instant::now();

        for (job, content) in jobs.iter().zip(extracted_contents.iter()) {
            let file_id = insert_file(
                conn,
                &job.path.to_string_lossy(),
                job.path.file_name().unwrap().to_str().unwrap(),
                job.path.extension().and_then(|e| e.to_str()).unwrap_or("txt"),
                job.size as i64,
                "2025-11-29T00:00:00Z",
                "2025-11-29T00:00:00Z",
                None,
                test_dir.to_str().unwrap(),
            )
            .expect("Insert failed");

            upsert_file_content(
                conn,
                file_id,
                Some(&content.text),
                content.summary.as_deref(),
            )
            .expect("Content insert failed");
        }

        let insert_duration = start.elapsed();

        println!("  ✓ Inserted {} files in {:?}", jobs.len(), insert_duration);
        println!("  Rate: {:.2} files/sec", file_count as f64 / insert_duration.as_secs_f64());
        println!("  Per file: {:.2}ms", insert_duration.as_millis() as f64 / file_count as f64);
        println!();

        // Benchmark 4: Full Pipeline
        println!("4. Full Pipeline Benchmark (Scan → Extract → Index)");
        let total_duration = scan_duration + extract_duration + insert_duration;

        println!("  Total time: {:?}", total_duration);
        println!("  Overall rate: {:.2} files/sec", file_count as f64 / total_duration.as_secs_f64());
        println!("  Per file: {:.2}ms", total_duration.as_millis() as f64 / file_count as f64);
        println!();

        // Performance Targets
        println!("Performance Evaluation:");
        let files_per_sec = file_count as f64 / total_duration.as_secs_f64();

        if files_per_sec > 100.0 {
            println!("  ✓ EXCELLENT: {:.2} files/sec (target: >100)", files_per_sec);
        } else if files_per_sec > 50.0 {
            println!("  ✓ GOOD: {:.2} files/sec (target: >50)", files_per_sec);
        } else if files_per_sec > 20.0 {
            println!("  ⚠ ACCEPTABLE: {:.2} files/sec (target: >20)", files_per_sec);
        } else {
            println!("  ✗ SLOW: {:.2} files/sec (needs optimization)", files_per_sec);
        }

        // Estimate time for 10K files
        let estimated_10k = total_duration.as_secs_f64() * (10000.0 / file_count as f64);
        println!("  Estimated time for 10K files: {:.2} seconds ({:.2} minutes)",
            estimated_10k,
            estimated_10k / 60.0
        );

        if estimated_10k < 300.0 {
            println!("  ✓ Meets 10K files in <5min target");
        } else {
            println!("  ✗ Exceeds 5min target for 10K files");
        }

        // Clean up for next test
        for job in &jobs {
            let _ = fs::remove_file(&job.path);
        }
    }

    println!("\n=== Indexing Benchmark Complete ===\n");
}
