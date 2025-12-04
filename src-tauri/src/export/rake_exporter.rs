use crate::db::{Database, File, FileContent};
use crate::error::{CortexError, Result};
use crate::export::{
    PathValidator, RakeChunk, RakeChunkMetadata, RakeExportConfig, RakeExportMetadata, RakeExportPackage,
};
use chrono::{DateTime, Utc};
use std::fs;

/// Exports indexed content in Rake-compatible format
pub struct RakeExporter {
    db: Database,
}

impl RakeExporter {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Export to Rake-compatible JSON format
    pub async fn export(&self, config: &RakeExportConfig) -> Result<RakeExportPackage> {
        let conn = self.db.get_connection();

        // Get files to export
        let files = self.get_files_for_export(config)?;

        // Convert files to chunks
        let mut chunks = Vec::new();
        for file in &files {
            let file_chunks = self.convert_file_to_chunks(file, config).await?;
            chunks.extend(file_chunks);
        }

        // Build metadata
        let metadata = RakeExportMetadata {
            cortex_version: env!("CARGO_PKG_VERSION").to_string(),
            collection_id: config.collection_id.clone(),
            collection_name: None, // TODO: Get from collections when implemented
            total_files: files.len(),
            total_chunks: chunks.len(),
            has_embeddings: config.include_embeddings,
            embedding_model: if config.include_embeddings {
                Some("all-MiniLM-L6-v2".to_string()) // Cortex's default model
            } else {
                None
            },
            export_mode: format!("{:?}", config.export_mode).to_lowercase(),
        };

        Ok(RakeExportPackage {
            version: "1.0".to_string(),
            source: "cortex_local".to_string(),
            tenant_id: config.tenant_id.clone(),
            export_timestamp: Utc::now(),
            metadata,
            chunks,
        })
    }

    /// Write export package to JSON file
    pub async fn export_to_file(&self, config: &RakeExportConfig) -> Result<String> {
        // Validate and sanitize output path (security)
        let validated_path = PathValidator::validate_export_path(&config.output_path)?;

        let package = self.export(config).await?;

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&package).map_err(|e| CortexError::Internal {
            message: format!("Failed to serialize export package: {}", e),
        })?;

        // Write to file (using validated path)
        fs::write(&validated_path, json).map_err(|e| CortexError::Internal {
            message: format!("Failed to write export file: {}", e),
        })?;

