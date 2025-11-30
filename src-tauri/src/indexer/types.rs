use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Priority levels for indexing files
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IndexPriority {
    /// Large files > 100MB - lowest priority
    Low = 0,

    /// Medium files 10MB - 100MB
    Normal = 1,

    /// Small files 1MB - 10MB
    High = 2,

    /// Very small files < 1MB or recently modified
    Immediate = 3,
}

impl IndexPriority {
    /// Determine priority based on file size
    pub fn from_size(size: u64) -> Self {
        if size < 1_000_000 {
            // < 1MB
            IndexPriority::Immediate
        } else if size < 10_000_000 {
            // 1MB - 10MB
            IndexPriority::High
        } else if size < 100_000_000 {
            // 10MB - 100MB
            IndexPriority::Normal
        } else {
            // > 100MB
            IndexPriority::Low
        }
    }
}

/// Represents a file to be indexed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexJob {
    pub path: PathBuf,
    pub priority: IndexPriority,
    pub size: u64,
    #[serde(skip, default = "std::time::SystemTime::now")]
    pub modified: std::time::SystemTime,
}

impl PartialEq for IndexJob {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.priority == other.priority && self.size == other.size
    }
}

impl Eq for IndexJob {}

impl IndexJob {
    pub fn new(path: PathBuf, size: u64, modified: std::time::SystemTime) -> Self {
        let priority = IndexPriority::from_size(size);
        Self {
            path,
            priority,
            size,
            modified,
        }
    }
}

/// Progress information for indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub total_files: usize,
    pub current_file: usize,
}

impl ScanProgress {
    pub fn new() -> Self {
        Self {
            total_files: 0,
            current_file: 0,
        }
    }

    pub fn percentage(&self) -> f64 {
        if self.total_files == 0 {
            0.0
        } else {
            (self.current_file as f64 / self.total_files as f64) * 100.0
        }
    }

    pub fn update_current(&mut self, _path: std::path::PathBuf) {
        self.current_file += 1;
    }

    pub fn add_error(&mut self, _error: String) {
        // Errors are tracked in AppState.indexing_errors
    }

}

impl Default for ScanProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_from_size() {
        assert_eq!(IndexPriority::from_size(500_000), IndexPriority::Immediate); // 500KB
        assert_eq!(IndexPriority::from_size(5_000_000), IndexPriority::High); // 5MB
        assert_eq!(IndexPriority::from_size(50_000_000), IndexPriority::Normal); // 50MB
        assert_eq!(IndexPriority::from_size(150_000_000), IndexPriority::Low); // 150MB
    }

    #[test]
    fn test_priority_ordering() {
        assert!(IndexPriority::Immediate > IndexPriority::High);
        assert!(IndexPriority::High > IndexPriority::Normal);
        assert!(IndexPriority::Normal > IndexPriority::Low);
    }

    #[test]
    fn test_scan_progress() {
        let mut progress = ScanProgress::new();
        assert_eq!(progress.percentage(), 0.0);

        progress.total_files = 100;
        progress.scanned_files = 50;
        assert_eq!(progress.percentage(), 50.0);

        progress.add_error("Test error".to_string());
        assert_eq!(progress.errors.len(), 1);
    }
}
