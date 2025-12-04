# CORTEX - CLAUDE DEVELOPMENT CONTEXT

**Version:** 2.0  
**Last Updated:** December 2024  
**Purpose:** Condensed context for efficient Claude collaboration in VS Code

---

## PROJECT OVERVIEW

**What is Cortex?**
Local-first AI file intelligence system. Semantic search + smart organization + auto-tagging. Built on Rust/Tauri/SvelteKit.

**Core Value:** 
- Solves file chaos for knowledge workers
- 10,000 files indexed in <5 min
- Search results <100ms (keyword), <500ms (semantic)
- 100% local privacy by default

**Strategic Position:**
- Standalone product ($49-79)
- AI memory layer for Forge ecosystem (AuthorForge, VibeForge, DataForge)
- Foundation for cross-app intelligence

---

## TECHNICAL STACK

### Backend (Rust)
```toml
tokio = "1"              # Async runtime
rusqlite = "0.31"        # SQLite with FTS5
notify = "6.0"           # Filesystem watching
tantivy = "0.21"         # Full-text search
docx-rs = "0.4"          # DOCX extraction
pdf-extract = "0.7"      # PDF extraction
ort = "1.16"             # ONNX Runtime (local embeddings)
reqwest = "0.11"         # NeuroForge API (optional)
```

### Frontend (SvelteKit + Tauri)
```json
{
  "@tauri-apps/api": "^2.0.0",
  "svelte": "^4.0.0",
  "@sveltejs/kit": "^2.0.0",
  "tailwindcss": "^3.4.0"
}
```

### Database
- **SQLite 3.44+** with FTS5 extension
- WAL mode for concurrent access
- Location: `~/.cortex/db.sqlite`

---

## ARCHITECTURE OVERVIEW

```
┌─────────────────────────────────────────────┐
│         TAURI APPLICATION                   │
├─────────────────────────────────────────────┤
│  Frontend (SvelteKit)                       │
│  ├─ Search Bar (global, Cmd+K)             │
│  ├─ Sidebar (navigation, stats)            │
│  ├─ Content Area (file grid/list)          │
│  └─ Preview Panel (file details)           │
├─────────────────────────────────────────────┤
│  Backend (Rust)                             │
│  ├─ Indexing Pipeline                      │
│  │  └─ [Scanner→Watcher→Extractor→AI]     │
│  ├─ Search Engine                          │
│  │  └─ [FTS + Vector + Hybrid Fusion]     │
│  ├─ Smart Features                         │
│  │  └─ [Collections, Duplicates, Cleanup] │
│  └─ Database (SQLite + FTS5)              │
├─────────────────────────────────────────────┤
│  Filesystem (User's files)                  │
│  Optional: NeuroForge/DataForge            │
└─────────────────────────────────────────────┘
```

---

## DATABASE SCHEMA (KEY TABLES)

```sql
-- Core file metadata
CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    filename TEXT NOT NULL,
    file_type TEXT NOT NULL,
    size INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    modified_at TIMESTAMP NOT NULL,
    hash TEXT,
    is_deleted BOOLEAN DEFAULT 0
);

-- Extracted content
CREATE TABLE file_content (
    file_id INTEGER PRIMARY KEY,
    text_content TEXT,
    word_count INTEGER,
    summary TEXT,              -- AI-generated
    language TEXT,
    FOREIGN KEY (file_id) REFERENCES files(id)
);

-- Semantic search vectors
CREATE TABLE embeddings (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL,
    chunk_text TEXT NOT NULL,
    embedding BLOB NOT NULL,   -- 384D vector
    chunk_index INTEGER NOT NULL,
    FOREIGN KEY (file_id) REFERENCES files(id)
);

-- Tags (user + AI)
CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    color TEXT,
    auto_generated BOOLEAN DEFAULT 0
);

CREATE TABLE file_tags (
    file_id INTEGER,
    tag_id INTEGER,
    confidence REAL,
    added_by TEXT,             -- 'user' or 'ai'
    PRIMARY KEY (file_id, tag_id)
);

-- Smart Collections (virtual folders)
CREATE TABLE smart_collections (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    rules JSON NOT NULL,       -- Filter criteria
    auto_update BOOLEAN DEFAULT 1
);

-- Full-text search
CREATE VIRTUAL TABLE files_fts USING fts5(
    filename,
    content,
    tokenize='porter unicode61'
);
```

---

## CRITICAL TAURI COMMANDS

