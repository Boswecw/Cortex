//! AI Commands for Tauri
//!
//! Provides Tauri commands for AI features:
//! - Embedding generation
//! - Semantic search
//! - Similarity finding

use crate::ai::{ensure_model_downloaded, is_model_downloaded, EmbeddingConfig, EmbeddingService};
use crate::db::operations::{
    count_embeddings, get_all_embeddings, get_embedding, get_file_content,
    get_files_by_ids, get_files_without_embeddings, upsert_embedding,
};
use crate::error::{CortexError, Result};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

const MODEL_VERSION: &str = "all-MiniLM-L6-v2";

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingStatus {
    pub total_files: i64,
    pub files_with_embeddings: i64,
    pub files_without_embeddings: i64,
    pub model_downloaded: bool,
    pub model_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SemanticSearchResult {
    pub file_id: i64,
    pub path: String,
    pub filename: String,
    pub file_type: String,
    pub similarity_score: f32,
}

/// Get embedding generation status
#[tauri::command]
pub async fn get_embedding_status(state: State<'_, Arc<AppState>>) -> Result<EmbeddingStatus> {
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let total_files: i64 = conn.query_row(
        "SELECT COUNT(*) FROM files WHERE is_deleted = 0",
        [],
        |row| row.get(0),
    )?;

    let files_with_embeddings = count_embeddings(conn)?;
    let files_without_embeddings = total_files - files_with_embeddings;

    let model_downloaded = is_model_downloaded().unwrap_or(false);

    Ok(EmbeddingStatus {
        total_files,
        files_with_embeddings,
        files_without_embeddings,
        model_downloaded,
        model_version: MODEL_VERSION.to_string(),
    })
}

/// Generate embeddings for specific file IDs
#[tauri::command]
pub async fn generate_embeddings(
    state: State<'_, Arc<AppState>>,
    file_ids: Vec<i64>,
) -> Result<usize> {
    // Ensure model is downloaded
    ensure_model_downloaded()
        .map_err(|e| CortexError::Internal {
            message: format!("Failed to download model: {}", e),
        })?;

    // Create embedding service
    let config = EmbeddingConfig::default();
    let mut service = EmbeddingService::new(config)
        .map_err(|e| CortexError::Internal {
            message: format!("Failed to initialize embedding service: {}", e),
        })?;

    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let mut generated_count = 0;

    for file_id in file_ids {
        // Get file content
        if let Some(content) = get_file_content(conn, file_id)? {
            if let Some(ref text) = content.text_content {
                // Generate embedding
                let embedding = service
                    .embed(text)
                    .map_err(|e| CortexError::Internal {
                        message: format!("Failed to generate embedding for file {}: {}", file_id, e),
                    })?;

                // Store in database
                upsert_embedding(conn, file_id, &embedding, MODEL_VERSION)?;
                generated_count += 1;
            }
        }
    }

    Ok(generated_count)
}

/// Generate embeddings for all files without embeddings (batch processing)
#[tauri::command]
pub async fn generate_all_embeddings(
    state: State<'_, Arc<AppState>>,
    batch_size: Option<usize>,
) -> Result<usize> {
    let batch_size = batch_size.unwrap_or(100);

    // Ensure model is downloaded
    ensure_model_downloaded()
        .map_err(|e| CortexError::Internal {
            message: format!("Failed to download model: {}", e),
        })?;

    // Create embedding service
    let config = EmbeddingConfig::default();
    let mut service = EmbeddingService::new(config)
        .map_err(|e| CortexError::Internal {
            message: format!("Failed to initialize embedding service: {}", e),
        })?;

    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let mut total_generated = 0;

    loop {
        // Get files without embeddings
        let files = get_files_without_embeddings(conn, batch_size)?;

        if files.is_empty() {
            break;
        }

        for file in files {
            // Get file content
            if let Some(content) = get_file_content(conn, file.id)? {
                if let Some(ref text) = content.text_content {
                    // Generate embedding
                    match service.embed(text) {
                        Ok(embedding) => {
                            // Store in database
                            upsert_embedding(conn, file.id, &embedding, MODEL_VERSION)?;
                            total_generated += 1;
                        }
                        Err(e) => {
                            log::warn!("Failed to generate embedding for file {}: {}", file.id, e);
                        }
                    }
                }
            }
        }

        // If we got fewer files than batch size, we're done
        if total_generated < batch_size {
            break;
        }
    }

    Ok(total_generated)
}

/// Semantic search using embeddings
#[tauri::command]
pub async fn semantic_search(
    state: State<'_, Arc<AppState>>,
    query: String,
    limit: Option<usize>,
    threshold: Option<f32>,
) -> Result<Vec<SemanticSearchResult>> {
    let limit = limit.unwrap_or(50);
    let threshold = threshold.unwrap_or(0.7);

    if query.trim().is_empty() {
        return Err(CortexError::Internal {
            message: "Query cannot be empty".to_string(),
        });
    }

    // Ensure model is downloaded
    ensure_model_downloaded()
        .map_err(|e| CortexError::Internal {
            message: format!("Failed to download model: {}", e),
        })?;

    // Create embedding service
    let config = EmbeddingConfig::default();
    let mut service = EmbeddingService::new(config)
        .map_err(|e| CortexError::Internal {
            message: format!("Failed to initialize embedding service: {}", e),
        })?;

    // Generate query embedding
    let query_embedding = service
        .embed(&query)
        .map_err(|e| CortexError::Internal {
            message: format!("Failed to generate query embedding: {}", e),
        })?;

    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    // Get all file embeddings
    let file_embeddings = get_all_embeddings(conn)?;

    // Calculate similarities
    use crate::ai::similarity::find_top_k;
    let scored_files = find_top_k(&query_embedding, &file_embeddings, limit, threshold);

    // Get file details
    let file_ids: Vec<i64> = scored_files.iter().map(|(id, _)| *id).collect();
    let files = get_files_by_ids(conn, &file_ids)?;

    // Create result with scores
    let mut results: Vec<SemanticSearchResult> = scored_files
        .into_iter()
        .filter_map(|(file_id, score)| {
            files.iter().find(|f| f.id == file_id).map(|file| SemanticSearchResult {
                file_id: file.id,
                path: file.path.clone(),
                filename: file.filename.clone(),
                file_type: file.file_type.clone(),
                similarity_score: score,
            })
        })
        .collect();

    // Sort by similarity descending
    results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());

    Ok(results)
}

