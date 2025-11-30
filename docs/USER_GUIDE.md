# Cortex User Guide

**Version:** 0.1.0 (Phase 0)
**Last Updated:** 2025-11-29

---

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Indexing Files](#indexing-files)
5. [Searching](#searching)
6. [Advanced Features](#advanced-features)
7. [Troubleshooting](#troubleshooting)
8. [FAQ](#faq)
9. [Performance Tips](#performance-tips)

---

## Introduction

### What is Cortex?

Cortex is a **fast, offline-first file indexing and search application** built with Tauri, Rust, and SvelteKit. It allows you to:

- **Index** thousands of files from your local filesystem
- **Search** content using powerful full-text search (FTS5)
- **Filter** results by file type, size, and date
- **View** file previews and full content
- **Track** indexing progress in real-time

### Key Features

- ✅ **Offline-first** - All data stored locally in SQLite
- ✅ **Fast search** - FTS5 with highlighted snippets (<100ms queries)
- ✅ **Multiple formats** - .txt, .md, .pdf, .docx, source code
- ✅ **Advanced filters** - Type, size, date range
- ✅ **Real-time progress** - Live indexing status updates
- ✅ **Privacy-focused** - No cloud, no tracking

### System Requirements

**Minimum:**
- OS: Linux, macOS, Windows
- RAM: 8GB
- Storage: 100MB + index size (varies)
- CPU: 2 cores

**Recommended:**
- RAM: 16GB
- Storage: SSD with 500MB+ free
- CPU: 4+ cores

---

## Installation

### Prerequisites

#### Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  libgtk-3-dev
```

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

#### Windows
No additional prerequisites required.

### Building from Source

```bash
# Clone repository
git clone https://github.com/yourusername/cortex.git
cd cortex

# Install frontend dependencies
npm install

# Build backend
cd src-tauri
cargo build --release

# Run application
cd ..
npm run tauri dev
```

### Installing Pre-built Binary

Download the latest release for your platform:
- [Linux (.deb)](https://github.com/yourusername/cortex/releases)
- [macOS (.dmg)](https://github.com/yourusername/cortex/releases)
- [Windows (.msi)](https://github.com/yourusername/cortex/releases)

Install and launch the application.

---

## Quick Start

### First Launch

1. **Launch Cortex** from your applications menu
2. The database will be created at `~/.cortex/db.sqlite`
3. You'll see the main search interface

### Index Your First Directory

1. Click **"Start Indexing"** or use the indexing panel
2. Select directories to index (e.g., `~/Documents`, `~/Projects`)
3. Click **"Begin Indexing"**
4. Watch real-time progress:
   - Files scanned
   - Files indexed
   - Current file being processed
   - Progress percentage

### Search Your Files

1. Enter a search query in the search bar (e.g., "rust programming")
2. Results appear instantly with:
   - Filename and path
   - Highlighted snippet showing match context
   - File type and size
   - Last modified date
3. Click a result to view full file details

---

## Indexing Files

### Supported File Types

Cortex can index the following file types:

**Documents:**
- `.txt` - Plain text
- `.md` - Markdown
- `.pdf` - PDF documents
- `.docx` - Microsoft Word

**Source Code:**
- `.rs` - Rust
- `.js` / `.ts` - JavaScript/TypeScript
- `.jsx` / `.tsx` - React
- `.py` - Python
- `.java` - Java
- `.cpp` / `.c` / `.h` - C/C++

**Configuration:**
- `.json` - JSON
- `.yaml` / `.yml` - YAML
- `.toml` - TOML
- `.xml` - XML

### Indexing Process

The indexing pipeline consists of 3 stages:

1. **Scanning** - Recursively finds all supported files
2. **Extraction** - Extracts text content from each file
3. **Indexing** - Stores content in searchable database

#### What Gets Indexed

- File metadata (path, size, dates)
- Full text content
- Word count and summary
- File type and hash

#### What Gets Skipped

- Hidden files (starting with `.`)
- System directories (`node_modules`, `.git`, `target`)
- Binary files without text content
- Files larger than 100MB (configurable)
- Symbolic links (optional)

### Starting an Index

**Via UI:**
1. Click **"Index"** in the sidebar
2. Click **"Add Directory"**
3. Browse to select directories
4. Click **"Start Indexing"**

**Multiple Directories:**
You can index multiple directories in one session:
```
~/Documents
~/Projects/work
~/Downloads
```

### Monitoring Progress

**Real-time Updates:**
- Total files found
- Files indexed so far
- Current file being processed
- Overall progress percentage
- Estimated time remaining

**Progress Events:**
The UI receives updates every 10 files via the `indexing:progress` event.

### Stopping an Index

Click **"Stop Indexing"** to gracefully cancel:
- Current file finishes processing
- All indexed files remain searchable
- Can resume later from same or different directories

---

## Searching

### Basic Search

Enter any text in the search bar:
```
rust programming
```

**Results include:**
- Files containing "rust" OR "programming"
- Ranked by relevance (FTS5 algorithm)
- Highlighted snippets with `<mark>` tags
- Up to 50 results by default

### Search Operators

**Phrase Search:**
```
"exact phrase"
```
Finds files containing the exact phrase.

**Prefix Search:**
```
program*
```
Matches "programming", "programmer", "programs", etc.

**Boolean NOT:**
```
rust NOT javascript
```
Finds files with "rust" but not "javascript".

### Advanced Filtering

**Filter by File Type:**
```
Query: tutorial
Filter: Type = "md"
```
Only searches Markdown files.

**Filter by Size:**
```
Query: documentation
Filter: Size >= 10KB, Size <= 1MB
```
Only searches files between 10KB and 1MB.

**Filter by Date:**
```
Query: report
Filter: Modified after 2025-01-01
```
Only searches files modified since January 1, 2025.

**Combined Filters:**
```
Query: API
Filters:
  - Type: "js"
  - Size: >= 1KB
  - Modified: >= 2025-11-01
```

### Pagination

**Page through results:**
- Default: 50 results per page
- Maximum: 1000 results per page

Navigate with **Next** and **Previous** buttons.

### Viewing Results

**Result Card Shows:**
- Filename
- File path (clickable)
- Snippet with highlighted match
- File type icon
- Size and last modified

**Click to View Details:**
- Full file metadata
- Content preview (500 characters)
- Option to load full content
- Word count and summary
- Open in external editor

---

## Advanced Features

### Statistics Dashboard

View indexing statistics:
- **Total files:** Number of files tracked
- **Indexed files:** Files with searchable content
- **Total size:** Combined size of all files
- **Indexing progress:** Percentage indexed

### Performance Monitoring

**Query Timing:**
Each search displays:
```
Found 42 results in 12ms
```

**Indexing Rate:**
Real-time display of files/second during indexing.

### File Watching (Future)

Automatic re-indexing when files change:
- Detects file modifications
- Indexes new files
- Removes deleted files
- Updates changed content

*Note: File watching is planned for Phase 1.*

### Incremental Indexing (Future)

Re-index only changed files:
- Faster than full re-index
- Preserves existing data
- Detects file moves/renames

*Note: Incremental indexing is planned for Phase 1.*

---

## Troubleshooting

### Indexing Issues

#### Problem: "Indexing is very slow"

**Solutions:**
1. Check disk speed (SSD recommended)
2. Close other applications
3. Check for large PDF files (they extract slowly)
4. Reduce indexed directories

**Performance Targets:**
- Good: >50 files/second
- Acceptable: >20 files/second
- Slow: <20 files/second

#### Problem: "Some files are not indexed"

**Possible Causes:**
1. File type not supported (check extension)
2. File is hidden (starts with `.`)
3. File in excluded directory (`node_modules`, `.git`)
4. File larger than 100MB
5. File is a symbolic link

**Solution:**
Check the indexing errors in the progress panel.

#### Problem: "Indexing stops unexpectedly"

**Possible Causes:**
1. Disk full
2. Database corruption
3. Permission errors

**Solution:**
1. Check available disk space
2. Check database integrity:
   ```bash
   sqlite3 ~/.cortex/db.sqlite "PRAGMA integrity_check;"
   ```
3. Check log file for errors

### Search Issues

#### Problem: "No results found"

**Solutions:**
1. Check if files are indexed (Statistics panel)
2. Try simpler search terms
3. Check filter settings (may be too restrictive)
4. Verify file type is supported

#### Problem: "Search is slow (>100ms)"

**Solutions:**
1. Simplify search query
2. Add filters to narrow results
3. Check database size (VACUUM if needed)
4. Restart application

**Database Optimization:**
```bash
sqlite3 ~/.cortex/db.sqlite "VACUUM;"
```

#### Problem: "Results don't match query"

**Explanation:**
FTS5 uses Porter stemming:
- "running" matches "run"
- "databases" matches "database"

This is expected behavior for better search.

### Application Issues

#### Problem: "Application won't start"

**Solutions:**
1. Check system dependencies installed
2. Check database permissions
3. Check log file:
   ```bash
   tail -f ~/.cortex/cortex.log
   ```
4. Try rebuilding:
   ```bash
   cargo clean && cargo build --release
   ```

#### Problem: "High memory usage"

**Solutions:**
1. Close application and restart
2. Reduce batch size (for developers)
3. Run VACUUM on database
4. Check for memory leaks (report issue)

### Database Issues

#### Problem: "Database is locked"

**Cause:**
Another process is accessing the database.

**Solution:**
```bash
# Check for other Cortex instances
ps aux | grep cortex

# If safe, remove lock
rm ~/.cortex/db.sqlite-wal
rm ~/.cortex/db.sqlite-shm
```

#### Problem: "Database corruption"

**Symptoms:**
- "database disk image is malformed"
- Application crashes on startup

**Recovery:**
```bash
# Backup current database
cp ~/.cortex/db.sqlite ~/.cortex/db.sqlite.backup

# Try recovery
sqlite3 ~/.cortex/db.sqlite "PRAGMA integrity_check;"
sqlite3 ~/.cortex/db.sqlite ".recover" | sqlite3 ~/.cortex/db_recovered.sqlite
mv ~/.cortex/db_recovered.sqlite ~/.cortex/db.sqlite
```

**Last Resort:**
```bash
# Delete database and re-index
rm ~/.cortex/db.sqlite
# Restart Cortex and re-index directories
```

---

## FAQ

### General

**Q: Is Cortex free and open source?**
A: Yes! Cortex is licensed under [LICENSE] and contributions are welcome.

**Q: Does Cortex send data to the cloud?**
A: No. All indexing and search happens locally on your machine. No telemetry or tracking.

**Q: How much disk space does the index use?**
A: Approximately 30-50% of the original file sizes. Example: 1GB of documents ≈ 300-500MB index.

**Q: Can I index network drives or external storage?**
A: Yes, but performance will depend on network/USB speed. Local SSD is recommended.

### Indexing

**Q: How long does it take to index 10,000 files?**
A: Approximately 3-5 minutes on recommended hardware (SSD, 4+ cores). See [PERFORMANCE.md](../PERFORMANCE.md) for details.

**Q: Can I index while searching?**
A: Yes! The database uses WAL mode allowing concurrent reads during writes.

**Q: What happens if I index the same directory twice?**
A: Files are updated (not duplicated). Content hash prevents duplicates.

**Q: Can I exclude specific files or patterns?**
A: Not yet. Planned for Phase 1 with custom ignore patterns.

### Searching

**Q: Why do results show matches I didn't search for?**
A: FTS5 uses Porter stemming (e.g., "run" matches "running"). This improves recall.

**Q: Can I search for special characters?**
A: Yes, but FTS5 may tokenize them differently. Use quotes for exact matches: `"user@example.com"`

**Q: How are results ranked?**
A: FTS5 BM25 algorithm ranks by:
- Term frequency (how often query appears)
- Document frequency (rarity of terms)
- Document length (shorter docs rank higher for same match)

**Q: Can I save searches?**
A: Not yet. Planned for Phase 2.

### Privacy & Security

**Q: Is my data encrypted?**
A: Not by default. The SQLite database is stored unencrypted. Use OS-level encryption (FileVault, BitLocker, LUKS) for sensitive data.

**Q: Can I password-protect the index?**
A: Not yet. Planned for Phase 2 with optional encryption.

**Q: What data is collected?**
A: None. Cortex is completely offline with no analytics or telemetry.

### Performance

**Q: Why is PDF extraction slow?**
A: PDF parsing is complex and CPU-intensive. Typical rate: ~50 files/second. Consider excluding large PDF directories for faster indexing.

**Q: Does Cortex support parallel indexing?**
A: Not yet. Single-threaded indexing in Phase 0. Parallel extraction planned for Phase 1.

**Q: How do I optimize search performance?**
A:
1. Use specific filters to narrow results
2. Run `VACUUM` periodically
3. Ensure SSD storage
4. Close other applications

---

## Performance Tips

### Indexing Performance

**1. Use SSD Storage**
- 5-10x faster than HDD
- Especially important for random I/O

**2. Index During Off-Hours**
- Less CPU competition
- System caches are warm
- Better sustained performance

**3. Exclude Large Directories**
- Skip `node_modules`, `target`, `.git`
- Use selective indexing for project directories
- Focus on document directories

**4. Optimize File Selection**
- Exclude large binary files
- Skip duplicate directories (backups)
- Index source code separately from documents

### Search Performance

**1. Use Filters**
```
Query: tutorial
Filter: Type = "md", Modified > 2025-01-01
```
Reduces search space dramatically.

**2. Specific Queries**
```
Bad:  a
Good: algorithm
```
Longer, specific terms are faster.

**3. Pagination**
Request only what you need:
```
Limit: 20 (instead of default 50)
```

**4. Database Maintenance**
Run monthly:
```bash
sqlite3 ~/.cortex/db.sqlite "VACUUM;"
sqlite3 ~/.cortex/db.sqlite "ANALYZE;"
```

### Storage Optimization

**1. Regular Cleanup**
- Remove deleted files from index
- Clean up old indexes
- Delete temporary files

**2. Compress Archives**
- Index extracted content, not archives
- Use external compression

**3. Selective Indexing**
- Don't index everything
- Focus on active projects/documents
- Create separate indexes for different contexts

---

## Getting Help

### Documentation

- **User Guide** - This document
- **[API Reference](API_REFERENCE.md)** - For developers
- **[Developer Guide](DEVELOPER_GUIDE.md)** - Contributing
- **[Performance Guide](../PERFORMANCE.md)** - Optimization
- **[Testing Guide](../TESTING.md)** - Running tests

### Support

- **GitHub Issues** - [Report bugs](https://github.com/yourusername/cortex/issues)
- **Discussions** - [Ask questions](https://github.com/yourusername/cortex/discussions)
- **Discord** - [Community chat](https://discord.gg/cortex)

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Code of conduct
- Development setup
- Pull request process
- Coding standards

---

**Cortex User Guide v0.1.0** | [Report Issue](https://github.com/yourusername/cortex/issues) | [Edit on GitHub](https://github.com/yourusername/cortex/edit/main/docs/USER_GUIDE.md)
