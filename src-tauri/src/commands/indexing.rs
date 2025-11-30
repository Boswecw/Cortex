use crate::db::{insert_file, upsert_file_content};
use crate::error::CortexError;
use crate::indexer::{ContentExtractor, FileScanner};
use crate::state::AppState;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatus {
    pub is_active: bool,
    pub total_files: usize,
    pub indexed_files: usize,
    pub current_file: Option<String>,
    pub errors: Vec<String>,
    pub progress_percentage: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexProgressEvent {
    pub total_files: usize,
    pub indexed_files: usize,
    pub current_file: String,
    pub progress_percentage: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexCompleteEvent {
    pub total_files: usize,
    pub indexed_files: usize,
    pub errors: Vec<String>,
    pub duration_secs: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexErrorEvent {
    pub file_path: String,
    pub error: String,
}

/// Start indexing one or more directories
#[tauri::command]
pub async fn start_indexing(
    paths: Vec<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Check if already indexing
    {
        let indexing_active = state.indexing_active.read().await;
        if *indexing_active {
            return Err(CortexError::IndexingInProgress.to_string());
        }
    }

    // Reset and start indexing
    state.reset_indexing_state().await;
    *state.indexing_active.write().await = true;

    log::info!("Starting indexing for {} paths", paths.len());

    // Spawn background task for indexing
    let db = state.db.clone();
    let indexing_active = state.indexing_active.clone();
    let indexing_progress = state.indexing_progress.clone();
    let indexing_errors = state.indexing_errors.clone();
    let stop_indexing = state.stop_indexing.clone();

    let state_clone = Arc::new(crate::state::AppState {
        db,
        indexing_active,
        indexing_progress,
        indexing_errors,
        stop_indexing,
    });
    let app_clone = app.clone();

    tokio::spawn(async move {
        let start_time = std::time::Instant::now();

        // Clone for use after the match
        let app_for_emit = app_clone.clone();
        let state_for_emit = state_clone.clone();

        // Run the indexing pipeline
        match run_indexing_pipeline(paths, app_clone, state_clone).await {
            Ok(stats) => {
                let duration = start_time.elapsed();
                log::info!(
                    "Indexing complete: {} files indexed in {:.2}s",
                    stats.indexed_files,
                    duration.as_secs_f64()
                );

                // Emit completion event
                let errors = state_for_emit.indexing_errors.read().await.clone();
                let _ = app_for_emit.emit(
                    "indexing:complete",
                    IndexCompleteEvent {
                        total_files: stats.total_files,
                        indexed_files: stats.indexed_files,
                        errors,
                        duration_secs: duration.as_secs_f64(),
                    },
                );
            }
            Err(e) => {
                log::error!("Indexing failed: {}", e);
                state_for_emit
                    .indexing_errors
                    .write()
                    .await
                    .push(format!("Fatal error: {}", e));
            }
        }

        // Mark indexing as complete
        *state_for_emit.indexing_active.write().await = false;
    });

    Ok("Indexing started in background".to_string())
}

/// Stop ongoing indexing operation
#[tauri::command]
pub async fn stop_indexing(state: State<'_, AppState>) -> Result<String, String> {
    let indexing_active = state.indexing_active.read().await;

    if !*indexing_active {
        return Err("No indexing operation in progress".to_string());
    }

    log::info!("Stopping indexing operation...");
    *state.stop_indexing.write().await = true;

    Ok("Indexing stop signal sent".to_string())
}

/// Get current indexing status
#[tauri::command]
pub async fn get_index_status(state: State<'_, AppState>) -> Result<IndexStatus, String> {
    let indexing_active = *state.indexing_active.read().await;
    let progress = state.indexing_progress.read().await.clone();
    let errors = state.indexing_errors.read().await.clone();

    let (total_files, indexed_files, current_file, progress_percentage) = match progress {
        Some(p) => (
            p.total_files,
            p.current_file,
            Some(format!("File {} of {}", p.current_file, p.total_files)),
            p.percentage(),
        ),
        None => (0, 0, None, 0.0),
    };

    Ok(IndexStatus {
        is_active: indexing_active,
        total_files,
        indexed_files,
        current_file,
        errors,
        progress_percentage,
    })
}

// Private helper types
struct IndexingStats {
    total_files: usize,
    indexed_files: usize,
}

/// Run the complete indexing pipeline
async fn run_indexing_pipeline(
    paths: Vec<String>,
    app: AppHandle,
    state: Arc<AppState>,
) -> Result<IndexingStats, CortexError> {
    let scanner = FileScanner::new();
    let mut all_jobs = Vec::new();

    // Step 1: Scan all directories
    log::info!("Scanning {} directories...", paths.len());
    for path_str in paths {
        let path = PathBuf::from(&path_str);

        if !path.exists() {
            let error = format!("Path does not exist: {}", path_str);
            log::warn!("{}", error);
            state.indexing_errors.write().await.push(error);
            continue;
        }

        match scanner.scan_directory(&path) {
            Ok(jobs) => {
                log::info!("Found {} files in {}", jobs.len(), path_str);
                all_jobs.extend(jobs);
            }
            Err(e) => {
                let error = format!("Failed to scan {}: {}", path_str, e);
                log::warn!("{}", error);
                state.indexing_errors.write().await.push(error);
            }
        }

        // Check for stop signal
        if *state.stop_indexing.read().await {
            log::info!("Indexing stopped during scan phase");
            return Ok(IndexingStats {
                total_files: all_jobs.len(),
                indexed_files: 0,
            });
        }
    }

    let total_files = all_jobs.len();
    log::info!("Total files to index: {}", total_files);

    // Update progress
    {
        let mut progress = state.indexing_progress.write().await;
        *progress = Some(crate::indexer::ScanProgress {
            total_files,
            current_file: 0,
        });
    }

    // Step 2: Extract and index each file
    let mut indexed_count = 0;

    for (idx, job) in all_jobs.iter().enumerate() {
        // Check for stop signal
        if *state.stop_indexing.read().await {
            log::info!("Indexing stopped at file {}/{}", idx + 1, total_files);
            break;
        }

        let current_file = idx + 1;

        // Update progress
        {
            let mut progress = state.indexing_progress.write().await;
            *progress = Some(crate::indexer::ScanProgress {
                total_files,
                current_file,
            });
        }

        // Emit progress event every 10 files or on first/last file
        if idx == 0 || idx == total_files - 1 || idx % 10 == 0 {
            let _ = app.emit(
                "indexing:progress",
                IndexProgressEvent {
                    total_files,
                    indexed_files: indexed_count,
                    current_file: job.path.display().to_string(),
                    progress_percentage: (current_file as f64 / total_files as f64) * 100.0,
                },
            );
        }

        // Extract content
        let extracted = match ContentExtractor::extract(&job.path) {
            Ok(content) => content,
            Err(e) => {
                let error = format!("Failed to extract {}: {}", job.path.display(), e);
                log::warn!("{}", error);
                state.indexing_errors.write().await.push(error.clone());

                // Emit error event
                let _ = app.emit(
                    "indexing:error",
                    IndexErrorEvent {
                        file_path: job.path.display().to_string(),
                        error: e.to_string(),
                    },
                );

                continue;
            }
        };

        // Log any extraction warnings
        for warning in &extracted.warnings {
            log::debug!("Extraction warning for {}: {}", job.path.display(), warning);
        }

        // Prepare database data
        let now = Utc::now().to_rfc3339();
        let file_type = job
            .path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown");

        let filename = job
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let root_path = job
            .path
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("");

        // Acquire database lock only for the insert operation
        // Scope the lock to avoid holding it across await points
        let insert_result = {
            let db = state.db.lock().unwrap();
            let conn = db.get_connection();

            match insert_file(
                conn,
                &job.path.to_string_lossy(),
                filename,
                file_type,
                job.size as i64,
                &now,
                &now,
                None,
                root_path,
            ) {
                Ok(file_id) => {
                    // Insert content
                    match upsert_file_content(
                        conn,
                        file_id,
                        Some(&extracted.text),
                        extracted.summary.as_deref(),
                    ) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(format!("Failed to insert content for {}: {}", filename, e)),
                    }
                }
                Err(e) => Err(format!("Failed to insert file {}: {}", filename, e)),
            }
        }; // db and conn are dropped here

        // Handle result with async operations outside the database lock
        match insert_result {
            Ok(()) => {
                indexed_count += 1;
            }
            Err(error) => {
                log::warn!("{}", error);
                state.indexing_errors.write().await.push(error);
            }
        }
    }

    Ok(IndexingStats {
        total_files,
        indexed_files: indexed_count,
    })
}

use std::sync::Arc;
