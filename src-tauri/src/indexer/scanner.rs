use crate::error::{CortexError, Result};
use crate::indexer::types::{IndexJob, IndexPriority, ScanProgress};
use std::collections::BinaryHeap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use walkdir::{DirEntry, WalkDir};

/// Supported file types for indexing
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "txt", "md", "pdf", "docx", "doc", "rtf",
    "rs", "js", "ts", "py", "java", "c", "cpp", "h", "hpp",
    "json", "yaml", "yml", "toml", "xml", "html", "css",
];

/// File scanner for recursive directory traversal
pub struct FileScanner {
    /// Progress tracking
    progress: Arc<RwLock<ScanProgress>>,

    /// Maximum file size to index (default: 100MB)
    max_file_size: u64,

    /// Whether to follow symlinks
    follow_symlinks: bool,
}

impl FileScanner {
    pub fn new() -> Self {
        Self {
            progress: Arc::new(RwLock::new(ScanProgress::new())),
            max_file_size: 100_000_000, // 100MB
            follow_symlinks: false,
        }
    }

    /// Set maximum file size to index
    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set whether to follow symlinks
    pub fn with_follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    /// Get current progress
    pub fn get_progress(&self) -> ScanProgress {
        self.progress.read().unwrap().clone()
    }

    /// Scan a directory and return a priority queue of files to index
    pub fn scan_directory(&self, root_path: &Path) -> Result<Vec<IndexJob>> {
        if !root_path.exists() {
            return Err(CortexError::FileNotFound {
                path: root_path.to_string_lossy().to_string(),
            });
        }

        if !root_path.is_dir() {
            return Err(CortexError::Internal {
                message: format!("{} is not a directory", root_path.display()),
            });
        }

        log::info!("Starting directory scan: {}", root_path.display());

        // First pass: count total files
        let total = self.count_files(root_path);
        {
            let mut progress = self.progress.write().unwrap();
            progress.total_files = total;
        }

        // Second pass: collect files
        let mut jobs = Vec::new();

        let walker = WalkDir::new(root_path)
            .follow_links(self.follow_symlinks)
            .into_iter()
            .filter_entry(|e| self.should_visit(e));

        for entry in walker {
            match entry {
                Ok(entry) => {
                    if let Some(job) = self.process_entry(&entry)? {
                        jobs.push(job);

                        // Update progress
                        let mut progress = self.progress.write().unwrap();
                        progress.update_current(entry.path().to_path_buf());
                    }
                }
                Err(e) => {
                    let error_msg = format!("Error walking directory: {}", e);
                    log::warn!("{}", error_msg);

                    let mut progress = self.progress.write().unwrap();
                    progress.add_error(error_msg);
                }
            }
        }

        // Sort by priority (highest first)
        jobs.sort_by(|a, b| b.priority.cmp(&a.priority));

        log::info!("Scan complete: {} files found", jobs.len());

        Ok(jobs)
    }

    /// Count total files in directory
    fn count_files(&self, root_path: &Path) -> usize {
        WalkDir::new(root_path)
            .follow_links(self.follow_symlinks)
            .into_iter()
            .filter_entry(|e| self.should_visit(e))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .count()
    }

    /// Check if we should visit this directory entry
    fn should_visit(&self, entry: &DirEntry) -> bool {
        let path = entry.path();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Skip hidden files and directories
        if file_name.starts_with('.') && file_name != "." {
            return false;
        }

        // Skip common directories to ignore
        let ignore_dirs = ["node_modules", "target", "dist", "build", ".git", ".svn"];
        if entry.file_type().is_dir() && ignore_dirs.contains(&file_name) {
            return false;
        }

        true
    }

    /// Process a directory entry and create an IndexJob if applicable
    fn process_entry(&self, entry: &DirEntry) -> Result<Option<IndexJob>> {
        // Only process files
        if !entry.file_type().is_file() {
            return Ok(None);
        }

        let path = entry.path();

        // Check if file extension is supported
        if !self.is_supported_file(path) {
            return Ok(None);
        }

        // Get file metadata
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Failed to get metadata for {}: {}", path.display(), e);
                return Ok(None);
            }
        };

        let size = metadata.len();

        // Skip files that are too large
        if size > self.max_file_size {
            log::debug!("Skipping large file: {} ({}MB)", path.display(), size / 1_000_000);
            return Ok(None);
        }

        let modified = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);

        Ok(Some(IndexJob::new(path.to_path_buf(), size, modified)))
    }

    /// Check if file has a supported extension
    fn is_supported_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }
}

impl Default for FileScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Priority queue wrapper for IndexJobs
pub struct IndexQueue {
    queue: BinaryHeap<PriorityJob>,
}

#[derive(Debug, Eq, PartialEq)]
struct PriorityJob {
    job: IndexJob,
}

