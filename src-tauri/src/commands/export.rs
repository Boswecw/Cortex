use crate::export::{
    BundleBuilder, ExportConfig, ExportPreview, ExportResult, RakeExportConfig, RakeExportMode,
    RakeExportMetadata, RakeExporter,
};
use crate::error::{CortexError, Result};
use crate::state::AppState;
use tauri::State;

/// Export VS Code Claude context bundle
#[tauri::command]
pub async fn export_vscode_context(
    collection_id: Option<String>,
    include_embeddings: bool,
    include_prompts: bool,
    output_path: String,
    project_name: Option<String>,
    custom_context: Option<String>,
    state: State<'_, AppState>,
) -> Result<ExportResult> {
    let config = ExportConfig {
        collection_id,
        include_embeddings,
        output_path,
        include_prompts,
        project_name,
        custom_context,
    };

    // Clone the database Arc to move into the blocking task
    let db_arc = state.db.clone();

    // Run export in blocking task since Database is not Send
    let result = tokio::task::spawn_blocking(move || {
        let db_guard = db_arc.lock().unwrap();
        let db = db_guard.clone();
        drop(db_guard);

        let bundler = BundleBuilder::new(db);
        // Call the sync version since we're in a blocking context
        tokio::runtime::Handle::current().block_on(bundler.create_bundle(&config))
    })
    .await
    .map_err(|e| CortexError::Internal {
        message: format!("Export task failed: {}", e),
    })??;

    Ok(result)
}

/// Export Rake-compatible package
#[tauri::command]
pub async fn export_rake_package(
    collection_id: Option<String>,
    tenant_id: String,
    output_path: String,
    include_embeddings: bool,
    export_mode: String, // "full", "incremental", or "collection"
    state: State<'_, AppState>,
) -> Result<String> {
    // Validate tenant_id
    if tenant_id.trim().is_empty() {
        return Err(CortexError::InvalidQuery {
            query: tenant_id,
            reason: "Tenant ID cannot be empty".to_string(),
        });
    }

    // Parse export mode
    let mode = match export_mode.to_lowercase().as_str() {
        "full" => RakeExportMode::Full,
        "incremental" => RakeExportMode::Incremental,
        "collection" => RakeExportMode::Collection,
        _ => {
            return Err(CortexError::InvalidQuery {
                query: export_mode,
                reason: "Invalid export mode. Use 'full', 'incremental', or 'collection'".to_string(),
            });
        }
    };

    let config = RakeExportConfig {
        collection_id,
        tenant_id,
        output_path,
        include_embeddings,
        export_mode: mode,
    };

    // Clone the database Arc to move into the blocking task
    let db_arc = state.db.clone();

    // Run export in blocking task since Database is not Send
    let file_path = tokio::task::spawn_blocking(move || {
        let db_guard = db_arc.lock().unwrap();
        let db = db_guard.clone();
        drop(db_guard);

        let exporter = RakeExporter::new(db);
        tokio::runtime::Handle::current().block_on(exporter.export_to_file(&config))
    })
    .await
    .map_err(|e| CortexError::Internal {
        message: format!("Export task failed: {}", e),
    })??;

    Ok(file_path)
}

/// Get preview of what will be exported
#[tauri::command]
pub async fn get_export_preview(
    _collection_id: Option<String>,
    include_embeddings: bool,
    state: State<'_, AppState>,
) -> Result<ExportPreview> {
    // Lock the database and perform all operations while holding the lock
    let db_guard = state.db.lock().unwrap();
    let conn = db_guard.get_connection();

    // Get files that would be exported
    let files = crate::db::operations::list_files(conn, 10000, 0)?;

    let file_count = files.len();
    let total_size: i64 = files.iter().map(|f| f.size).sum();

    // Estimate chunk count
    let chunk_count: usize = files
        .iter()
        .filter_map(|f| {
            crate::db::operations::get_file_content(conn, f.id)
                .ok()
                .flatten()
                .and_then(|fc| fc.word_count)
                .map(|wc| ((wc as f32 * 0.75) / 375.0).ceil() as usize)
        })
        .sum();

    // Check embeddings
    let embedded_file_count = if include_embeddings {
        crate::db::operations::count_embeddings(conn)? as usize
    } else {
        0
    };

    let has_embeddings = embedded_file_count > 0;
    let estimated_size_human = crate::export::format_file_size(total_size);

    Ok(ExportPreview {
        file_count,
        chunk_count,
        estimated_size: total_size,
        estimated_size_human,
        has_embeddings,
        embedded_file_count,
    })
}