```rust
// Indexing
#[tauri::command]
async fn start_indexing(paths: Vec<String>) -> Result<String, String>

#[tauri::command]
async fn get_index_status() -> Result<IndexStatus, String>

// Search
#[tauri::command]
async fn search_files(
    query: String,
    filters: Option<SearchFilters>,
    limit: usize,
    offset: usize
) -> Result<SearchResults, String>

// Semantic (Phase 2)
#[tauri::command]
async fn semantic_search(filters: SemanticSearchFilters) -> Result<Vec<SemanticSearchResult>, String>

#[tauri::command]
async fn get_embedding_status() -> Result<EmbeddingStatus, String>

// File operations
#[tauri::command]
async fn get_file_detail(file_id: i64) -> Result<FileDetail, String>

// Tags
#[tauri::command]
async fn apply_tags(file_id: i64, tag_ids: Vec<i64>) -> Result<(), String>

// Collections
#[tauri::command]
async fn create_collection(
    name: String,
    rules: CollectionRules
) -> Result<i64, String>

// Window management
#[tauri::command]
fn show_main_and_close_splash<R: Runtime>(app: AppHandle<R>) -> Result<(), String>
```

---

## DESIGN SYSTEM

### Color Palette (Neural Gold)
```css
--cortex-black: #0A0A0C;
--cortex-deep: #0E0F12;
--slate-byte: #15161A;
--matrix: #181A1F;
--ash: #22242A;

--neural-gold: #C9A46C;
--ember-gold: #F3C87D;
--silver-neural: #CCCCD6;

--signal-blue: #4CA3FF;
--neural-violet: #AD8FFF;
--mint-neural: #64D2B4;

--text-primary: #FFFFFF;
--text-secondary: #CCCCD6;
--text-tertiary: #9C9CA6;
```

### Typography
```css
--font-primary: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto;
--font-mono: "SF Mono", Monaco, "Cascadia Code";

--text-xs: 0.75rem;
--text-sm: 0.875rem;
--text-base: 1rem;
--text-lg: 1.125rem;
--text-xl: 1.25rem;
```

### Component Patterns
- **Buttons:** Neural gold primary, transparent secondary
- **Inputs:** Matrix background, ash borders, gold focus
- **Cards:** Slate-byte background, ash borders, hover elevation
- **Hover:** Gold rim glow + 1px elevation
- **Active:** Deeper glow + silver outline

---

## CURRENT IMPLEMENTATION STATUS

### ✅ COMPLETED (Phase 0-1)
- [x] Project setup (Rust/Tauri/SvelteKit)
- [x] Database schema with FTS5
- [x] Basic file indexing pipeline
- [x] Filesystem watcher
- [x] Content extraction (txt, md, docx, pdf)
- [x] Keyword search with filters
- [x] Desktop UI (sidebar, search, grid, preview)
- [x] Settings page
- [x] Onboarding flow
- [x] Toast notifications
- [x] Keyboard shortcuts
- [x] Splash screen with custom icon

### 🚧 IN PROGRESS (Phase 2)
- [ ] Local embedding generation (ONNX)
- [ ] Vector similarity search
- [ ] Hybrid search (FTS + semantic fusion)
- [ ] Auto-tagging system
- [ ] AI document summaries
- [ ] Related files detection
- [ ] Image OCR (tesseract)
- [ ] NeuroForge integration (optional)

### 📋 PLANNED (Phase 3-4)
- [ ] Smart Collections engine
- [ ] Duplicate detection
- [ ] Cleanup analyzer
- [ ] Command palette (Cmd+P)
- [ ] Bulk operations
- [ ] Export/import
- [ ] Backup/restore
- [ ] Beta testing + polish

---

## KEY DEVELOPMENT PRINCIPLES

1. **100% Test Coverage**: Unit + integration tests from start
2. **Production-Ready Code**: No prototyping, ship-quality only
3. **Edge Cases First**: Handle errors, empty states, loading gracefully
4. **Evidence-Based UX**: Cognitive Load Theory, Hick's Law, Fitts's Law
5. **Local-First Privacy**: All data on device, optional cloud
6. **Performance Budget**: <2s startup, <100ms search, <150MB idle

---

## COMMON PATTERNS

### Error Handling
```rust
#[derive(Debug, Serialize)]
struct CortexError {
    kind: ErrorKind,
    message: String,
    context: Option<serde_json::Value>,
}

// Always return Result<T, String> in Tauri commands
// Log errors with tracing::error!
// Show user-friendly messages in UI toasts
```