/// Find similar files to a given file
#[tauri::command]
pub async fn find_similar_files(
    state: State<'_, Arc<AppState>>,
    file_id: i64,
    limit: Option<usize>,
    threshold: Option<f32>,
) -> Result<Vec<SemanticSearchResult>> {
    let limit = limit.unwrap_or(10);
    let threshold = threshold.unwrap_or(0.7);

    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    // Get embedding for the reference file
    let reference_embedding = get_embedding(conn, file_id)?
        .ok_or_else(|| CortexError::Internal {
            message: format!("No embedding found for file {}", file_id),
        })?;

    // Get all other file embeddings
    let all_embeddings = get_all_embeddings(conn)?;

    // Filter out the reference file itself
    let other_embeddings: Vec<(i64, Vec<f32>)> = all_embeddings
        .into_iter()
        .filter(|(id, _)| *id != file_id)
        .collect();

    // Find top K similar files
    use crate::ai::similarity::find_top_k;
    let scored_files = find_top_k(&reference_embedding.embedding, &other_embeddings, limit, threshold);

    // Get file details
    let file_ids: Vec<i64> = scored_files.iter().map(|(id, _)| *id).collect();
    let files = get_files_by_ids(conn, &file_ids)?;

    // Create results
    let results: Vec<SemanticSearchResult> = scored_files
        .into_iter()
        .filter_map(|(file_id, score)| {
            files.iter().find(|f| f.id == file_id).map(|file| SemanticSearchResult {
                file_id: file.id,
                path: file.path.clone(),
                filename: file.filename.clone(),
                file_type: file.file_type.clone(),
                similarity_score: score,
            })
        })
        .collect();

    Ok(results)
}
