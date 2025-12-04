# Cortex Export Feature - Performance Testing Plan

**Date:** December 4, 2025
**Feature:** VS Code Claude Export & Rake Export
**Goal:** Validate performance with large codebases and optimize if needed

---

## 🎯 Testing Objectives

1. **Baseline Performance** - Measure current performance with real-world projects
2. **Scalability** - Verify the feature handles large codebases (1000+ files)
3. **Resource Usage** - Monitor memory consumption and CPU usage
4. **Bottlenecks** - Identify performance bottlenecks for optimization
5. **Optimization** - Implement improvements if needed

---

## 📊 Test Scenarios

### Scenario 1: Small Project (Baseline)
- **Dataset:** Cortex itself (~50-100 files)
- **Purpose:** Establish baseline metrics
- **Metrics:** Export time, memory usage, output size

### Scenario 2: Medium Project
- **Dataset:** Real-world project with 100-500 files
- **Purpose:** Typical use case validation
- **Metrics:** Export time, memory usage, output size

### Scenario 3: Large Project (Stress Test)
- **Dataset:** Synthetic or real project with 1000+ files
- **Purpose:** Scalability validation
- **Metrics:** Export time, memory usage, output size, potential failures

### Scenario 4: Very Large Files
- **Dataset:** Projects with files >1MB
- **Purpose:** Large file handling validation
- **Metrics:** Processing time per file, memory spikes

---

## 🔧 Performance Metrics to Measure

### Time Metrics
- Total export duration (seconds)
- Time per file (ms/file)
- Database query time
- File I/O time
- JSON serialization time

### Memory Metrics
- Peak memory usage (MB)
- Memory per file (KB/file)
- Memory growth rate

### Output Metrics
- Total output size (MB)
- Compression ratio (if applicable)
- Number of chunks generated

### Quality Metrics
- Export completeness (all files included?)
- Error rate (files that failed to process)
- Content accuracy (spot checks)

---

## 🛠️ Testing Tools

### Built-in Tools
- `cargo build --release` - Use optimized build
- `time` command - Basic timing
- `/usr/bin/time -v` - Detailed resource usage on Linux
- Rust's built-in benchmarking

### Optional Advanced Tools
- `hyperfine` - Statistical benchmarking
- `valgrind` - Memory profiling (if installed)
- `heaptrack` - Heap memory profiler (if installed)

---

## 📝 Test Execution Plan

### Phase 1: Baseline Testing (30 min)
1. Build release version: `cargo build --release`
2. Run Cortex export on itself
3. Measure time and memory
4. Document baseline metrics

### Phase 2: Synthetic Large Dataset (30 min)
1. Generate 1000+ test files with varying sizes
2. Run export on synthetic dataset
3. Measure scalability
4. Identify bottlenecks

### Phase 3: Analysis & Optimization (60 min)
1. Analyze performance data
2. Identify bottlenecks (if any)
3. Implement optimizations (if needed)
4. Re-test to verify improvements

### Phase 4: Documentation (30 min)
1. Document all findings
2. Create performance recommendations
3. Update user documentation

---

## ✅ Success Criteria

### Minimum Performance Requirements
- ✅ Export 100 files in < 10 seconds
- ✅ Export 1000 files in < 60 seconds
- ✅ Peak memory < 500MB for 1000 files
- ✅ Zero crashes or panics
- ✅ Error rate < 1%

### Optimal Performance Goals
- 🎯 Export 100 files in < 5 seconds
- 🎯 Export 1000 files in < 30 seconds
- 🎯 Peak memory < 200MB for 1000 files
- 🎯 Linear scaling with file count
- 🎯 Error rate < 0.1%

---

## 🚨 Known Potential Bottlenecks

Based on code review, potential bottlenecks:

1. **Database Operations**
   - Multiple queries per file
   - FTS indexing overhead
   - Transaction handling

2. **File I/O**
   - Reading large files into memory
   - Directory scanning
   - Text extraction (especially PDF, DOCX)

3. **Memory**
   - Loading all files into memory at once
   - Large embedding vectors (if enabled)
   - JSON serialization

4. **Content Extraction**
   - PDF parsing (pdf-extract crate)
   - DOCX parsing (docx-rs crate)
   - Markdown parsing

---

## 🔄 Optimization Strategies (if needed)

### If Time is the Bottleneck:
- Implement parallel processing (use rayon)
- Batch database operations
- Stream large files instead of loading fully
- Cache frequently accessed data

### If Memory is the Bottleneck:
- Process files in batches (e.g., 100 at a time)
- Stream JSON output instead of building in memory
- Release resources eagerly
- Use memory-mapped files for large files

### If I/O is the Bottleneck:
- Use async I/O (tokio already available)
- Implement read-ahead buffering
- Parallelize file reading
- Optimize file scanning (already using walkdir)

---

## 📋 Test Execution Checklist

- [ ] Build release version
- [ ] Test Scenario 1: Cortex itself (~50-100 files)
- [ ] Test Scenario 2: Medium project (100-500 files)
- [ ] Test Scenario 3: Large synthetic dataset (1000+ files)
- [ ] Test Scenario 4: Large file handling
- [ ] Measure all metrics
- [ ] Analyze results
- [ ] Implement optimizations (if needed)
- [ ] Re-test after optimizations
- [ ] Document findings
- [ ] Create performance recommendations

---

## 📊 Results Template

### Test Results

**Test Date:** [Date]
**Build:** Release (optimized)
**System:** [OS, CPU, RAM]

| Scenario | Files | Size | Time | Memory | Output | Notes |
|----------|-------|------|------|--------|--------|-------|
| Cortex | ~75 | ~2MB | ? | ? | ? | Baseline |
| Medium | 250 | ~10MB | ? | ? | ? | Real-world |
| Large | 1000 | ~50MB | ? | ? | ? | Stress test |
| XL Files | 10 | ~100MB | ? | ? | ? | Large files |

### Performance Analysis

**Bottlenecks Identified:**
- TBD

**Optimization Opportunities:**
- TBD

**Recommendations:**
- TBD

---

*This is a living document. Update as testing progresses.*