/// Get Rake export preview
#[tauri::command]
pub async fn get_rake_export_preview(
    collection_id: Option<String>,
    tenant_id: String,
    include_embeddings: bool,
    export_mode: String,
    state: State<'_, AppState>,
) -> Result<RakeExportMetadata> {
    // Parse export mode
    let mode = match export_mode.to_lowercase().as_str() {
        "full" => RakeExportMode::Full,
        "incremental" => RakeExportMode::Incremental,
        "collection" => RakeExportMode::Collection,
        _ => RakeExportMode::Full,
    };

    let config = RakeExportConfig {
        collection_id,
        tenant_id,
        output_path: String::new(), // Not needed for preview
        include_embeddings,
        export_mode: mode,
    };

    // Clone database from Arc<Mutex<Database>> and perform sync operation
    let preview = {
        let db_guard = state.db.lock().unwrap();
        let db = db_guard.clone();
        drop(db_guard); // Release lock before async operations

        let exporter = RakeExporter::new(db);
        exporter.preview_sync(&config)?
    };

    Ok(preview)
}

/// List available prompt templates
#[tauri::command]
pub async fn list_prompt_templates() -> Result<Vec<PromptTemplateInfo>> {
    // Return predefined templates
    Ok(vec![
        PromptTemplateInfo {
            id: "add_feature".to_string(),
            name: "Add Feature".to_string(),
            short_description: "Implement a new feature".to_string(),
            description: "Guided prompt for implementing new features following project patterns".to_string(),
            icon: "✨".to_string(),
            category: "feature".to_string(),
        },
        PromptTemplateInfo {
            id: "fix_bug".to_string(),
            name: "Fix Bug".to_string(),
            short_description: "Debug and fix issues".to_string(),
            description: "Systematic approach to identifying and fixing bugs".to_string(),
            icon: "🐛".to_string(),
            category: "bug_fix".to_string(),
        },
        PromptTemplateInfo {
            id: "refactor".to_string(),
            name: "Refactor Code".to_string(),
            short_description: "Improve code quality".to_string(),
            description: "Refactor code while maintaining functionality".to_string(),
            icon: "♻️".to_string(),
            category: "refactor".to_string(),
        },
        PromptTemplateInfo {
            id: "add_tests".to_string(),
            name: "Add Tests".to_string(),
            short_description: "Create test coverage".to_string(),
            description: "Add comprehensive unit and integration tests".to_string(),
            icon: "🧪".to_string(),
            category: "testing".to_string(),
        },
        PromptTemplateInfo {
            id: "documentation".to_string(),
            name: "Documentation".to_string(),
            short_description: "Update documentation".to_string(),
            description: "Create or update project documentation".to_string(),
            icon: "📚".to_string(),
            category: "documentation".to_string(),
        },
    ])
}

/// Prompt template info (simplified version for IPC)
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct PromptTemplateInfo {
    pub id: String,
    pub name: String,
    pub short_description: String,
    pub description: String,
    pub icon: String,
    pub category: String,
}

/// Get database statistics for export preview
#[tauri::command]
pub async fn get_export_stats(state: State<'_, AppState>) -> Result<ExportStatsInfo> {
    // Lock the database and perform all operations while holding the lock
    let db_guard = state.db.lock().unwrap();
    let conn = db_guard.get_connection();

    let (total_files, indexed_files, total_size) = crate::db::operations::get_db_stats(conn)?;
    let embedding_count = crate::db::operations::count_embeddings(conn)?;

    Ok(ExportStatsInfo {
        total_files: total_files as usize,
        indexed_files: indexed_files as usize,
        total_size,
        total_size_human: crate::export::format_file_size(total_size),
        embedded_files: embedding_count as usize,
    })
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ExportStatsInfo {
    pub total_files: usize,
    pub indexed_files: usize,
    pub total_size: i64,
    pub total_size_human: String,
    pub embedded_files: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_templates_list() {
        // Test that we have at least the 5 core templates
        let templates = tokio_test::block_on(list_prompt_templates()).unwrap();
        assert_eq!(templates.len(), 5);

        let feature_template = templates.iter().find(|t| t.id == "add_feature");
        assert!(feature_template.is_some());
        assert_eq!(feature_template.unwrap().name, "Add Feature");
    }
}
