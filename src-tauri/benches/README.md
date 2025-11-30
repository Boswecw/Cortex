# Cortex Performance Benchmarks

This directory contains comprehensive performance benchmarks for the Cortex file intelligence system.

## Available Benchmarks

### 1. Database Benchmark (`db_benchmark.rs`)
Tests core database operations and FTS5 search performance.

**What it tests:**
- File insertion rate (1000 files)
- Content indexing performance
- FTS5 search query speed
- Average search latency over 100 iterations

**Run with:**
```bash
cargo run --release --bin db_benchmark
```

**Expected Performance:**
- ✓ INSERT: >50 files/second
- ✓ SEARCH: <100ms per query

---

### 2. Indexing Benchmark (`indexing_benchmark.rs`)
Tests the complete indexing pipeline with real file operations.

**What it tests:**
- Directory scanning speed (100, 500, 1000 files)
- Content extraction performance (txt, md)
- Database insertion rate
- End-to-end pipeline throughput

**Run with:**
```bash
cargo run --release --bin indexing_benchmark
```

**Expected Performance:**
- ✓ SCAN: >500 files/second
- ✓ EXTRACT: >100 files/second
- ✓ INDEX: >50 files/second
- ✓ PIPELINE: >20 files/second (full end-to-end)

**Performance Targets:**
- Excellent: >100 files/sec
- Good: >50 files/sec
- Acceptable: >20 files/sec

**Estimates time for 10,000 files** based on test results.

---

### 3. Search Benchmark (`search_benchmark.rs`)
Tests search performance under various conditions with 1000 indexed documents.

**What it tests:**
- Simple single-word queries
- Multi-word complex queries
- Filtered searches (by file type, size)
- Pagination performance (limit/offset)
- Concurrent search throughput

**Run with:**
```bash
cargo run --release --bin search_benchmark
```

**Expected Performance:**
- ✓ SIMPLE QUERIES: <10ms average
- ✓ COMPLEX QUERIES: <50ms average
- ✓ FILTERED QUERIES: <100ms average
- ✓ THROUGHPUT: >100 searches/second

---

### 4. Load Test (`load_test.rs`)
Comprehensive load testing with large datasets (1000, 2500, 5000 files).

**What it tests:**
- Full pipeline with realistic file sizes and content
- Search performance degradation with dataset size
- Database statistics and storage requirements
- Memory usage estimation
- 10K file performance projection

**Run with:**
```bash
cargo run --release --bin load_test
```

**Performance Evaluation:**

**Indexing:**
- ✓ EXCELLENT: >100 files/sec
- ✓ GOOD: >50 files/sec
- ⚠ ACCEPTABLE: >20 files/sec

**Search:**
- ✓ EXCELLENT: <20ms average
- ✓ GOOD: <100ms average

**Target:** Index 10,000 files in <5 minutes

---

## Running All Benchmarks

Run all benchmarks sequentially:

```bash
cd src-tauri

# Run each benchmark
cargo run --release --bin db_benchmark
cargo run --release --bin indexing_benchmark
cargo run --release --bin search_benchmark
cargo run --release --bin load_test
```

**Note:** Each benchmark creates temporary databases and files, which are cleaned up automatically.

---

## Understanding Results

### Performance Tiers

**Indexing Speed:**
- 🟢 >100 files/sec - Excellent (10K files in <2 min)
- 🟡 >50 files/sec - Good (10K files in <4 min)
- 🟠 >20 files/sec - Acceptable (10K files in <9 min)
- 🔴 <20 files/sec - Needs optimization

**Search Latency:**
- 🟢 <10ms - Excellent (real-time feel)
- 🟡 <50ms - Good (very responsive)
- 🟠 <100ms - Acceptable (perceptible but fine)
- 🔴 >100ms - Needs optimization

### Key Metrics

**Throughput:** Files processed per second
- Affected by: CPU speed, disk I/O, file size

**Latency:** Time per operation (ms or µs)
- Affected by: Database size, query complexity, indexes

**Scalability:** Performance degradation with dataset size
- FTS5 should remain fast even with 10K+ files
- SQLite indexes provide O(log n) complexity

---

## Optimization Tips

### If Indexing is Slow (<20 files/sec):
1. Check disk I/O (SSD recommended)
2. Verify WAL mode is enabled
3. Increase SQLite cache size
4. Batch database operations
5. Profile with `flamegraph`

### If Search is Slow (>100ms):
1. Check FTS5 index integrity
2. Verify query syntax (avoid wildcards at start)
3. Add indexes on filtered columns
4. Optimize snippet generation parameters
5. Use query result caching

### If Memory Usage is High:
1. Stream large files instead of loading fully
2. Implement content size limits
3. Use database pooling efficiently
4. Clear unused file handles

---

## Continuous Performance Monitoring

### Recommended Practice:

Run benchmarks after major changes:
```bash
# Before changes
cargo run --release --bin load_test > before.txt

# After changes
cargo run --release --bin load_test > after.txt

# Compare results
diff before.txt after.txt
```

### CI/CD Integration:

Add to `.github/workflows/benchmark.yml`:
```yaml
- name: Run benchmarks
  run: |
    cd src-tauri
    cargo run --release --bin load_test
```

Set performance thresholds and fail CI if regression detected.

---

## Hardware Recommendations

**Minimum:**
- CPU: 2+ cores, 2.0GHz+
- RAM: 4GB
- Disk: Any (HDD acceptable)

**Recommended:**
- CPU: 4+ cores, 2.5GHz+
- RAM: 8GB+
- Disk: SSD (10x faster I/O)

**Optimal:**
- CPU: 8+ cores, 3.0GHz+
- RAM: 16GB+
- Disk: NVMe SSD
- Results: >200 files/sec indexing, <5ms search

---

## Troubleshooting

### Benchmark fails with "database locked"
- Ensure no other Cortex instance is running
- Check WAL mode is enabled
- Verify file permissions

### Inconsistent results between runs
- Normal variation ±10-20%
- Run multiple times and average
- Ensure no background processes

### Much slower than expected
- Check if running in debug mode (use `--release`)
- Verify system resources available
- Check antivirus isn't scanning files

---

## Contributing

When adding new benchmarks:
1. Follow existing naming convention
2. Include performance targets
3. Add cleanup code for temp files
4. Update this README

For performance improvements:
1. Run benchmarks before and after
2. Document the change and impact
3. Include results in PR description
