use crate::error::{CortexError, Result};
use crate::indexer::types::IndexJob;
use crossbeam_channel::{bounded, Receiver, Sender};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// Filesystem watcher that detects file changes
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    receiver: Receiver<IndexJob>,
}

impl FileWatcher {
    /// Create a new filesystem watcher for the given paths
    pub fn new(paths: Vec<PathBuf>) -> Result<Self> {
        let (tx, rx) = bounded(1000);

        let watcher = Self::create_watcher(tx, paths)?;

        Ok(Self {
            _watcher: watcher,
            receiver: rx,
        })
    }

    /// Create the notify watcher
    fn create_watcher(
        tx: Sender<IndexJob>,
        paths: Vec<PathBuf>,
    ) -> Result<RecommendedWatcher> {
        let tx = Arc::new(tx);
        let tx_clone = Arc::clone(&tx);

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    if let Some(job) = Self::process_event(&event) {
                        if let Err(e) = tx_clone.send(job) {
                            log::error!("Failed to send watch event: {}", e);
                        }
                    }
                }
                Err(e) => log::error!("Watch error: {}", e),
            }
        })
        .map_err(|e| CortexError::Internal {
            message: format!("Failed to create file watcher: {}", e),
        })?;

        // Watch all provided paths
        for path in paths {
            watcher
                .watch(&path, RecursiveMode::Recursive)
                .map_err(|e| CortexError::Internal {
                    message: format!("Failed to watch {}: {}", path.display(), e),
                })?;

            log::info!("Watching directory: {}", path.display());
        }

        Ok(watcher)
    }

    /// Process a filesystem event and create an IndexJob if relevant
    fn process_event(event: &Event) -> Option<IndexJob> {
        match event.kind {
            // File created or modified
            EventKind::Create(_) | EventKind::Modify(_) => {
                if let Some(path) = event.paths.first() {
                    if path.is_file() {
                        return Self::create_index_job(path);
                    }
                }
            }
            // We might also want to handle renames
            EventKind::Modify(notify::event::ModifyKind::Name(_)) => {
                if let Some(path) = event.paths.last() {
                    if path.is_file() {
                        return Self::create_index_job(path);
                    }
                }
            }
            _ => {}
        }

        None
    }

    /// Create an IndexJob from a file path
    fn create_index_job(path: &Path) -> Option<IndexJob> {
        let metadata = path.metadata().ok()?;

        if !metadata.is_file() {
            return None;
        }

        let size = metadata.len();
        let modified = metadata
            .modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

        Some(IndexJob::new(path.to_path_buf(), size, modified))
    }

    /// Get the receiver for watch events
    pub fn receiver(&self) -> &Receiver<IndexJob> {
        &self.receiver
    }

    /// Try to receive a watch event (non-blocking)
    pub fn try_recv(&self) -> Option<IndexJob> {
        self.receiver.try_recv().ok()
    }

    /// Receive a watch event with timeout
    pub fn recv_timeout(&self, timeout: Duration) -> Option<IndexJob> {
        self.receiver.recv_timeout(timeout).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let watcher = FileWatcher::new(vec![temp_dir.path().to_path_buf()]);
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_watcher_detects_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        let watcher = FileWatcher::new(vec![dir_path.to_path_buf()]).unwrap();

        // Give watcher time to initialize
        thread::sleep(Duration::from_millis(100));

        // Create a new file
        let file_path = dir_path.join("test.txt");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);

        // Wait for event to be processed
        thread::sleep(Duration::from_millis(500));

        // Check if we received an event
        let event = watcher.recv_timeout(Duration::from_secs(1));
        assert!(event.is_some(), "Should detect new file creation");

        if let Some(job) = event {
            assert!(job.path.ends_with("test.txt"));
        }
    }

    #[test]
    fn test_watcher_detects_modification() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create file before watcher
        let file_path = dir_path.join("existing.txt");
        fs::write(&file_path, "initial content").unwrap();

        let watcher = FileWatcher::new(vec![dir_path.to_path_buf()]).unwrap();

        // Give watcher time to initialize
        thread::sleep(Duration::from_millis(100));

        // Modify the file
        fs::write(&file_path, "modified content").unwrap();

        // Wait for event
        thread::sleep(Duration::from_millis(500));

        let event = watcher.recv_timeout(Duration::from_secs(1));
        assert!(event.is_some(), "Should detect file modification");
    }

    #[test]
    fn test_watcher_non_blocking_recv() {
        let temp_dir = TempDir::new().unwrap();
        let watcher = FileWatcher::new(vec![temp_dir.path().to_path_buf()]).unwrap();

        // Should return None immediately if no events
        let result = watcher.try_recv();
        assert!(result.is_none());
    }
}
