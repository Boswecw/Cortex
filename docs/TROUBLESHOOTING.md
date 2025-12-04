# Cortex Troubleshooting Guide

Complete guide to diagnosing and fixing common issues with Cortex.

**Last Updated:** December 4, 2025

---

## 📑 Table of Contents

- [Quick Diagnostic Checklist](#quick-diagnostic-checklist)
- [Installation Issues](#installation-issues)
- [Application Launch Issues](#application-launch-issues)
- [Indexing Issues](#indexing-issues)
- [Search Issues](#search-issues)
- [Export Issues](#export-issues)
- [Performance Issues](#performance-issues)
- [Database Issues](#database-issues)
- [File System Issues](#file-system-issues)
- [Platform-Specific Issues](#platform-specific-issues)
- [Getting Help](#getting-help)

---

## 🚀 Quick Diagnostic Checklist

Before diving into specific issues, run through this checklist:

- [ ] **Cortex version:** Run `cortex --version` (check you're on latest)
- [ ] **System requirements:** OS compatibility, disk space, permissions
- [ ] **Database integrity:** Check `.cortex/cortex.db` exists and is readable
- [ ] **Logs location:** Find logs at:
  - Linux: `~/.local/share/cortex/logs/`
  - macOS: `~/Library/Application Support/cortex/logs/`
  - Windows: `%APPDATA%\cortex\logs\`
- [ ] **Recent changes:** Did you update, move files, or change permissions?

---

## 🔧 Installation Issues

### Issue: Build Fails with "Cannot find Rust"

**Symptoms:**
```
error: could not find `rustc`
```

**Solution:**
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload shell
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Issue: "Could not compile tauri" Errors

**Symptoms:**
```
error: failed to compile `tauri-cli`
```

**Causes:**
- Missing system dependencies
- Outdated Rust toolchain

**Solutions:**

**On Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

**On macOS:**
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

**On Windows:**
- Install Microsoft Visual C++ Build Tools
- Install WebView2 Runtime

**Update Rust:**
```bash
rustup update stable
```

### Issue: Node/pnpm Installation Problems

**Symptoms:**
```
error: pnpm: command not found
```

**Solution:**
```bash
# Install Node.js (v18+)
# Via nvm (recommended):
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
nvm use 20

# Install pnpm
npm install -g pnpm

# Verify
pnpm --version
```

---

## 🚀 Application Launch Issues

### Issue: Application Won't Start

**Symptoms:**
- Double-clicking does nothing
- Terminal shows errors
- App crashes immediately

**Diagnostic Steps:**

1. **Run from terminal to see errors:**
   ```bash
   cd src-tauri
   cargo run --release
   ```

2. **Check for port conflicts:**
   ```bash
   # Check if default port is in use
   lsof -i :1420  # Default Tauri dev port
   ```

3. **Check database permissions:**
   ```bash
   ls -la ~/.cortex/
   # Should be readable/writable by your user
   ```

**Solutions:**

**Database locked error:**
```bash
# Remove lock file
rm -f ~/.cortex/cortex.db-wal
rm -f ~/.cortex/cortex.db-shm

# Restart application
```

**Permissions error:**
```bash
# Fix permissions
chmod 755 ~/.cortex
chmod 644 ~/.cortex/cortex.db
```

### Issue: "WebView2 Not Found" (Windows)

**Solution:**
Download and install Microsoft Edge WebView2 Runtime:
https://developer.microsoft.com/en-us/microsoft-edge/webview2/

### Issue: Application Crashes on Startup

**Check logs:**
```bash
# Linux/macOS
tail -f ~/.local/share/cortex/logs/cortex.log

# Windows
type %APPDATA%\cortex\logs\cortex.log
```

**Common causes:**
- Corrupted database → Delete `~/.cortex/cortex.db` and restart
- Missing dependencies → Reinstall
- Conflicting installations → Remove old versions

---

## 📚 Indexing Issues

### Issue: Files Not Being Indexed

**Symptoms:**
- File count stays at 0
- "Scan complete: 0 files" message
- Files exist but don't appear in Cortex

**Diagnostic:**
```bash
# Check if files are in supported formats
# Supported: .rs, .js, .ts, .py, .java, .c, .cpp, .txt, .md, .pdf, .docx, etc.

# Check file permissions
ls -la /path/to/project
```

**Solutions:**

1. **Check file extensions:**
   - Only supported file types are indexed
   - See [src-tauri/src/indexer/scanner.rs](../src-tauri/src/indexer/scanner.rs#L8-L12) for full list

2. **Hidden files/directories:**
   - Files starting with `.` are ignored (except `.md`, `.rs` in project root)
   - `node_modules`, `target`, `dist`, `build`, `.git` are auto-ignored

3. **File size limits:**
   - Default max: 100MB per file
   - Large files are skipped
   - Check logs for "Skipping large file" messages

4. **Permissions:**
   ```bash
   # Make files readable
   chmod -R +r /path/to/project
   ```

### Issue: Indexing is Very Slow

**Expected Performance:**
- ~100-500 files/second (depends on file size and type)
- PDF/DOCX parsing is slower than plain text

**Optimization Tips:**

1. **Exclude unnecessary directories:**
   - Manually exclude `node_modules`, large build artifacts
   - Use .gitignore-style patterns (feature coming)

2. **Reduce file size:**
   - Consider excluding very large files
   - Remove unnecessary binary files

3. **Check system resources:**
   ```bash
   # Monitor CPU/memory during indexing
   top
   htop  # if installed
   ```

### Issue: "Failed to extract content" Errors

**Causes:**
- Corrupted files
- Unsupported file encodings
- PDF/DOCX parsing failures

**Solutions:**

1. **Check file integrity:**
   ```bash
   file /path/to/problematic-file
   ```

2. **Re-save file with UTF-8 encoding:**
   ```bash
   # Convert file to UTF-8
   iconv -f ISO-8859-1 -t UTF-8 input.txt > output.txt
   ```

3. **Skip problematic files:**
   - Indexing continues despite individual file failures
   - Check logs to identify problem files

---

## 🔍 Search Issues

### Issue: Search Returns No Results

**Symptoms:**
- Know file exists and is indexed
- Search query returns 0 results
- FTS search not working

**Diagnostic:**

1. **Verify file is indexed:**
   - Check file list in UI
   - Confirm file count > 0

2. **Check query syntax:**
   - FTS uses SQLite FTS5 syntax
   - Special characters need quoting
   - Minimum query length: 2 characters

**Solutions:**

**Query syntax examples:**
```
# Good queries:
rust functions        # Multiple words (AND)
"exact phrase"        # Exact match
rust OR python        # Either word
rust NOT deprecated   # Exclude word

# Bad queries:
a                     # Too short
@#$%                  # Invalid characters
```

**Rebuild FTS index:**
```sql
# Open database with sqlite3
sqlite3 ~/.cortex/cortex.db

# Rebuild FTS
INSERT INTO files_fts(files_fts) VALUES('rebuild');

.quit
```

### Issue: Search Results Are Incorrect

**Symptoms:**
- Getting results for wrong files
- Results don't match query
- Outdated content in search

**Solution:**
1. Re-index the project (Update → Reindex in UI)
2. Check if files were modified after indexing
3. Verify FTS triggers are working (see Database Issues)

---

## 📤 Export Issues

### Issue: Export Fails with "Invalid Path" Error

**Symptoms:**
```
Error: InvalidPath { path: "../../etc/passwd", reason: "Path traversal is not allowed" }
```

**Cause:** Security validation blocking path traversal attempts

**Solution:**
Use valid export paths:
```bash
# ✅ Valid paths
.cortex-export                    # Relative to project
./exports/my-export               # Subdirectory
~/exports/cortex                  # Home directory
/tmp/cortex-export                # Temp directory

# ❌ Invalid paths
../../etc/passwd                  # Path traversal
/system/critical                  # System directories
```

### Issue: Export Creates Empty Files

**Symptoms:**
- CONTEXT.md exists but is empty or very small
- Export completes but no content

**Diagnostic:**
```bash
# Check file count in database
sqlite3 ~/.cortex/cortex.db "SELECT COUNT(*) FROM files WHERE is_deleted = 0"

# Check indexed content
sqlite3 ~/.cortex/cortex.db "SELECT COUNT(*) FROM file_content"
```

**Solutions:**

1. **No files indexed:**
   - Index project first
   - Check "Indexing Issues" section above

2. **Files indexed but no content:**
   ```sql
   # Check if extraction failed
   sqlite3 ~/.cortex/cortex.db
   SELECT COUNT(*) FROM files f
   LEFT JOIN file_content fc ON f.id = fc.file_id
   WHERE fc.file_id IS NULL;
   ```
   - Re-index files to extract content

3. **Check export config:**
   - Verify collection_id is valid (if specified)
   - Check include_prompts setting

### Issue: Export is Very Large (> 50MB)

**Cause:** Exporting many large files or embeddings

**Solutions:**

1. **Exclude embeddings:**
   - Uncheck "Include Embeddings" in export settings
   - Embeddings can add 10-100x size

2. **Export specific collection:**
   - Create a collection with only needed files
   - Export that collection instead of entire project

3. **Check file count:**
   ```sql
   sqlite3 ~/.cortex/cortex.db "SELECT COUNT(*), SUM(size) FROM files"
   ```

### Issue: "no such column: T.filename" Error During Export

**Symptoms:**
```
DatabaseError { message: "no such column: T.filename" }
```

**Cause:** Database schema mismatch (FTS external content issue)

**Solution:**
This was fixed in commit `f85bfc0`. Update to latest version:
```bash
git pull
cargo build --release
```

If still occurring, rebuild database:
```bash
# Backup first!
cp ~/.cortex/cortex.db ~/.cortex/cortex.db.backup

# Delete and re-create
rm ~/.cortex/cortex.db
# Restart Cortex, it will recreate DB
```

---

## ⚡ Performance Issues

### Issue: High Memory Usage

**Symptoms:**
- Application using > 1GB RAM
- System becomes slow during indexing
- Memory grows over time

**Diagnostic:**
```bash
# Monitor memory
ps aux | grep cortex

# Or use system monitor
top -p $(pgrep cortex)
```

**Solutions:**

1. **Batch processing:**
   - Index large projects in chunks
   - Use collections to limit scope

2. **Disable embeddings:**
   - Embeddings use significant memory
   - Only enable if needed for similarity search

3. **Restart periodically:**
   - Memory leaks are being investigated
   - Restart Cortex after indexing large projects

### Issue: Slow File Scanning

**Expected:** ~1000-5000 files/sec for scanning (not indexing)

**Causes:**
- Network drives (very slow)
- Many small files
- Antivirus scanning

**Solutions:**

1. **Use local drive:**
   - Copy project to local SSD
   - Index there, then move back

2. **Exclude from antivirus:**
   - Add Cortex and project to AV exclusions
   - Scanning can slow I/O by 10-100x

3. **Check disk:**
   ```bash
   # Test disk speed
   dd if=/dev/zero of=test.bin bs=1M count=1024
   ```

### Issue: Database Queries Are Slow

**Symptoms:**
- Search takes > 1 second
- UI feels sluggish
- Loading files is slow

**Solutions:**

1. **Vacuum database:**
   ```sql
   sqlite3 ~/.cortex/cortex.db
   VACUUM;
   ANALYZE;
   .quit
   ```

2. **Rebuild indexes:**
   ```sql
   sqlite3 ~/.cortex/cortex.db
   REINDEX;
   .quit
   ```

3. **Check database size:**
   ```bash
   du -h ~/.cortex/cortex.db
   # If > 1GB for small project, corruption possible
   ```

---

## 🗄️ Database Issues

### Issue: "Database is Locked" Error

**Symptoms:**
```
Error: SqliteFailure(Error { code: Busy, extended_code: 5 }, Some("database is locked"))
```

**Cause:** Another process has the database open

**Solutions:**

1. **Close other Cortex instances:**
   ```bash
   # Find all Cortex processes
   ps aux | grep cortex

   # Kill if needed
   killall cortex
   ```

2. **Remove lock files:**
   ```bash
   cd ~/.cortex
   rm -f cortex.db-shm cortex.db-wal
   ```

3. **Check for zombie processes:**
   ```bash
   lsof ~/.cortex/cortex.db
   ```

### Issue: "Database Disk Image is Malformed"

**Symptoms:**
```
Error: DatabaseError { message: "database disk image is malformed" }
```

**Cause:** Corruption (crash, power loss, disk error)

**Solutions:**

1. **Try recovery:**
   ```bash
   # Backup corrupted DB
   cp ~/.cortex/cortex.db ~/.cortex/cortex.db.corrupted

   # Attempt recovery
   sqlite3 ~/.cortex/cortex.db ".recover" | sqlite3 ~/.cortex/cortex_recovered.db
   mv ~/.cortex/cortex_recovered.db ~/.cortex/cortex.db
   ```

2. **Start fresh:**
   ```bash
   # Delete corrupted database
   rm ~/.cortex/cortex.db

   # Restart Cortex (will create new DB)
   # Re-index your project
   ```

3. **Prevent future corruption:**
   - Don't force-quit Cortex
   - Let indexing complete before closing
   - Use reliable storage (not network drives)

### Issue: Database Growing Too Large

**Expected sizes:**
- 1000 files: ~10-50MB
- 10000 files: ~100-500MB
- With embeddings: 5-10x larger

**If larger than expected:**

1. **Vacuum to reclaim space:**
   ```sql
   sqlite3 ~/.cortex/cortex.db VACUUM;
   ```

2. **Check for deleted files:**
   ```sql
   sqlite3 ~/.cortex/cortex.db
   SELECT COUNT(*) FROM files WHERE is_deleted = 1;
   -- If many deleted files, they're still in DB
   ```

3. **Purge deleted files:**
   ```sql
   -- Permanently remove soft-deleted files
   DELETE FROM file_content WHERE file_id IN
     (SELECT id FROM files WHERE is_deleted = 1);
   DELETE FROM files WHERE is_deleted = 1;
   VACUUM;
   ```

---

## 📁 File System Issues

### Issue: "Permission Denied" Errors

**Solutions:**

```bash
# Make Cortex data directory writable
chmod 755 ~/.cortex
chmod 644 ~/.cortex/*

# Make project readable
chmod -R +r /path/to/project
```

### Issue: Files on Network Drive Won't Index

**Network drives are supported but slow.**

**Optimizations:**
1. Use absolute paths
2. Ensure stable connection
3. Consider copying to local drive first

### Issue: Symlinks Not Followed

**By default, symlinks are NOT followed** (security)

**To enable:**
- Feature coming in future release
- For now, use actual directories or hard links

---

## 💻 Platform-Specific Issues

### macOS

**Issue: "Cannot verify developer" warning**

**Solution:**
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine /path/to/Cortex.app
```

**Issue: Gatekeeper blocking launch**

System Preferences → Security & Privacy → Allow Cortex

### Windows

**Issue: SmartScreen blocking**

"More info" → "Run anyway"

**Issue: Antivirus false positive**

Add exclusion for:
- Cortex installation directory
- `%APPDATA%\cortex`
- Your project directories

### Linux

**Issue: AppImage won't execute**

```bash
chmod +x Cortex.AppImage
./Cortex.AppImage
```

**Issue: Missing dependencies**

```bash
# Install required libraries
sudo apt install libwebkit2gtk-4.1-0 libappindicator3-1
```

---

## 🆘 Getting Help

### Before Asking for Help

1. **Check this troubleshooting guide**
2. **Search existing GitHub issues**
3. **Collect diagnostic information:**

```bash
# Cortex version
cortex --version

# System info
uname -a                  # Linux/macOS
systeminfo                # Windows

# Recent logs (last 50 lines)
tail -50 ~/.local/share/cortex/logs/cortex.log

# Database stats
sqlite3 ~/.cortex/cortex.db "SELECT
  (SELECT COUNT(*) FROM files) as total_files,
  (SELECT COUNT(*) FROM file_content) as indexed_files,
  (SELECT COUNT(*) FROM files WHERE is_deleted = 1) as deleted_files;"
```

### How to Report a Bug

**Include:**
1. Cortex version
2. Operating system and version
3. Steps to reproduce
4. Expected vs actual behavior
5. Relevant log excerpts
6. Database statistics (if applicable)

**GitHub Issues:**
https://github.com/your-org/cortex/issues

**Format:**
```markdown
## Bug Report

**Cortex Version:** v0.1.0
**OS:** Ubuntu 22.04 LTS
**Node Version:** v20.10.0

**Steps to Reproduce:**
1. Index project with 1000 files
2. Open search
3. Type query "function"
4. Click search

**Expected:** Show search results
**Actual:** Application freezes

**Logs:**
```
[ERROR] Search query failed: timeout
```

**Database Stats:**
- Total files: 1000
- Indexed: 950
- Deleted: 50
```

---

## 📋 Quick Reference

### Common Commands

```bash
# Build release version
cargo build --release

# Run with logs
RUST_LOG=debug cargo run

# Run tests
cargo test

# Clean build
cargo clean && cargo build --release

# Check database
sqlite3 ~/.cortex/cortex.db "SELECT COUNT(*) FROM files"

# Reset Cortex (DELETES ALL DATA)
rm -rf ~/.cortex
```

### Log Levels

```bash
# Minimal logging
RUST_LOG=error cargo run

# Normal logging
RUST_LOG=info cargo run

# Verbose logging
RUST_LOG=debug cargo run

# Everything
RUST_LOG=trace cargo run
```

### Emergency Recovery

**If nothing works:**

```bash
# 1. Backup everything
cp -r ~/.cortex ~/.cortex.backup

# 2. Clean slate
rm -rf ~/.cortex

# 3. Reinstall
git pull
cargo clean
cargo build --release

# 4. Start fresh
cargo run --release
```

---

## 🔄 Maintenance Tips

### Regular Maintenance

**Weekly:**
- Vacuum database: `sqlite3 ~/.cortex/cortex.db VACUUM`
- Check log file size

**Monthly:**
- Update Cortex: `git pull && cargo build --release`
- Review and purge deleted files
- Check disk space

**Before Major Updates:**
- Backup database: `cp ~/.cortex/cortex.db ~/cortex-backup.db`
- Export critical collections
- Note custom configurations

---

## 📚 Additional Resources

- **User Guide:** [docs/USER_GUIDE.md](USER_GUIDE.md)
- **Architecture:** [docs/ARCHITECTURE.md](ARCHITECTURE.md)
- **Contributing:** [CONTRIBUTING.md](../CONTRIBUTING.md)
- **API Reference:** [docs/API_REFERENCE.md](API_REFERENCE.md)

---

*Last updated: December 4, 2025*
*Cortex version: 0.1.0*
