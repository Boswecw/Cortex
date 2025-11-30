use cortex_lib::db::{Database, insert_file, upsert_file_content, search_files_fts};
use std::time::Instant;

#[tokio::main]
async fn main() {
    println!("=== Cortex Database Benchmark ===\n");

    let db = Database::new().await.expect("Failed to create database");
    let conn = db.get_connection();

    // Benchmark: Insert 1000 files
    println!("Benchmarking file insertion...");
    let start = Instant::now();

    for i in 0..1000 {
        let file_id = insert_file(
            conn,
            &format!("/bench/file_{}.txt", i),
            &format!("file_{}.txt", i),
            "txt",
            1024 + i as i64,
            "2025-11-29T00:00:00Z",
            "2025-11-29T00:00:00Z",
            Some(&format!("hash_{}", i)),
            "/bench",
        ).expect("Failed to insert file");

        if i % 2 == 0 {
            // Index half of the files
            upsert_file_content(
                conn,
                file_id,
                Some(&format!("This is content for file number {} with searchable keywords rust programming database", i)),
                Some(&format!("Summary for file {}", i)),
            ).expect("Failed to insert content");
        }
    }

    let duration = start.elapsed();
    let files_per_sec = 1000.0 / duration.as_secs_f64();

    println!("  Inserted 1000 files in {:?}", duration);
    println!("  Rate: {:.2} files/second", files_per_sec);
    println!("  ✓ Target: >50 files/sec - {}", if files_per_sec > 50.0 { "PASS" } else { "FAIL" });
    println!();

    // Benchmark: Search queries
    println!("Benchmarking search queries...");

    let queries = vec![
        "rust",
        "programming",
        "database",
        "rust programming",
        "searchable keywords",
    ];

    for query in &queries {
        let start = Instant::now();
        let results = search_files_fts(conn, query, 20).expect("Search failed");
        let duration = start.elapsed();

        println!("  Query '{}': {} results in {:?} ({} ms)",
            query,
            results.len(),
            duration,
            duration.as_millis()
        );

        if duration.as_millis() > 100 {
            println!("    ⚠ Warning: Search exceeded 100ms target");
        }
    }

    // Average search time
    println!("\nRunning 100 search iterations for average...");
    let start = Instant::now();

    for _ in 0..100 {
        let _ = search_files_fts(conn, "rust programming", 20);
    }

    let avg_duration = start.elapsed() / 100;
    println!("  Average search time: {:?} ({} ms)", avg_duration, avg_duration.as_millis());
    println!("  ✓ Target: <100ms - {}", if avg_duration.as_millis() < 100 { "PASS" } else { "FAIL" });

    println!("\n=== Benchmark Complete ===");
}
