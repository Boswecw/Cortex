use crate::db::{get_file_by_id, get_file_content, search_files_fts, SearchResult};
use crate::error::CortexError;
use crate::state::AppState;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub file_type: Option<String>,
    pub min_size: Option<i64>,
    pub max_size: Option<i64>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub results: Vec<SearchResult>,
    pub total: usize,
    pub query_time_ms: u64,
}

/// Search files using FTS5 with optional filters and pagination
#[tauri::command]
pub async fn search_files(
    query: String,
    filters: Option<SearchFilters>,
    limit: Option<usize>,
    offset: Option<usize>,
    state: State<'_, AppState>,
) -> Result<SearchResults, String> {
    let start_time = Instant::now();

    log::info!(
        "Searching for: '{}' with filters: {:?}, limit: {:?}, offset: {:?}",
        query,
        filters,
        limit,
        offset
    );

    // Validate query
    if query.trim().is_empty() {
        return Err(CortexError::InvalidQuery {
            query: query.clone(),
            reason: "Query cannot be empty".to_string(),
        }
        .to_string());
    }

    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let search_limit = limit.unwrap_or(50).min(1000); // Max 1000 results
    let search_offset = offset.unwrap_or(0);

    // If we have filters or offset, use custom query; otherwise use simple FTS search
    let results = if filters.is_some() || search_offset > 0 {
        perform_filtered_search(conn, &query, filters, search_limit, search_offset)
            .map_err(|e| e.to_string())?
    } else {
        search_files_fts(conn, &query, search_limit).map_err(|e| e.to_string())?
    };

    let query_time = start_time.elapsed();

    log::info!(
        "Search completed: {} results in {:.2}ms",
        results.len(),
        query_time.as_secs_f64() * 1000.0
    );

    Ok(SearchResults {
        total: results.len(),
        results,
        query_time_ms: query_time.as_millis() as u64,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDetail {
    pub id: i64,
    pub path: String,
    pub filename: String,
    pub file_type: String,
    pub size: i64,
    pub created_at: String,
    pub modified_at: String,
    pub content_preview: Option<String>,
    pub full_content: Option<String>,
    pub word_count: Option<i64>,
    pub summary: Option<String>,
}

/// Get detailed information about a specific file
#[tauri::command]
pub async fn get_file_detail(
    file_id: i64,
    include_full_content: Option<bool>,
    state: State<'_, AppState>,
) -> Result<FileDetail, String> {
    log::info!("Getting file detail for ID: {}", file_id);

    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    // Get file metadata
    let file = get_file_by_id(conn, file_id)
        .map_err(|e| e.to_string())?;

    // Get file content
    let content = get_file_content(conn, file_id).map_err(|e| e.to_string())?;

    let (content_preview, full_content, word_count, summary) = match content {
        Some(c) => {
            let preview = c.text_content.as_ref().map(|text| {
                let preview_len = 500.min(text.len());
                format!("{}...", &text[..preview_len])
            });

            let full = if include_full_content.unwrap_or(false) {
                c.text_content
            } else {
                None
            };

            (preview, full, c.word_count, c.summary)
        }
        None => (None, None, None, None),
    };

    Ok(FileDetail {
        id: file.id,
        path: file.path,
        filename: file.filename,
        file_type: file.file_type,
        size: file.size,
        created_at: file.created_at,
        modified_at: file.modified_at,
        content_preview,
        full_content,
        word_count,
        summary,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    pub total_files: i64,
    pub indexed_files: i64,
    pub total_size_bytes: i64,
}

/// Get search/indexing statistics
#[tauri::command]
pub async fn get_search_stats(state: State<'_, AppState>) -> Result<SearchStats, String> {
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let (total_files, indexed_files, total_size) =
        crate::db::get_db_stats(conn).map_err(|e| e.to_string())?;

    Ok(SearchStats {
        total_files,
        indexed_files,
        total_size_bytes: total_size,
    })
}

// Private helper function for filtered search
fn perform_filtered_search(
    conn: &Connection,
    query: &str,
    filters: Option<SearchFilters>,
    limit: usize,
    offset: usize,
) -> Result<Vec<SearchResult>, CortexError> {
    let mut where_clauses = vec!["files_fts MATCH ?1", "f.is_deleted = 0"];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(query.to_string())];

    if let Some(f) = filters {
        if let Some(file_type) = f.file_type {
            where_clauses.push("f.file_type = ?");
            params.push(Box::new(file_type));
        }

        if let Some(min_size) = f.min_size {
            where_clauses.push("f.size >= ?");
            params.push(Box::new(min_size));
        }

        if let Some(max_size) = f.max_size {
            where_clauses.push("f.size <= ?");
            params.push(Box::new(max_size));
        }

        if let Some(date_from) = f.date_from {
            where_clauses.push("f.modified_at >= ?");
            params.push(Box::new(date_from));
        }

        if let Some(date_to) = f.date_to {
            where_clauses.push("f.modified_at <= ?");
            params.push(Box::new(date_to));
        }
    }

    params.push(Box::new(limit));
    params.push(Box::new(offset));

    let sql = format!(
        "SELECT f.id, f.path, f.filename,
                snippet(files_fts, 1, '<mark>', '</mark>', '...', 32) as snippet,
                rank
         FROM files_fts
         INNER JOIN files f ON files_fts.rowid = f.id
         WHERE {}
         ORDER BY rank
         LIMIT ? OFFSET ?",
        where_clauses.join(" AND ")
    );

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;

    let results = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(SearchResult {
                file_id: row.get(0)?,
                path: row.get(1)?,
                filename: row.get(2)?,
                snippet: row.get(3)?,
                score: row.get(4)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(results)
}
