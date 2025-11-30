# Cortex API Reference

**Version:** 0.1.0 (Phase 0)
**Last Updated:** 2025-11-29

---

## Table of Contents

1. [Overview](#overview)
2. [Indexing Commands](#indexing-commands)
3. [Search Commands](#search-commands)
4. [Type Definitions](#type-definitions)
5. [Error Handling](#error-handling)
6. [Events](#events)
7. [Code Examples](#code-examples)

---

## Overview

Cortex exposes 6 Tauri commands to the frontend via the `@tauri-apps/api` invoke function. All commands are asynchronous and return `Promise<T>`.

### Command List

| Command | Module | Description |
|---------|--------|-------------|
| `start_indexing` | indexing | Start indexing directories in background |
| `stop_indexing` | indexing | Stop ongoing indexing gracefully |
| `get_index_status` | indexing | Get current indexing status |
| `search_files` | search | Search files with FTS5 |
| `get_file_detail` | search | Get file metadata and content |
| `get_search_stats` | search | Get indexing statistics |

### Basic Usage

```typescript
import { invoke } from '@tauri-apps/api';

// Example: Search files
const results = await invoke('search_files', {
  query: 'rust programming',
  limit: 50
});
```

---

## Indexing Commands

### `start_indexing`

Start indexing one or more directories in the background.

**Signature:**
```typescript
function start_indexing(
  paths: string[]
): Promise<string>
```

**Parameters:**
- `paths` (string[]) - Array of absolute directory paths to index

**Returns:**
- Promise<string> - Success message: "Indexing started in background"

**Errors:**
- "Indexing is already in progress" - Another indexing operation is running
- "No paths provided" - Empty paths array
- "Permission denied: <path>" - Cannot read directory

**Example:**
```typescript
try {
  const message = await invoke('start_indexing', {
    paths: [
      '/home/user/Documents',
      '/home/user/Projects/work'
    ]
  });
  console.log(message); // "Indexing started in background"
} catch (error) {
  console.error('Indexing failed:', error);
}
```

**Events Emitted:**
- `indexing:progress` - Real-time progress updates
- `indexing:complete` - Indexing finished successfully
- `indexing:error` - Per-file errors (non-fatal)

**Notes:**
- Indexing runs in background (non-blocking)
- Can index multiple directories in one call
- Files are deduplicated by content hash
- Skips hidden files and excluded directories

---

### `stop_indexing`

Gracefully stop the current indexing operation.

**Signature:**
```typescript
function stop_indexing(): Promise<string>
```

**Parameters:**
None

**Returns:**
- Promise<string> - Success message: "Indexing stopped"

**Errors:**
- "No indexing operation is currently running"

**Example:**
```typescript
const message = await invoke('stop_indexing');
console.log(message); // "Indexing stopped"
```

**Behavior:**
- Current file completes processing
- No new files are started
- All indexed files remain searchable
- Progress events stop

**Timing:**
- Stops at next checkpoint (between files)
- May take 1-5 seconds to fully stop
- Check `get_index_status` to confirm stopped

---

### `get_index_status`

Get the current status of indexing operations.

**Signature:**
```typescript
function get_index_status(): Promise<IndexStatus>
```

**Parameters:**
None

**Returns:**
```typescript
interface IndexStatus {
  is_indexing: boolean;
  progress: ScanProgress | null;
  errors: string[];
}
```

**Example:**
```typescript
const status = await invoke('get_index_status');

if (status.is_indexing) {
  console.log(`Progress: ${status.progress.percentage}%`);
  console.log(`Files: ${status.progress.files_indexed} / ${status.progress.total_files}`);
  console.log(`Current: ${status.progress.current_file}`);
}

if (status.errors.length > 0) {
  console.log('Errors encountered:', status.errors);
}
```

**ScanProgress Fields:**
```typescript
interface ScanProgress {
  total_files: number;      // Total files found during scan
  files_indexed: number;    // Files successfully indexed
  current_file: string;     // Currently processing file path
  percentage: number;       // Progress (0-100)
}
```

**Poll Frequency:**
Recommended: 1-2 seconds for smooth UI updates

---

## Search Commands

### `search_files`

Search indexed files using FTS5 full-text search with optional filters and pagination.

**Signature:**
```typescript
function search_files(
  query: string,
  filters?: SearchFilters,
  limit?: number,
  offset?: number
): Promise<SearchResults>
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | Yes | - | Search query (FTS5 syntax) |
| `filters` | SearchFilters | No | null | Filter options |
| `limit` | number | No | 50 | Max results (1-1000) |
| `offset` | number | No | 0 | Skip N results |

**Returns:**
```typescript
interface SearchResults {
  results: SearchResult[];
  total: number;           // Number of results returned
  query_time_ms: number;   // Query execution time
}
```

**SearchResult:**
```typescript
interface SearchResult {
  file_id: number;
  path: string;
  filename: string;
  snippet: string;         // Highlighted excerpt with <mark> tags
  score: number;           // Relevance score (lower = better)
}
```

**SearchFilters:**
```typescript
interface SearchFilters {
  file_type?: string;      // e.g., "txt", "md", "pdf"
  min_size?: number;       // Minimum bytes
  max_size?: number;       // Maximum bytes
  date_from?: string;      // ISO 8601 date string
  date_to?: string;        // ISO 8601 date string
}
```

**Example: Basic Search**
```typescript
const results = await invoke('search_files', {
  query: 'rust programming'
});

console.log(`Found ${results.total} results in ${results.query_time_ms}ms`);

results.results.forEach(result => {
  console.log(`${result.filename}: ${result.snippet}`);
});
```

**Example: Filtered Search**
```typescript
const results = await invoke('search_files', {
  query: 'API documentation',
  filters: {
    file_type: 'md',
    min_size: 1000,
    date_from: '2025-01-01T00:00:00Z'
  },
  limit: 20
});
```

**Example: Paginated Search**
```typescript
// Page 1 (first 20)
const page1 = await invoke('search_files', {
  query: 'tutorial',
  limit: 20,
  offset: 0
});

// Page 2 (next 20)
const page2 = await invoke('search_files', {
  query: 'tutorial',
  limit: 20,
  offset: 20
});
```

**Query Syntax:**

**Simple Terms:**
```
rust programming
```
Finds documents containing "rust" OR "programming"

**Phrase Search:**
```
"exact phrase"
```
Finds exact phrase match

**Prefix Search:**
```
program*
```
Matches "programming", "programmer", "programs"

**Boolean NOT:**
```
rust NOT javascript
```
Finds "rust" but excludes documents with "javascript"

**Errors:**
- "Query cannot be empty" - Empty or whitespace-only query
- "Invalid filter: <reason>" - Malformed filter parameters
- "Database error: <details>" - Internal database issue

**Performance:**
- Target: <100ms for most queries
- Typical: <10ms for simple queries
- Complex filters may add 10-50ms

---

### `get_file_detail`

Get detailed information about a specific file including metadata and content.

**Signature:**
```typescript
function get_file_detail(
  file_id: number,
  include_full_content?: boolean
): Promise<FileDetail>
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `file_id` | number | Yes | - | File ID from search results |
| `include_full_content` | boolean | No | false | Load full content |

**Returns:**
```typescript
interface FileDetail {
  id: number;
  path: string;
  filename: string;
  file_type: string;
  size: number;                    // Bytes
  created_at: string;              // ISO 8601
  modified_at: string;             // ISO 8601
  content_preview: string | null;  // First 500 chars
  full_content: string | null;     // Full text (if requested)
  word_count: number | null;
  summary: string | null;
}
```

**Example: Preview Only**
```typescript
const detail = await invoke('get_file_detail', {
  file_id: 123,
  include_full_content: false
});

console.log(`File: ${detail.filename}`);
console.log(`Size: ${detail.size} bytes`);
console.log(`Words: ${detail.word_count}`);
console.log(`Summary: ${detail.summary}`);
console.log(`Preview: ${detail.content_preview}`);
```

**Example: Full Content**
```typescript
const detail = await invoke('get_file_detail', {
  file_id: 123,
  include_full_content: true
});

console.log(`Full content:\n${detail.full_content}`);
```

**Errors:**
- "File with ID <id> not found" - Invalid file ID
- "File has been deleted" - File marked as deleted
- "Database error: <details>" - Internal database issue

**Performance:**
- Preview mode: <5ms (only metadata + 500 chars)
- Full content: 5-50ms depending on file size

**Use Cases:**
- Preview: Display in search results
- Full content: Open in viewer/editor

---

### `get_search_stats`

Get indexing and search statistics.

**Signature:**
```typescript
function get_search_stats(): Promise<SearchStats>
```

**Parameters:**
None

**Returns:**
```typescript
interface SearchStats {
  total_files: number;       // Total files tracked
  indexed_files: number;     // Files with searchable content
  total_size_bytes: number;  // Combined size of all files
}
```

**Example:**
```typescript
const stats = await invoke('get_search_stats');

console.log(`Total Files: ${stats.total_files}`);
console.log(`Indexed Files: ${stats.indexed_files}`);
console.log(`Total Size: ${(stats.total_size_bytes / 1024 / 1024).toFixed(2)} MB`);

const progress = (stats.indexed_files / stats.total_files) * 100;
console.log(`Indexing Progress: ${progress.toFixed(1)}%`);
```

**Errors:**
- "Database error: <details>" - Internal database issue

**Performance:**
- Typical: <5ms
- Uses cached counts

**Use Cases:**
- Dashboard display
- Progress monitoring
- Storage analysis

---

## Type Definitions

### Rust Types

All types are defined in the Rust backend and serialized with Serde.

**IndexStatus** ([state.rs](../src-tauri/src/state.rs))
```rust
pub struct IndexStatus {
    pub is_indexing: bool,
    pub progress: Option<ScanProgress>,
    pub errors: Vec<String>,
}
```

**ScanProgress** ([indexer/types.rs](../src-tauri/src/indexer/types.rs))
```rust
pub struct ScanProgress {
    pub total_files: usize,
    pub files_indexed: usize,
    pub current_file: String,
    pub percentage: f64,
}
```

**SearchFilters** ([commands/search.rs](../src-tauri/src/commands/search.rs))
```rust
pub struct SearchFilters {
    pub file_type: Option<String>,
    pub min_size: Option<i64>,
    pub max_size: Option<i64>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}
```

**SearchResults** ([commands/search.rs](../src-tauri/src/commands/search.rs))
```rust
pub struct SearchResults {
    pub results: Vec<SearchResult>,
    pub total: usize,
    pub query_time_ms: u64,
}
```

**SearchResult** ([db/operations.rs](../src-tauri/src/db/operations.rs))
```rust
pub struct SearchResult {
    pub file_id: i64,
    pub path: String,
    pub filename: String,
    pub snippet: String,
    pub score: f64,
}
```

**FileDetail** ([commands/search.rs](../src-tauri/src/commands/search.rs))
```rust
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
```

**SearchStats** ([commands/search.rs](../src-tauri/src/commands/search.rs))
```rust
pub struct SearchStats {
    pub total_files: i64,
    pub indexed_files: i64,
    pub total_size_bytes: i64,
}
```

### TypeScript Types

For frontend development, create type definitions:

**types/api.ts:**
```typescript
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

export interface SearchFilters {
  file_type?: string;
  min_size?: number;
  max_size?: number;
  date_from?: string;
  date_to?: string;
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
  snippet: string;
  score: number;
}

export interface FileDetail {
  id: number;
  path: string;
  filename: string;
  file_type: string;
  size: number;
  created_at: string;
  modified_at: string;
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
```

---

## Error Handling

### Error Types

All commands return `Result<T, String>` where errors are user-friendly messages.

**CortexError Enum** ([error.rs](../src-tauri/src/error.rs)):
```rust
pub enum CortexError {
    DatabaseError(String),
    IndexingInProgress,
    IndexingNotRunning,
    InvalidQuery { query: String, reason: String },
    FileNotFound { path: String },
    PermissionDenied { path: String },
    // ... more variants
}
```

### Error Handling Best Practices

**TypeScript:**
```typescript
try {
  const results = await invoke('search_files', { query: 'test' });
  // Handle success
} catch (error) {
  // Error is a string from Rust
  if (error.includes('empty')) {
    console.error('Query cannot be empty');
  } else if (error.includes('not found')) {
    console.error('File not found');
  } else {
    console.error('Unexpected error:', error);
  }
}
```

**Common Error Messages:**

| Error Message | Cause | Solution |
|---------------|-------|----------|
| "Query cannot be empty" | Empty search query | Provide non-empty query |
| "Indexing is already in progress" | Concurrent indexing | Wait for current operation |
| "File with ID X not found" | Invalid file ID | Use IDs from search results |
| "Permission denied: <path>" | Cannot read directory | Check permissions |
| "Database error: <details>" | Internal DB issue | Check logs, report bug |

---

## Events

Cortex emits events for real-time updates during indexing.

### Event System

**TypeScript Setup:**
```typescript
import { listen } from '@tauri-apps/api/event';

// Listen for indexing progress
const unlisten = await listen('indexing:progress', (event) => {
  const progress: IndexProgressEvent = event.payload;
  console.log(`Progress: ${progress.progress_percentage}%`);
});

// Cleanup when done
unlisten();
```

### `indexing:progress`

Real-time progress updates during indexing (emitted every 10 files).

**Payload:**
```typescript
interface IndexProgressEvent {
  total_files: number;
  indexed_files: number;
  current_file: string;
  progress_percentage: number;
}
```

**Example:**
```typescript
await listen('indexing:progress', (event) => {
  const { indexed_files, total_files, progress_percentage } = event.payload;
  updateProgressBar(progress_percentage);
  updateStatusText(`${indexed_files} / ${total_files} files`);
});
```

**Frequency:**
- Emitted every 10 files
- Always emitted on first and last file

### `indexing:complete`

Emitted when indexing finishes successfully.

**Payload:**
```typescript
interface IndexCompleteEvent {
  total_files: number;
  indexed_files: number;
  failed_files: number;
  duration_seconds: number;
  errors: string[];
}
```

**Example:**
```typescript
await listen('indexing:complete', (event) => {
  const { indexed_files, duration_seconds } = event.payload;
  console.log(`Indexed ${indexed_files} files in ${duration_seconds}s`);
  showNotification('Indexing complete!');
});
```

### `indexing:error`

Emitted when a file fails to index (non-fatal).

**Payload:**
```typescript
interface IndexErrorEvent {
  file_path: string;
  error_message: string;
}
```

**Example:**
```typescript
await listen('indexing:error', (event) => {
  const { file_path, error_message } = event.payload;
  console.warn(`Failed to index ${file_path}: ${error_message}`);
  addErrorToList(file_path, error_message);
});
```

**Notes:**
- Individual file errors don't stop indexing
- All errors are collected in `IndexStatus.errors`
- Check errors after completion

---

## Code Examples

### Complete Indexing Workflow

```typescript
import { invoke, listen } from '@tauri-apps/api';
import type { IndexProgressEvent, IndexCompleteEvent } from './types/api';

class IndexingManager {
  private progressUnlisten: (() => void) | null = null;
  private completeUnlisten: (() => void) | null = null;

  async startIndexing(paths: string[]) {
    // Setup event listeners
    this.progressUnlisten = await listen('indexing:progress',
      this.handleProgress.bind(this)
    );

    this.completeUnlisten = await listen('indexing:complete',
      this.handleComplete.bind(this)
    );

    // Start indexing
    try {
      const message = await invoke('start_indexing', { paths });
      console.log(message);
    } catch (error) {
      this.cleanup();
      throw error;
    }
  }

  async stopIndexing() {
    try {
      await invoke('stop_indexing');
    } finally {
      this.cleanup();
    }
  }

  handleProgress(event: { payload: IndexProgressEvent }) {
    const { progress_percentage, current_file } = event.payload;
    this.updateUI(progress_percentage, current_file);
  }

  handleComplete(event: { payload: IndexCompleteEvent }) {
    const { indexed_files, duration_seconds } = event.payload;
    this.showComplete(indexed_files, duration_seconds);
    this.cleanup();
  }

  updateUI(percentage: number, file: string) {
    document.getElementById('progress-bar')!.style.width = `${percentage}%`;
    document.getElementById('current-file')!.textContent = file;
  }

  showComplete(files: number, duration: number) {
    alert(`Indexed ${files} files in ${duration.toFixed(1)}s`);
  }

  cleanup() {
    this.progressUnlisten?.();
    this.completeUnlisten?.();
    this.progressUnlisten = null;
    this.completeUnlisten = null;
  }
}
```

### Advanced Search Component

```typescript
import { invoke } from '@tauri-apps/api';
import type { SearchResults, SearchFilters } from './types/api';

class SearchManager {
  private currentPage = 0;
  private pageSize = 20;

  async search(
    query: string,
    filters: SearchFilters = {}
  ): Promise<SearchResults> {
    if (!query.trim()) {
      throw new Error('Query cannot be empty');
    }

    try {
      return await invoke('search_files', {
        query,
        filters,
        limit: this.pageSize,
        offset: this.currentPage * this.pageSize
      });
    } catch (error) {
      console.error('Search failed:', error);
      throw error;
    }
  }

  async nextPage(query: string, filters: SearchFilters = {}) {
    this.currentPage++;
    return this.search(query, filters);
  }

  async prevPage(query: string, filters: SearchFilters = {}) {
    if (this.currentPage > 0) {
      this.currentPage--;
    }
    return this.search(query, filters);
  }

  resetPagination() {
    this.currentPage = 0;
  }
}

// Usage
const searchManager = new SearchManager();

async function performSearch() {
  const query = document.getElementById('search-input')!.value;
  const filters: SearchFilters = {
    file_type: 'md',
    min_size: 1000
  };

  try {
    const results = await searchManager.search(query, filters);
    displayResults(results);
  } catch (error) {
    displayError(error);
  }
}
```

### Real-time Status Dashboard

```typescript
import { invoke } from '@tauri-apps/api';
import type { SearchStats, IndexStatus } from './types/api';

class StatusDashboard {
  private intervalId: number | null = null;

  start() {
    this.update();
    this.intervalId = setInterval(() => this.update(), 2000);
  }

  stop() {
    if (this.intervalId !== null) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  async update() {
    try {
      const [stats, status] = await Promise.all([
        invoke<SearchStats>('get_search_stats'),
        invoke<IndexStatus>('get_index_status')
      ]);

      this.updateStats(stats);
      this.updateStatus(status);
    } catch (error) {
      console.error('Failed to update dashboard:', error);
    }
  }

  updateStats(stats: SearchStats) {
    const progressPercent = (stats.indexed_files / stats.total_files) * 100;
    const sizeMB = (stats.total_size_bytes / 1024 / 1024).toFixed(2);

    document.getElementById('total-files')!.textContent =
      stats.total_files.toString();
    document.getElementById('indexed-files')!.textContent =
      stats.indexed_files.toString();
    document.getElementById('total-size')!.textContent =
      `${sizeMB} MB`;
    document.getElementById('index-progress')!.textContent =
      `${progressPercent.toFixed(1)}%`;
  }

  updateStatus(status: IndexStatus) {
    if (status.is_indexing && status.progress) {
      document.getElementById('status')!.textContent = 'Indexing...';
      document.getElementById('current-file')!.textContent =
        status.progress.current_file;
    } else {
      document.getElementById('status')!.textContent = 'Idle';
      document.getElementById('current-file')!.textContent = '';
    }
  }
}

// Usage
const dashboard = new StatusDashboard();
dashboard.start();

// Cleanup on unmount
window.addEventListener('beforeunload', () => dashboard.stop());
```

---

## Best Practices

### Performance

1. **Batch Operations**
   - Index multiple directories in one call
   - Use pagination for large result sets
   - Cache search results when appropriate

2. **Resource Management**
   - Clean up event listeners
   - Limit concurrent operations
   - Use preview mode for file content

3. **Error Handling**
   - Always use try-catch
   - Display user-friendly messages
   - Log errors for debugging

### User Experience

1. **Progress Feedback**
   - Show progress bar during indexing
   - Display current file being processed
   - Show query timing for searches

2. **Debouncing**
   - Debounce search input (300-500ms)
   - Don't search on every keystroke
   - Cancel previous searches if still pending

3. **Pagination**
   - Load 20-50 results per page
   - Implement infinite scroll or pagination
   - Show total result count

### Security

1. **Path Validation**
   - Validate user-provided paths
   - Don't allow arbitrary path indexing
   - Sanitize displayed file paths

2. **Query Sanitization**
   - Limit query length
   - Handle special characters
   - Prevent injection attempts

3. **Error Messages**
   - Don't expose sensitive paths in errors
   - Log details server-side
   - Show generic messages to users

---

**Cortex API Reference v0.1.0** | [Report Issue](https://github.com/yourusername/cortex/issues) | [Edit on GitHub](https://github.com/yourusername/cortex/edit/main/docs/API_REFERENCE.md)