        Ok(validated_path.to_string_lossy().to_string())
    }

    /// Get files to export based on configuration
    fn get_files_for_export(&self, config: &RakeExportConfig) -> Result<Vec<File>> {
        let conn = self.db.get_connection();

        // For now, get all files (collection support will be added in Phase 3)
        // In incremental mode, we would filter by last sync time
        let files = crate::db::operations::list_files(conn, 10000, 0)?;

        Ok(files)
    }

    /// Convert a single file to Rake chunks
    async fn convert_file_to_chunks(
        &self,
        file: &File,
        config: &RakeExportConfig,
    ) -> Result<Vec<RakeChunk>> {
        let conn = self.db.get_connection();

        // Get file content
        let content = crate::db::operations::get_file_content(conn, file.id)?;

        let text_content = match content {
            Some(fc) => fc.text_content,
            None => None,
        };

        if text_content.is_none() {
            // Skip files without content
            return Ok(Vec::new());
        }

        let text = text_content.unwrap();

        // Chunk the text (Rake uses ~500 token chunks)
        let chunks = self.chunk_text(&text, 500);

        let mut rake_chunks = Vec::new();

        for (index, chunk_content) in chunks.into_iter().enumerate() {
            // Get embedding if requested
            let embedding = if config.include_embeddings {
                self.get_chunk_embedding(file.id).await?
            } else {
                None
            };

            let chunk_id = format!("{}-chunk-{}", file.id, index);
            let token_count = self.estimate_tokens(&chunk_content);

            // Parse modified_at as DateTime
            let modified_at = DateTime::parse_from_rfc3339(&file.modified_at)
                .unwrap_or_else(|_| DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z").unwrap())
                .with_timezone(&Utc);

            rake_chunks.push(RakeChunk {
                id: chunk_id,
                document_id: file.id.to_string(),
                content: chunk_content,
                embedding,
                position: index as u32,
                token_count,
                metadata: RakeChunkMetadata {
                    file_path: file.path.clone(),
                    file_type: file.file_type.clone(),
                    file_name: file.filename.clone(),
                    modified_at,
                    collection_id: config.collection_id.clone(),
                    collection_name: None,
                },
            });
        }

        Ok(rake_chunks)
    }

    /// Chunk text into segments (simple word-based chunking)
    fn chunk_text(&self, text: &str, target_tokens: usize) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut chunks = Vec::new();

        // Approximate: 1 token ≈ 0.75 words
        let words_per_chunk = (target_tokens as f32 * 0.75) as usize;

        let mut current_chunk = Vec::new();

        for word in words {
            current_chunk.push(word);

            if current_chunk.len() >= words_per_chunk {
                chunks.push(current_chunk.join(" "));
                current_chunk.clear();
            }
        }

        // Add remaining words
        if !current_chunk.is_empty() {
            chunks.push(current_chunk.join(" "));
        }

        // Return at least one chunk
        if chunks.is_empty() {
            chunks.push(text.to_string());
        }

        chunks
    }

    /// Get embedding for a chunk (file-level for now)
    async fn get_chunk_embedding(&self, file_id: i64) -> Result<Option<Vec<f32>>> {
        let conn = self.db.get_connection();

        match crate::db::operations::get_embedding(conn, file_id)? {
            Some(emb) => Ok(Some(emb.embedding)),
            None => Ok(None),
        }
    }

    /// Estimate token count for text
    fn estimate_tokens(&self, text: &str) -> u32 {
        // Rough estimate: 1 token ≈ 4 characters
        (text.len() / 4) as u32
    }

    /// Get export preview (before actual export)
    pub fn preview_sync(&self, config: &RakeExportConfig) -> Result<RakeExportMetadata> {
        let files = self.get_files_for_export(config)?;
        let conn = self.db.get_connection();

        // Estimate total chunks (assuming ~500 tokens per chunk, ~3 chars per word)
        let total_chunks: usize = files
            .iter()
            .filter_map(|f| {
                crate::db::operations::get_file_content(conn, f.id)
                    .ok()
                    .flatten()
                    .and_then(|fc| fc.word_count)
                    .map(|wc| ((wc as f32 * 0.75) / 375.0).ceil() as usize) // ~375 words per chunk
            })
            .sum();

        Ok(RakeExportMetadata {
            cortex_version: env!("CARGO_PKG_VERSION").to_string(),
            collection_id: config.collection_id.clone(),
            collection_name: None,
            total_files: files.len(),
            total_chunks,
            has_embeddings: config.include_embeddings,
            embedding_model: if config.include_embeddings {
                Some("all-MiniLM-L6-v2".to_string())
            } else {
                None
            },
            export_mode: format!("{:?}", config.export_mode).to_lowercase(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text() {
        let exporter = RakeExporter {
            db: Database::new().await.unwrap(), // This will fail in tests without proper setup
        };

        let text = "This is a test. ".repeat(100); // ~400 words
        let chunks = exporter.chunk_text(&text, 500);

        // Should create ~2 chunks (375 words per chunk)
        assert!(chunks.len() >= 1);
        assert!(chunks.len() <= 3);
    }

    #[test]
    fn test_estimate_tokens() {
        let exporter = RakeExporter {
            db: Database::new().await.unwrap(),
        };

        let text = "Hello world this is a test";
        let tokens = exporter.estimate_tokens(text);

        // ~26 characters / 4 = ~6 tokens
        assert!(tokens >= 5 && tokens <= 7);
    }
}
