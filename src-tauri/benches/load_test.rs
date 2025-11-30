use cortex_lib::db::{get_db_stats, insert_file, search_files_fts, upsert_file_content, Database};
use cortex_lib::indexer::{ContentExtractor, FileScanner};
use std::fs;
use std::io::Write;
use std::time::Instant;
use tempfile::TempDir;

/// Generate realistic file content
fn generate_content(file_num: usize, content_type: usize) -> String {
    let base_content = match content_type {
        0 => {
            format!(
                "# Technical Documentation #{}\n\n\
                ## Overview\n\
                This document provides comprehensive information about system architecture, \
                design patterns, and implementation details.\n\n\
                ## Key Concepts\n\
                - Microservices architecture\n\
                - Event-driven design\n\
                - Scalability considerations\n\
                - Performance optimization\n\n\
                ## Implementation\n\
                The system uses modern technologies including Rust, TypeScript, and PostgreSQL. \
                {}",
                file_num,
                "Each component is designed for maximum efficiency and maintainability. ".repeat(30)
            )
        }
        1 => {
            format!(
                "# Project Specifications #{}\n\n\
                ## Requirements\n\
                Detailed specifications for feature implementation and integration.\n\n\
                ## User Stories\n\
                As a user, I want to be able to search through documents quickly and efficiently.\n\n\
                ## Acceptance Criteria\n\
                - Search results appear within 100ms\n\
                - Full-text indexing supports 10,000+ files\n\
                {}",
                file_num,
                "Additional requirements and specifications are documented here. ".repeat(30)
            )
        }
        2 => {
            format!(
                "Meeting Notes - Session #{}\n\n\
                Date: 2025-11-29\n\
                Attendees: Development Team\n\n\
                Discussion Topics:\n\
                - Performance optimization strategies\n\
                - Database indexing improvements\n\
                - Search algorithm enhancements\n\
                - User interface refinements\n\n\
                Action Items:\n\
                {}",
                file_num,
                "Follow up on implementation details and code review. ".repeat(30)
            )
        }
        _ => {
            format!(
                "Research Notes #{}\n\n\
                Topic: Full-Text Search Optimization\n\n\
                Key Findings:\n\
                SQLite FTS5 provides excellent performance for moderate datasets. \
                Porter stemming improves search relevance. \
                Snippet generation helps users identify relevant content.\n\n\
                References:\n\
                {}",
                file_num,
                "Additional research materials and citations are included. ".repeat(30)
            )
        }
    };

    base_content
}