### Async Operations
```rust
use tokio::task;

// Offload CPU-intensive work
let result = task::spawn_blocking(move || {
    expensive_operation(&data)
}).await?;
```

### State Management (Frontend)
```typescript
// Svelte 5 runes
let query = $state('');
let results = $state<SearchResults | null>(null);
let isSearching = $state(false);

// Derived state
let hasResults = $derived(results !== null && results.total > 0);
```

### Database Transactions
```rust
let mut tx = db.transaction()?;

// Batch operations
for chunk in items.chunks(100) {
    tx.execute_batch(/* SQL */)?;
}

tx.commit()?;
```

---

## CRITICAL PATHS TO IMPLEMENT

### Path 1: Semantic Search (Priority)
1. Implement local ONNX model loading
2. Generate embeddings for indexed files
3. Store vectors in `embeddings` table
4. Build cosine similarity search
5. Create hybrid fusion (FTS + vector)
6. Add UI toggle (keyword vs semantic)

### Path 2: Auto-Tagging
1. Analyze file content for keywords
2. Use TF-IDF for importance scoring
3. Generate tag suggestions with confidence
4. Store in `file_tags` with confidence scores
5. UI for accepting/rejecting suggestions

### Path 3: Smart Collections
1. Parse collection rules (JSON format)
2. Convert rules to SQL WHERE clauses
3. Real-time updates when files change
4. Collection templates (Recent Writing, etc.)
5. UI for rule builder

---

## TESTING STRATEGY

### Unit Tests (Rust)
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_file_extraction() {
        let content = extract_text("test.txt").unwrap();
        assert_eq!(content, "expected");
    }
    
    #[tokio::test]
    async fn test_search() {
        let results = search_files("query", None).await.unwrap();
        assert!(results.len() > 0);
    }
}
```

### Integration Tests
- Full indexing pipeline (scan → extract → store)
- Search after indexing
- Filesystem watcher triggers re-index
- Tag operations persist correctly

### UI Tests (Playwright)
- Search shows results
- Keyboard shortcuts work
- File selection updates preview
- Onboarding flow completes

---

## PERFORMANCE TARGETS

- **Indexing:** 50+ files/second
- **Search:** <100ms (keyword), <500ms (semantic)
- **Startup:** <2 seconds cold start
- **Memory:** <150MB idle, <500MB indexing
- **Database:** ~20KB per file
- **UI:** 60 FPS scrolling

---

## FILE LOCATIONS

```
cortex/
├── src/                          # SvelteKit frontend
│   ├── lib/
│   │   ├── components/          # UI components
│   │   ├── types/api.ts         # TypeScript types
│   │   └── stores/              # Svelte stores
│   ├── routes/                  # Pages
│   └── app.css                  # Global styles
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              # Entry point
│   │   ├── indexer/             # Indexing logic
│   │   ├── search/              # Search engine
│   │   ├── db/                  # Database layer
│   │   └── ai/                  # AI features
│   ├── icons/                   # App icons
│   ├── splashscreen.html        # Splash screen
│   └── tauri.conf.json          # Tauri config
└── Cortex_Complete_Implementation_Plan.md
```

---

## NEXT SESSION GOALS

**Current Focus:** Phase 2 - AI Features

**Priority Tasks:**
1. Implement local ONNX embedding model
2. Generate embeddings for indexed files
3. Build vector similarity search
4. Create semantic search UI component
5. Add hybrid search (FTS + semantic fusion)

**Success Criteria:**
- Semantic search returns relevant results
- Search time <500ms for semantic queries
- Embedding generation runs in background
- UI shows embedding progress
- All features have tests

---

## USEFUL COMMANDS

```bash
# Development
npm run tauri dev

# Build
npm run tauri build

# Test Rust
cd src-tauri && cargo test

# Test Frontend
npm run test

# Format
cargo fmt
npm run format

# Lint
cargo clippy
npm run lint
```

---

## QUESTIONS TO ASK WHEN STUCK

1. Does this maintain 100% local privacy?
2. Is this the simplest implementation that works?
3. Have I handled the error cases?
4. Does this match the design system?
5. Will this perform well with 10,000+ files?
6. Is there a test for this?
7. What's the user experience on failure?

---

**Remember:** Ship production-ready code. Test everything. Focus on user value. Keep it fast and private.
