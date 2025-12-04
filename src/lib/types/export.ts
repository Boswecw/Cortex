/**
 * Export types for Cortex Export feature
 * Matches Rust backend types from src-tauri/src/export/mod.rs
 */

export interface ExportResult {
  context_file: string;
  starter_prompt_file: string;
  prompt_files: string[];
  stats: ExportStats;
  exported_at: string; // ISO 8601 timestamp
}

export interface ExportStats {
  total_files: number;
  total_chunks: number;
  total_size_bytes: number;
  files_with_embeddings: number;
  prompts_generated: number;
}

export interface ExportPreview {
  file_count: number;
  chunk_count: number;
  estimated_size: number; // bytes
  estimated_size_human: string;
  has_embeddings: boolean;
  embedded_file_count: number;
}

export interface ExportStatsInfo {
  total_files: number;
  indexed_files: number;
  total_size: number;
  total_size_human: string;
  embedded_files: number;
}

export interface RakeExportMetadata {
  cortex_version: string;
  collection_id: string | null;
  collection_name: string | null;
  total_files: number;
  total_chunks: number;
  has_embeddings: boolean;
  embedding_model: string | null;
  export_mode: string;
}

export interface PromptTemplateInfo {
  id: string;
  name: string;
  short_description: string;
  description: string;
  icon: string;
  category: string;
}