#[tokio::main]
async fn main() {
    println!("\n=== Cortex Load Test ===\n");
    println!("Testing system performance with large datasets\n");

    // Test configuration
    let file_counts = vec![1000, 2500, 5000];

    for &num_files in &file_counts {
        println!("\n--- Load Test: {} files ---\n", num_files);

        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path();

        // Phase 1: File Creation
        println!("Phase 1: Creating {} test files...", num_files);
        let start = Instant::now();

        for i in 0..num_files {
            let content_type = i % 4;
            let content = generate_content(i, content_type);

            let extension = match i % 3 {
                0 => "md",
                1 => "txt",
                _ => "md",
            };

            let filename = format!("document_{:05}.{}", i, extension);
            let path = test_dir.join(&filename);

            let mut file = fs::File::create(&path).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        let creation_time = start.elapsed();
        println!(
            "  ✓ Created {} files in {:?} ({:.2} files/sec)\n",
            num_files,
            creation_time,
            num_files as f64 / creation_time.as_secs_f64()
        );

        // Phase 2: Full Indexing Pipeline
        println!("Phase 2: Running full indexing pipeline...");

        let db = Database::new().await.expect("Failed to create database");
        let conn = db.get_connection();

        let pipeline_start = Instant::now();

        // 2a: Scan
        let scan_start = Instant::now();
        let scanner = FileScanner::new();
        let jobs = scanner.scan_directory(test_dir).expect("Scan failed");
        let scan_time = scan_start.elapsed();

        println!("  Scan: {:?} ({:.2} files/sec)", scan_time, num_files as f64 / scan_time.as_secs_f64());

        // 2b: Extract
        let extract_start = Instant::now();
        let mut contents = Vec::with_capacity(jobs.len());

        for job in &jobs {
            let content = ContentExtractor::extract(&job.path).expect("Extract failed");
            contents.push(content);
        }

        let extract_time = extract_start.elapsed();
        println!("  Extract: {:?} ({:.2} files/sec)", extract_time, num_files as f64 / extract_time.as_secs_f64());

        // 2c: Index (Insert + Content)
        let index_start = Instant::now();

        for (job, content) in jobs.iter().zip(contents.iter()) {
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

            upsert_file_content(conn, file_id, Some(&content.text), content.summary.as_deref())
                .expect("Content insert failed");
        }

        let index_time = index_start.elapsed();
        println!("  Index: {:?} ({:.2} files/sec)", index_time, num_files as f64 / index_time.as_secs_f64());

        let total_pipeline_time = pipeline_start.elapsed();
        println!("\n  Total Pipeline: {:?}", total_pipeline_time);
        println!("  Overall Rate: {:.2} files/sec", num_files as f64 / total_pipeline_time.as_secs_f64());
        println!();

        // Phase 3: Search Performance Under Load
        println!("Phase 3: Search performance with {} indexed files...", num_files);

        let search_queries = vec![
            "documentation",
            "performance optimization",
            "user interface",
            "database indexing",
            "search algorithm",
            "microservices",
            "requirements",
            "meeting notes",
        ];

        let mut search_times = Vec::new();

        for query in &search_queries {
            let start = Instant::now();
            let results = search_files_fts(conn, query, 20).expect("Search failed");
            let duration = start.elapsed();

            search_times.push(duration.as_micros());

            println!(
                "  Query '{}': {} results in {}µs",
                query,
                results.len(),
                duration.as_micros()
            );
        }

        let avg_search_time = search_times.iter().sum::<u128>() / search_times.len() as u128;
        let max_search_time = *search_times.iter().max().unwrap();
        let min_search_time = *search_times.iter().min().unwrap();

        println!("\n  Search Statistics:");
        println!("    Average: {}µs ({:.2}ms)", avg_search_time, avg_search_time as f64 / 1000.0);
        println!("    Min: {}µs ({:.2}ms)", min_search_time, min_search_time as f64 / 1000.0);
        println!("    Max: {}µs ({:.2}ms)", max_search_time, max_search_time as f64 / 1000.0);
        println!();

        // Phase 4: Database Statistics
        println!("Phase 4: Database statistics...");

        let (total_files, indexed_files, total_size) = get_db_stats(conn).expect("Stats failed");

        println!("  Total files: {}", total_files);
        println!("  Indexed files: {}", indexed_files);
        println!("  Total size: {} bytes ({:.2} MB)", total_size, total_size as f64 / 1_048_576.0);
        println!();

        // Phase 5: Performance Evaluation
        println!("=== Performance Evaluation ===\n");

        let indexing_rate = num_files as f64 / total_pipeline_time.as_secs_f64();
        let search_avg_ms = avg_search_time as f64 / 1000.0;

        println!("Indexing Performance:");
        if indexing_rate > 100.0 {
            println!("  ✓ EXCELLENT: {:.2} files/sec", indexing_rate);
        } else if indexing_rate > 50.0 {
            println!("  ✓ GOOD: {:.2} files/sec", indexing_rate);
        } else if indexing_rate > 20.0 {
            println!("  ⚠ ACCEPTABLE: {:.2} files/sec", indexing_rate);
        } else {
            println!("  ✗ NEEDS IMPROVEMENT: {:.2} files/sec", indexing_rate);
        }

        println!("\nSearch Performance:");
        if search_avg_ms < 20.0 {
            println!("  ✓ EXCELLENT: {:.2}ms average", search_avg_ms);
        } else if search_avg_ms < 100.0 {
            println!("  ✓ GOOD: {:.2}ms average", search_avg_ms);
        } else {
            println!("  ⚠ NEEDS IMPROVEMENT: {:.2}ms average", search_avg_ms);
        }

        // Extrapolate to 10K files
        if num_files == 5000 {
            let estimated_10k = total_pipeline_time.as_secs_f64() * 2.0;
            println!("\nEstimated time for 10,000 files:");
            println!("  {:.2} seconds ({:.2} minutes)", estimated_10k, estimated_10k / 60.0);

            if estimated_10k < 300.0 {
                println!("  ✓ Meets <5 minute target");
            } else {
                println!("  ⚠ Exceeds 5 minute target");
            }
        }

        // Memory estimation (rough)
        let avg_file_size = total_size / num_files as i64;
        let estimated_10k_memory = (avg_file_size * 10000) as f64 / 1_048_576.0;

        println!("\nEstimated memory for 10K files:");
        println!("  {:.2} MB indexed content", estimated_10k_memory);

        println!("\n" + &"-".repeat(60) + "\n");
    }

    println!("=== Load Test Complete ===\n");
}
