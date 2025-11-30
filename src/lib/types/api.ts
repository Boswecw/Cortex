// TypeScript type definitions matching Rust backend API

// Indexing Types

export interface IndexStatus {
  is_indexing: boolean;
  progress: ScanProgress | null;
  errors: string[];
}

export interface ScanProgress {
  total_files: number;
  files_indexed: number;
  current_file: string;
  percentage: number;
}

export interface IndexProgressEvent {
  total_files: number;
  indexed_files: number;
  current_file: string;
  progress_percentage: number;
}

export interface IndexCompleteEvent {
  total_files: number;
  indexed_files: number;
  failed_files: number;
  duration_seconds: number;
  errors: string[];
}

export interface IndexErrorEvent {
  file_path: string;
  error_message: string;
}

// Search Types

export interface SearchFilters {
  file_type?: string;
  min_size?: number;
  max_size?: number;
  date_from?: string;  // ISO 8601
  date_to?: string;    // ISO 8601
}

export interface SearchResults {
  results: SearchResult[];
  total: number;
  query_time_ms: number;
}

export interface SearchResult {
  file_id: number;
  path: string;
  filename: string;
  snippet: string;  // HTML with <mark> tags
  score: number;
}

export interface FileDetail {
  id: number;
  path: string;
  filename: string;
  file_type: string;
  size: number;
  created_at: string;  // ISO 8601
  modified_at: string; // ISO 8601
  content_preview: string | null;
  full_content: string | null;
  word_count: number | null;
  summary: string | null;
}

export interface SearchStats {
  total_files: number;
  indexed_files: number;
  total_size_bytes: number;
}

// File type definitions for UI

export const FILE_TYPES = {
  txt: { label: 'Text', color: 'text-blue-400' },
  md: { label: 'Markdown', color: 'text-purple-400' },
  pdf: { label: 'PDF', color: 'text-red-400' },
  docx: { label: 'Word', color: 'text-blue-600' },
  rs: { label: 'Rust', color: 'text-orange-400' },
  js: { label: 'JavaScript', color: 'text-yellow-400' },
  ts: { label: 'TypeScript', color: 'text-blue-500' },
  jsx: { label: 'React', color: 'text-cyan-400' },
  tsx: { label: 'React TS', color: 'text-cyan-500' },
  py: { label: 'Python', color: 'text-green-400' },
  java: { label: 'Java', color: 'text-red-500' },
  cpp: { label: 'C++', color: 'text-blue-300' },
  c: { label: 'C', color: 'text-blue-300' },
  h: { label: 'Header', color: 'text-gray-400' },
  json: { label: 'JSON', color: 'text-yellow-300' },
  yaml: { label: 'YAML', color: 'text-pink-400' },
  yml: { label: 'YAML', color: 'text-pink-400' },
  toml: { label: 'TOML', color: 'text-orange-300' },
  xml: { label: 'XML', color: 'text-green-300' },
} as const;

export type FileType = keyof typeof FILE_TYPES;

// Utility functions

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';

  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${units[i]}`;
}

export function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds.toFixed(1)}s`;

  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;

  if (minutes < 60) {
    return `${minutes}m ${remainingSeconds.toFixed(0)}s`;
  }

  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;

  return `${hours}h ${remainingMinutes}m`;
}

export function formatDate(isoString: string): string {
  const date = new Date(isoString);
  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
}

export function getFileTypeInfo(fileType: string) {
  return FILE_TYPES[fileType as FileType] || { label: fileType.toUpperCase(), color: 'text-gray-400' };
}

// ============================================================================
// AI Types (Phase 2: AI Features)
// ============================================================================

export interface EmbeddingStatus {
  total_files: number;
  files_with_embeddings: number;
  files_without_embeddings: number;
  model_downloaded: boolean;
  model_version: string;
}

export interface SemanticSearchResult {
  file_id: number;
  path: string;
  filename: string;
  file_type: string;
  similarity_score: number;
}

export interface SemanticSearchFilters {
  query: string;
  limit?: number;
  threshold?: number; // 0.0 - 1.0, default 0.7
}

export interface SimilarFilesParams {
  file_id: number;
  limit?: number;
  threshold?: number;
}
