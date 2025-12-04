pub mod context_builder;
pub mod prompt_builder;
pub mod bundler;
pub mod rake_exporter;
pub mod path_validator;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// Re-exports for convenience
pub use context_builder::ContextBuilder;
pub use prompt_builder::PromptBuilder;
pub use bundler::BundleBuilder;
pub use rake_exporter::RakeExporter;
pub use path_validator::PathValidator;

/// Configuration for exporting context to VS Code Claude
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Optional collection ID to export (None = entire project)
    pub collection_id: Option<String>,

    /// Whether to include embeddings in the export
    pub include_embeddings: bool,

    /// Output directory path
    pub output_path: String,

    /// Whether to include prompt templates
    pub include_prompts: bool,

    /// Project name (auto-detected if None)
    pub project_name: Option<String>,

    /// Additional context to include
    pub custom_context: Option<String>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            collection_id: None,
            include_embeddings: false,
            output_path: ".cortex-export".to_string(),
            include_prompts: true,
            project_name: None,
            custom_context: None,
        }
    }
}

/// Result of a VS Code context export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    /// Path to the generated CONTEXT.md file
    pub context_file: String,

    /// Path to the generated STARTER_PROMPT.md file
    pub starter_prompt_file: String,

    /// Paths to generated prompt template files
    pub prompt_files: Vec<String>,

    /// Export statistics
    pub stats: ExportStats,

    /// Timestamp of export
    pub exported_at: DateTime<Utc>,
}

/// Statistics about the export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStats {
    /// Total number of files included
    pub total_files: usize,

    /// Total number of chunks generated
    pub total_chunks: usize,

    /// Total size in bytes
    pub total_size_bytes: i64,

    /// Number of files with embeddings
    pub files_with_embeddings: usize,

    /// Number of prompts generated
    pub prompts_generated: usize,
}

/// Export statistics summary for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStatsInfo {
    /// Total number of files in database
    pub total_files: usize,

    /// Number of files with indexed content
    pub indexed_files: usize,

    /// Total size in bytes
    pub total_size: i64,

    /// Human-readable size string
    pub total_size_human: String,

    /// Number of files with embeddings
    pub embedded_files: usize,
}

/// Configuration for Rake export package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RakeExportConfig {
    /// Optional collection ID to export (None = entire project)
    pub collection_id: Option<String>,

    /// Tenant ID for Rake multi-tenancy
    pub tenant_id: String,

    /// Output file path (JSON)
    pub output_path: String,

    /// Whether to include pre-computed embeddings
    pub include_embeddings: bool,

    /// Export mode: full, incremental, or collection
    pub export_mode: RakeExportMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RakeExportMode {
    /// Export all indexed files
    Full,

    /// Export only changes since last sync
    Incremental,

    /// Export a specific collection
    Collection,
}

/// Rake export package (compatible with Rake V1 format)
#[derive(Debug, Serialize, Deserialize)]
pub struct RakeExportPackage {
    /// Export format version
    pub version: String,

    /// Source identifier
    pub source: String,

    /// Tenant ID for multi-tenancy
    pub tenant_id: String,

    /// Export timestamp
    pub export_timestamp: DateTime<Utc>,

    /// Export metadata
    pub metadata: RakeExportMetadata,

    /// Pre-chunked and optionally embedded content
    pub chunks: Vec<RakeChunk>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RakeExportMetadata {
    /// Cortex version that generated the export
    pub cortex_version: String,

    /// Optional collection ID
    pub collection_id: Option<String>,

    /// Optional collection name
    pub collection_name: Option<String>,

    /// Total number of files exported
    pub total_files: usize,

    /// Total number of chunks
    pub total_chunks: usize,

    /// Whether embeddings are included
    pub has_embeddings: bool,

    /// Embedding model used (if applicable)
    pub embedding_model: Option<String>,

    /// Export mode used
    pub export_mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RakeChunk {
    /// Unique chunk ID
    pub id: String,

    /// Document/file ID
    pub document_id: String,

    /// Chunk content (text)
    pub content: String,

    /// Pre-computed embedding (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,

    /// Position within the document (0-indexed)
    pub position: u32,

    /// Approximate token count
    pub token_count: u32,

    /// Chunk metadata
    pub metadata: RakeChunkMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RakeChunkMetadata {
    /// Full file path
    pub file_path: String,

    /// File type/extension
    pub file_type: String,

    /// File name
    pub file_name: String,

    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,

    /// Optional collection ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<String>,

    /// Optional collection name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_name: Option<String>,
}

/// Preview of what will be exported (before actual export)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPreview {
    /// Number of files to be exported
    pub file_count: usize,

    /// Number of chunks to be exported
    pub chunk_count: usize,

    /// Estimated size in bytes
    pub estimated_size: i64,

    /// Estimated size as human-readable string
    pub estimated_size_human: String,

    /// Whether embeddings are available
    pub has_embeddings: bool,

    /// Number of files with embeddings
    pub embedded_file_count: usize,
}

/// Prompt template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// Unique template ID
    pub id: String,

    /// Display name
    pub name: String,

    /// Short description
    pub short_description: String,

    /// Full description
    pub description: String,

    /// Icon/emoji for UI
    pub icon: String,

    /// Template category
    pub category: PromptCategory,

    /// Variables that can be customized
    pub variables: Vec<PromptVariable>,

    /// Template file path (Handlebars .hbs)
    pub template_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PromptCategory {
    /// General starter prompts
    Starter,

    /// Feature implementation
    Feature,

    /// Bug fixes
    BugFix,

    /// Code refactoring
    Refactor,

    /// Testing
    Testing,

    /// Documentation
    Documentation,

    /// Integration (e.g., Rake)
    Integration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVariable {
    /// Variable key (e.g., "feature_name")
    pub key: String,

    /// Display label
    pub label: String,

    /// Placeholder text
    pub placeholder: String,

    /// Whether this variable is required
    pub required: bool,

    /// Default value (if any)
    pub default: Option<String>,
}

/// Helper function to format file size as human-readable string
pub fn format_file_size(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500.00 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1536), "1.50 KB");
        assert_eq!(format_file_size(1048576), "1.00 MB");
        assert_eq!(format_file_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_export_config_default() {
        let config = ExportConfig::default();
        assert_eq!(config.output_path, ".cortex-export");
        assert_eq!(config.include_embeddings, false);
        assert_eq!(config.include_prompts, true);
        assert!(config.collection_id.is_none());
    }
}
