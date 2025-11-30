use crate::db::Database;
use crate::error::Result;
use crate::indexer::ScanProgress;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

/// Global application state shared across Tauri commands
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub indexing_active: Arc<RwLock<bool>>,
    pub indexing_progress: Arc<RwLock<Option<ScanProgress>>>,
    pub indexing_errors: Arc<RwLock<Vec<String>>>,
    /// Used to signal the indexing task to stop
    pub stop_indexing: Arc<RwLock<bool>>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let db = Database::new().await?;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            indexing_active: Arc::new(RwLock::new(false)),
            indexing_progress: Arc::new(RwLock::new(None)),
            indexing_errors: Arc::new(RwLock::new(Vec::new())),
            stop_indexing: Arc::new(RwLock::new(false)),
        })
    }

    /// Reset indexing state for a new indexing session
    pub async fn reset_indexing_state(&self) {
        *self.indexing_active.write().await = false;
        *self.indexing_progress.write().await = None;
        *self.indexing_errors.write().await = Vec::new();
        *self.stop_indexing.write().await = false;
    }
}
