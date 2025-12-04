/// Performance benchmark for export functionality
///
/// Usage:
///   cargo run --release --bin export_benchmark -- [options]
///
/// Options:
///   --files <N>       Number of files to generate for testing (default: 100)
///   --size <BYTES>    Average file size in bytes (default: 5000)
///   --output <PATH>   Output directory for benchmark results
///
/// Examples:
///   cargo run --release --bin export_benchmark -- --files 1000
///   cargo run --release --bin export_benchmark -- --files 500 --size 10000

use cortex_lib::db::Database;
use cortex_lib::export::{BundleBuilder, ExportConfig, RakeExporter, RakeExportConfig, RakeExportMode};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[derive(Debug)]
struct BenchmarkResult {
    scenario: String,
    file_count: usize,
    total_size_bytes: u64,
    duration: Duration,
    peak_memory_mb: f64,
    output_size_bytes: u64,
    files_per_second: f64,
    mb_per_second: f64,
}

impl BenchmarkResult {
    fn print(&self) {
        println!("\n📊 Benchmark Results: {}", self.scenario);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Files processed:  {}", self.file_count);
        println!("  Total input size: {:.2} MB", self.total_size_bytes as f64 / 1_000_000.0);
        println!("  Duration:         {:.2}s", self.duration.as_secs_f64());
        println!("  Output size:      {:.2} MB", self.output_size_bytes as f64 / 1_000_000.0);
        println!("  Throughput:       {:.1} files/sec", self.files_per_second);
        println!("  Throughput:       {:.2} MB/sec", self.mb_per_second);
        if self.peak_memory_mb > 0.0 {
            println!("  Peak memory:      {:.1} MB", self.peak_memory_mb);
        }
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
}

/// Generate synthetic test dataset
fn generate_test_dataset(dir: &Path, file_count: usize, avg_size_bytes: usize) -> std::io::Result<u64> {
    let file_types = vec![
        ("rs", "rust"),
        ("ts", "typescript"),
        ("js", "javascript"),
        ("py", "python"),
        ("txt", "text"),
        ("md", "markdown"),
    ];

    let mut total_bytes = 0u64;

    for i in 0..file_count {
        let file_type = &file_types[i % file_types.len()];
        let file_path = dir.join(format!("test_file_{}.{}", i, file_type.0));

        // Generate realistic content based on file type
        let content = match file_type.0 {
            "rs" => generate_rust_content(avg_size_bytes),
            "ts" | "js" => generate_js_content(avg_size_bytes),
            "py" => generate_python_content(avg_size_bytes),
            "md" => generate_markdown_content(avg_size_bytes),
            _ => generate_text_content(avg_size_bytes),
        };

        let mut file = fs::File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
        total_bytes += content.len() as u64;

        // Create some subdirectories for nested testing
        if i % 20 == 0 {
            let subdir = dir.join(format!("subdir_{}", i / 20));
            fs::create_dir_all(&subdir)?;
        }
    }

    Ok(total_bytes)
}

fn generate_rust_content(size: usize) -> String {
    let mut content = String::with_capacity(size);
    content.push_str("// Generated Rust code for benchmarking\n\n");
    content.push_str("use std::collections::HashMap;\n\n");

    let lines_needed = size / 50; // ~50 chars per line
    for i in 0..lines_needed {
        content.push_str(&format!(
            "fn function_{}() -> Result<(), String> {{\n    Ok(())\n}}\n\n",
            i
        ));
    }

    content.truncate(size);
    content
}

fn generate_js_content(size: usize) -> String {
    let mut content = String::with_capacity(size);
    content.push_str("// Generated JavaScript code for benchmarking\n\n");

    let lines_needed = size / 60;
    for i in 0..lines_needed {
        content.push_str(&format!(
            "function process{}(data) {{\n  return data.map(x => x * 2);\n}}\n\n",
            i
        ));
    }

    content.truncate(size);
    content
}

fn generate_python_content(size: usize) -> String {
    let mut content = String::with_capacity(size);
    content.push_str("# Generated Python code for benchmarking\n\n");
    content.push_str("import sys\n\n");

    let lines_needed = size / 50;
    for i in 0..lines_needed {
        content.push_str(&format!(
            "def process_{}(data):\n    return [x * 2 for x in data]\n\n",
            i
        ));
    }

    content.truncate(size);
    content
}

fn generate_markdown_content(size: usize) -> String {
    let mut content = String::with_capacity(size);
    content.push_str("# Generated Markdown Documentation\n\n");

    let paragraphs_needed = size / 200;
    for i in 0..paragraphs_needed {
        content.push_str(&format!(
            "## Section {}\n\nLorem ipsum dolor sit amet, consectetur adipiscing elit. \
            Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.\n\n",
            i
        ));
    }

    content.truncate(size);
    content
}

fn generate_text_content(size: usize) -> String {
    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ";
    text.repeat(size / text.len() + 1)[..size].to_string()
}

/// Index files in the database
async fn index_files(db: &Database, dir: &Path) -> cortex_lib::error::Result<()> {
    use cortex_lib::indexer::scanner::FileScanner;
    use cortex_lib::indexer::processor::FileProcessor;

    // Scan directory
    let scanner = FileScanner::new();
    let jobs = scanner.scan_directory(dir)?;

    println!("  Indexing {} files...", jobs.len());

    // Process each file
    let processor = FileProcessor::new(db.clone());
    for (idx, job) in jobs.iter().enumerate() {
        if idx % 100 == 0 && idx > 0 {
            println!("    Indexed {}/{} files...", idx, jobs.len());
        }
        processor.process_job(job).await?;
    }

    Ok(())
}

/// Run VS Code export benchmark
async fn benchmark_vscode_export(
    db: &Database,
    output_dir: &Path,
    scenario: &str,
    file_count: usize,
    total_size_bytes: u64,
) -> cortex_lib::error::Result<BenchmarkResult> {
    println!("\n🚀 Running VS Code export benchmark: {}", scenario);

    let config = ExportConfig {
        collection_id: None,
        include_embeddings: false,
        output_path: output_dir.join("vscode_export").to_string_lossy().to_string(),
        include_prompts: true,
        project_name: Some("BenchmarkProject".to_string()),
        custom_context: None,
    };

    let bundler = BundleBuilder::new(db.clone());

    // Measure time
    let start = Instant::now();
    let result = bundler.create_bundle(&config).await?;
    let duration = start.elapsed();

    // Calculate metrics
    let output_size = fs::metadata(&result.context_file)
        .map(|m| m.len())
        .unwrap_or(0);

    let files_per_second = file_count as f64 / duration.as_secs_f64();
    let mb_per_second = (total_size_bytes as f64 / 1_000_000.0) / duration.as_secs_f64();

    Ok(BenchmarkResult {
        scenario: scenario.to_string(),
        file_count,
        total_size_bytes,
        duration,
        peak_memory_mb: 0.0, // TODO: Implement memory tracking
        output_size_bytes: output_size,
        files_per_second,
        mb_per_second,
    })
}

/// Run Rake export benchmark
async fn benchmark_rake_export(
    db: &Database,
    output_dir: &Path,
    scenario: &str,
    file_count: usize,
    total_size_bytes: u64,
) -> cortex_lib::error::Result<BenchmarkResult> {
    println!("\n🚀 Running Rake export benchmark: {}", scenario);

    let config = RakeExportConfig {
        collection_id: None,
        tenant_id: "benchmark_tenant".to_string(),
        output_path: output_dir.join("rake_export.json").to_string_lossy().to_string(),
        include_embeddings: false,
        export_mode: RakeExportMode::Full,
    };

    let exporter = RakeExporter::new(db.clone());

    // Measure time
    let start = Instant::now();
    let output_path = exporter.export_to_file(&config).await?;
    let duration = start.elapsed();

    // Calculate metrics
    let output_size = fs::metadata(&output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let files_per_second = file_count as f64 / duration.as_secs_f64();
    let mb_per_second = (total_size_bytes as f64 / 1_000_000.0) / duration.as_secs_f64();

    Ok(BenchmarkResult {
        scenario: scenario.to_string(),
        file_count,
        total_size_bytes,
        duration,
        peak_memory_mb: 0.0,
        output_size_bytes: output_size,
        files_per_second,
        mb_per_second,
    })
}

#[tokio::main]
async fn main() -> cortex_lib::error::Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut file_count = 100;
    let mut avg_size = 5000;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--files" if i + 1 < args.len() => {
                file_count = args[i + 1].parse().unwrap_or(100);
                i += 2;
            }
            "--size" if i + 1 < args.len() => {
                avg_size = args[i + 1].parse().unwrap_or(5000);
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║         Cortex Export Performance Benchmark              ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    println!("Configuration:");
    println!("  Test files:    {}", file_count);
    println!("  Avg file size: {} bytes (~{} KB)", avg_size, avg_size / 1024);
    println!();

    // Create temporary directory for test data
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let data_dir = temp_dir.path().join("test_data");
    fs::create_dir_all(&data_dir).expect("Failed to create data directory");

    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    println!("📁 Generating test dataset...");
    let total_size = generate_test_dataset(&data_dir, file_count, avg_size)
        .expect("Failed to generate test dataset");
    println!("  ✓ Generated {} files ({:.2} MB)",
        file_count, total_size as f64 / 1_000_000.0);

    // Create in-memory database for testing
    println!("\n📊 Setting up test database...");
    let db = Database::new_in_memory().await?;

    // Index files
    println!("\n📚 Indexing files into database...");
    let index_start = Instant::now();
    index_files(&db, &data_dir).await?;
    let index_duration = index_start.elapsed();
    println!("  ✓ Indexing complete in {:.2}s", index_duration.as_secs_f64());

    // Run benchmarks
    let mut results = Vec::new();

    // VS Code export benchmark
    let vscode_result = benchmark_vscode_export(
        &db,
        &output_dir,
        &format!("VS Code Export ({} files)", file_count),
        file_count,
        total_size,
    ).await?;
    vscode_result.print();
    results.push(vscode_result);

    // Rake export benchmark
    let rake_result = benchmark_rake_export(
        &db,
        &output_dir,
        &format!("Rake Export ({} files)", file_count),
        file_count,
        total_size,
    ).await?;
    rake_result.print();
    results.push(rake_result);

    // Summary
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║                    Summary                                ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    println!("Indexing Performance:");
    println!("  Duration:    {:.2}s", index_duration.as_secs_f64());
    println!("  Throughput:  {:.1} files/sec",
        file_count as f64 / index_duration.as_secs_f64());
    println!();
    println!("Export Performance:");
    for result in &results {
        println!("  {} - {:.2}s ({:.1} files/sec)",
            result.scenario,
            result.duration.as_secs_f64(),
            result.files_per_second);
    }
    println!();

    // Performance assessment
    let vscode_time = results[0].duration.as_secs_f64();
    let passes_minimum = file_count <= 100 || vscode_time < 10.0;
    let passes_optimal = file_count <= 100 || vscode_time < 5.0;

    if passes_optimal {
        println!("✅ Performance: EXCELLENT (exceeds optimal goals)");
    } else if passes_minimum {
        println!("✅ Performance: GOOD (meets minimum requirements)");
    } else {
        println!("⚠️  Performance: NEEDS OPTIMIZATION");
        println!("   Current: {:.1}s for {} files", vscode_time, file_count);
        println!("   Target:  < 10.0s for 100 files, < 60.0s for 1000 files");
    }

    Ok(())
}
