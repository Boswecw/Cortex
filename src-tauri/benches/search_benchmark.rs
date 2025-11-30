use cortex_lib::db::{insert_file, search_files_fts, upsert_file_content, Database};
use rusqlite::{params, Connection};
use std::time::Instant;

/// Perform filtered search with dynamic SQL
fn search_with_filters(
    conn: &Connection,
    query: &str,
    file_type: Option<&str>,
    min_size: Option<i64>,
    limit: usize,
) -> Result<Vec<cortex_lib::db::SearchResult>, Box<dyn std::error::Error>> {
    let mut where_clauses = vec!["files_fts MATCH ?1", "f.is_deleted = 0"];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(query.to_string())];

    if let Some(ft) = file_type {
        where_clauses.push("f.file_type = ?");
        params_vec.push(Box::new(ft.to_string()));
    }

    if let Some(ms) = min_size {
        where_clauses.push("f.size >= ?");
        params_vec.push(Box::new(ms));
    }

    params_vec.push(Box::new(limit));

    let sql = format!(
        "SELECT f.id, f.path, f.filename,
                snippet(files_fts, 1, '<mark>', '</mark>', '...', 32) as snippet,
                rank
         FROM files_fts
         INNER JOIN files f ON files_fts.rowid = f.id
         WHERE {}
         ORDER BY rank
         LIMIT ?",
        where_clauses.join(" AND ")
    );

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;

    let results = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(cortex_lib::db::SearchResult {
                file_id: row.get(0)?,
                path: row.get(1)?,
                filename: row.get(2)?,
                snippet: row.get(3)?,
                score: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}

#[tokio::main]
async fn main() {
    println!("\n=== Cortex Search Performance Benchmark ===\n");

    // Initialize database with test data
    println!("Setting up test database...");
    let db = Database::new().await.expect("Failed to create database");
    let conn = db.get_connection();

    // Insert test data with variety
    let test_data = vec![
        ("rust", "Rust programming language tutorial", "md", 5000),
        ("rust", "Advanced Rust concepts and patterns", "txt", 8000),
        ("javascript", "JavaScript modern development guide", "md", 6000),
        ("javascript", "Node.js backend development", "txt", 7500),
        ("python", "Python data science with pandas", "md", 9000),
        ("python", "Django web framework tutorial", "txt", 8500),
        ("database", "SQL database design principles", "md", 10000),
        ("database", "NoSQL databases comparison", "txt", 7000),
        ("rust database", "Rust database drivers guide", "md", 6500),
        ("javascript database", "MongoDB with Node.js", "txt", 8000),
    ];

    for i in 0..1000 {
        let (keyword, title, ext, size) = &test_data[i % test_data.len()];

        let file_id = insert_file(
            conn,
            &format!("/test/doc_{}.{}", i, ext),
            &format!("doc_{}.{}", i, ext),
            ext,
            *size + i as i64,
            "2025-11-29T00:00:00Z",
            "2025-11-29T00:00:00Z",
            Some(&format!("hash_{}", i)),
            "/test",
        )
        .expect("Insert failed");

        let content = format!(
            "{} - Document number {}. This document contains information about {}. {}",
            title,
            i,
            keyword,
            "Additional searchable content to make documents more realistic. ".repeat(20)
        );

        upsert_file_content(conn, file_id, Some(&content), Some(title))
            .expect("Content insert failed");
    }

    println!("✓ Inserted 1000 test documents\n");

    // Benchmark 1: Simple Queries
    println!("1. Simple Query Benchmark");
    println!("   Testing basic FTS5 search performance\n");

    let simple_queries = vec![
        "rust",
        "javascript",
        "python",
        "database",
        "programming",
        "tutorial",
        "development",
        "framework",
    ];

    let mut total_time = 0u128;
    let mut total_results = 0;

    for query in &simple_queries {
        let start = Instant::now();
        let results = search_files_fts(conn, query, 20).expect("Search failed");
        let duration = start.elapsed();

        total_time += duration.as_micros();
        total_results += results.len();

        println!(
            "  Query '{}': {} results in {}µs",
            query,
            results.len(),
            duration.as_micros()
        );
    }

    let avg_time = total_time / simple_queries.len() as u128;
    println!(
        "\n  Average: {}µs ({:.2}ms)",
        avg_time,
        avg_time as f64 / 1000.0
    );
    println!("  Total results: {}", total_results);

    if avg_time < 10000 {
        println!("  ✓ EXCELLENT: <10ms average");
    } else if avg_time < 50000 {
        println!("  ✓ GOOD: <50ms average");
    } else {
        println!("  ⚠ SLOW: >50ms average");
    }
    println!();

    // Benchmark 2: Multi-word Queries
    println!("2. Multi-word Query Benchmark");
    println!("   Testing complex search queries\n");

    let complex_queries = vec![
        "rust programming",
        "javascript development",
        "python data science",
        "database design",
        "rust database",
        "javascript framework",
    ];

    let mut total_time = 0u128;

    for query in &complex_queries {
        let start = Instant::now();
        let results = search_files_fts(conn, query, 20).expect("Search failed");
        let duration = start.elapsed();

        total_time += duration.as_micros();

        println!(
            "  Query '{}': {} results in {}µs",
            query,
            results.len(),
            duration.as_micros()
        );
    }

    let avg_time = total_time / complex_queries.len() as u128;
    println!(
        "\n  Average: {}µs ({:.2}ms)",
        avg_time,
        avg_time as f64 / 1000.0
    );
    println!();

    // Benchmark 3: Filtered Searches
    println!("3. Filtered Search Benchmark");
    println!("   Testing searches with filters\n");

    let filter_tests = vec![
        ("rust", Some("md"), None),
        ("javascript", Some("txt"), None),
        ("database", None, Some(8000i64)),
        ("python", Some("md"), Some(5000i64)),
    ];

    let mut total_time = 0u128;

    for (query, file_type, min_size) in &filter_tests {
        let start = Instant::now();
        let results = search_with_filters(conn, query, *file_type, *min_size, 20)
            .expect("Filtered search failed");
        let duration = start.elapsed();

        total_time += duration.as_micros();

        println!(
            "  Query '{}' (type: {:?}, min_size: {:?}): {} results in {}µs",
            query,
            file_type,
            min_size,
            results.len(),
            duration.as_micros()
        );
    }

    let avg_time = total_time / filter_tests.len() as u128;
    println!(
        "\n  Average: {}µs ({:.2}ms)",
        avg_time,
        avg_time as f64 / 1000.0
    );
    println!();

    // Benchmark 4: Pagination Performance
    println!("4. Pagination Benchmark");
    println!("   Testing limit/offset performance\n");

    let query = "programming";
    let page_size = 20;
    let pages = 10;

    let mut total_time = 0u128;

    for page in 0..pages {
        let offset = page * page_size;

        let start = Instant::now();

        let sql = "SELECT f.id, f.path, f.filename,
                          snippet(files_fts, 1, '<mark>', '</mark>', '...', 32) as snippet,
                          rank
                   FROM files_fts
                   INNER JOIN files f ON files_fts.rowid = f.id
                   WHERE files_fts MATCH ? AND f.is_deleted = 0
                   ORDER BY rank
                   LIMIT ? OFFSET ?";

        let mut stmt = conn.prepare(sql).unwrap();
        let results: Vec<_> = stmt
            .query_map(params![query, page_size, offset], |row| {
                Ok(cortex_lib::db::SearchResult {
                    file_id: row.get(0)?,
                    path: row.get(1)?,
                    filename: row.get(2)?,
                    snippet: row.get(3)?,
                    score: row.get(4)?,
                })
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let duration = start.elapsed();
        total_time += duration.as_micros();

        println!(
            "  Page {} (offset {}): {} results in {}µs",
            page + 1,
            offset,
            results.len(),
            duration.as_micros()
        );
    }

    let avg_time = total_time / pages as u128;
    println!(
        "\n  Average per page: {}µs ({:.2}ms)",
        avg_time,
        avg_time as f64 / 1000.0
    );
    println!();

    // Benchmark 5: Concurrent Searches
    println!("5. Concurrent Search Benchmark");
    println!("   Testing 100 rapid searches\n");

    let start = Instant::now();

    for i in 0..100 {
        let query = &simple_queries[i % simple_queries.len()];
        let _ = search_files_fts(conn, query, 20);
    }

    let duration = start.elapsed();
    let avg_per_search = duration.as_micros() / 100;

    println!("  Total time: {:?}", duration);
    println!("  Average: {}µs ({:.2}ms)", avg_per_search, avg_per_search as f64 / 1000.0);
    println!("  Throughput: {:.2} searches/sec", 100.0 / duration.as_secs_f64());
    println!();

    // Performance Evaluation
    println!("=== Performance Evaluation ===\n");

    if avg_per_search < 5000 {
        println!("✓ EXCELLENT: Search performance <5ms average");
    } else if avg_per_search < 20000 {
        println!("✓ GOOD: Search performance <20ms average");
    } else if avg_per_search < 100000 {
        println!("⚠ ACCEPTABLE: Search performance <100ms average");
    } else {
        println!("✗ SLOW: Search performance >100ms average");
    }

    println!("\n=== Search Benchmark Complete ===\n");
}