impl Ord for PriorityJob {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first
        self.job.priority.cmp(&other.job.priority)
            // Then by modified time (newer first)
            .then_with(|| other.job.modified.cmp(&self.job.modified))
    }
}

impl PartialOrd for PriorityJob {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl IndexQueue {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, job: IndexJob) {
        self.queue.push(PriorityJob { job });
    }

    pub fn pop(&mut self) -> Option<IndexJob> {
        self.queue.pop().map(|pj| pj.job)
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn extend(&mut self, jobs: Vec<IndexJob>) {
        for job in jobs {
            self.push(job);
        }
    }
}

impl Default for IndexQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_scanner_basic() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create test files
        create_test_file(dir_path, "test1.txt", "content 1");
        create_test_file(dir_path, "test2.md", "content 2");
        create_test_file(dir_path, "test3.rs", "content 3");

        let scanner = FileScanner::new();
        let jobs = scanner.scan_directory(dir_path).unwrap();

        assert_eq!(jobs.len(), 3);
    }

    #[test]
    fn test_scanner_ignores_unsupported() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        create_test_file(dir_path, "test.txt", "content");
        create_test_file(dir_path, "test.exe", "binary");
        create_test_file(dir_path, "test.xyz", "unsupported");

        let scanner = FileScanner::new();
        let jobs = scanner.scan_directory(dir_path).unwrap();

        assert_eq!(jobs.len(), 1);
        assert!(jobs[0].path.ends_with("test.txt"));
    }

    #[test]
    fn test_scanner_ignores_hidden() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        create_test_file(dir_path, "visible.txt", "content");
        create_test_file(dir_path, ".hidden.txt", "content");

        let scanner = FileScanner::new();
        let jobs = scanner.scan_directory(dir_path).unwrap();

        assert_eq!(jobs.len(), 1);
        assert!(jobs[0].path.ends_with("visible.txt"));
    }

    #[test]
    fn test_scanner_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create nested structure
        let sub_dir = dir_path.join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        create_test_file(dir_path, "root.txt", "root content");
        create_test_file(&sub_dir, "nested.txt", "nested content");

        let scanner = FileScanner::new();
        let jobs = scanner.scan_directory(dir_path).unwrap();

        assert_eq!(jobs.len(), 2);
    }

    #[test]
    fn test_scanner_ignores_node_modules() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        let node_modules = dir_path.join("node_modules");
        fs::create_dir(&node_modules).unwrap();

        create_test_file(dir_path, "app.js", "app");
        create_test_file(&node_modules, "lib.js", "lib");

        let scanner = FileScanner::new();
        let jobs = scanner.scan_directory(dir_path).unwrap();

        assert_eq!(jobs.len(), 1);
        assert!(jobs[0].path.ends_with("app.js"));
    }

    #[test]
    fn test_priority_queue_ordering() {
        let mut queue = IndexQueue::new();

        let small_file = IndexJob::new(
            PathBuf::from("small.txt"),
            500_000, // 500KB - Immediate priority
            std::time::SystemTime::now(),
        );

        let large_file = IndexJob::new(
            PathBuf::from("large.txt"),
            50_000_000, // 50MB - Normal priority
            std::time::SystemTime::now(),
        );

        // Add in reverse order
        queue.push(large_file);
        queue.push(small_file.clone());

        // Should get small file first (higher priority)
        let first = queue.pop().unwrap();
        assert!(first.path.ends_with("small.txt"));
        assert_eq!(first.priority, IndexPriority::Immediate);

        let second = queue.pop().unwrap();
        assert!(second.path.ends_with("large.txt"));
        assert_eq!(second.priority, IndexPriority::Normal);
    }

    #[test]
    fn test_scanner_progress_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        create_test_file(dir_path, "test1.txt", "content 1");
        create_test_file(dir_path, "test2.txt", "content 2");

        let scanner = FileScanner::new();
        let _jobs = scanner.scan_directory(dir_path).unwrap();

        let progress = scanner.get_progress();
        assert_eq!(progress.total_files, 2);
        assert_eq!(progress.scanned_files, 2);
        assert_eq!(progress.percentage(), 100.0);
    }

    #[test]
    fn test_scanner_max_file_size() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        create_test_file(dir_path, "small.txt", "content");

        // Create a "large" file (for testing, just check metadata)
        let large_path = dir_path.join("large.txt");
        let mut file = fs::File::create(&large_path).unwrap();
        file.write_all(&vec![0u8; 1000]).unwrap();

        // Set max size very low
        let scanner = FileScanner::new().with_max_file_size(100);
        let jobs = scanner.scan_directory(dir_path).unwrap();

        // Should only find the small file
        assert_eq!(jobs.len(), 1);
    }
}
