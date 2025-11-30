# Cortex Developer Guide

**Version:** 0.1.0 (Phase 0)
**Last Updated:** 2025-11-29

---

## Table of Contents

1. [Project Architecture](#project-architecture)
2. [Development Setup](#development-setup)
3. [Project Structure](#project-structure)
4. [Backend Development](#backend-development)
5. [Frontend Development](#frontend-development)
6. [Testing](#testing)
7. [Building](#building)
8. [Contributing Workflow](#contributing-workflow)
9. [Code Style](#code-style)
10. [Troubleshooting](#troubleshooting)

---

## Project Architecture

### Technology Stack

**Backend:**
- **Tauri 2.0** - Desktop application framework
- **Rust 1.75+** - Systems programming language
- **SQLite 3** - Embedded database
- **FTS5** - Full-text search engine
- **Tokio** - Async runtime

**Frontend:**
- **SvelteKit** - Web framework
- **Svelte 5** - UI framework with runes
- **TypeScript** - Type-safe JavaScript
- **Tailwind CSS** - Utility-first CSS

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend (SvelteKit)                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │   Search    │  │  Indexing   │  │ Statistics  │     │
│  │  Component  │  │  Component  │  │  Dashboard  │     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │
│         │                 │                 │            │
│         └─────────────────┴─────────────────┘            │
│                           │                              │
│                    Tauri IPC (invoke)                    │
└───────────────────────────┼──────────────────────────────┘
                            │
┌───────────────────────────┼──────────────────────────────┐
│                    Backend (Rust)                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │   Search    │  │  Indexing   │  │   Database  │     │
│  │  Commands   │  │  Commands   │  │   Layer     │     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │
│         │                 │                 │            │
│         └─────────────────┴─────────────────┘            │
│                           │                              │
│  ┌──────────────────────────────────────────────────┐   │
│  │            Application State (AppState)          │   │
│  │  • Database (Arc<RwLock<Database>>)             │   │
│  │  • Indexing Progress (Arc<RwLock<Progress>>)    │   │
│  │  • Error Collection (Arc<RwLock<Vec<Error>>>)   │   │
│  └──────────────────────────────────────────────────┘   │
│                           │                              │
│  ┌────────────┐  ┌───────────────┐  ┌──────────────┐   │
│  │  Indexer   │  │   Extractor   │  │   Scanner    │   │
│  │  Pipeline  │  │   (txt/pdf)   │  │   (files)    │   │
│  └────────────┘  └───────────────┘  └──────────────┘   │
└───────────────────────────┼──────────────────────────────┘
                            │
                 ┌──────────┴──────────┐
                 │   SQLite + FTS5      │
                 │  ~/.cortex/db.sqlite │
                 └─────────────────────┘
```

### Data Flow

**Indexing Flow:**
```
User clicks "Index"
  → Frontend invokes start_indexing(paths)
  → Backend spawns background task
  → Scanner finds all files
  → For each file:
      → Extract content (txt/pdf/docx/md)
      → Insert into database
      → Emit progress event
  → Emit completion event
  → Frontend updates UI
```

**Search Flow:**
```
User types query
  → Frontend debounces input (300ms)
  → Frontend invokes search_files(query, filters)
  → Backend performs FTS5 query
  → Returns results with snippets
  → Frontend displays with highlights
  → User clicks result
  → Frontend invokes get_file_detail(id)
  → Backend returns full metadata
  → Frontend displays in modal/panel
```

---

## Development Setup

### Prerequisites

**Required:**
- Rust 1.75+ ([rustup.rs](https://rustup.rs))
- Node.js 18+ ([nodejs.org](https://nodejs.org))
- npm 9+ (comes with Node.js)
- Git

**Platform-Specific:**

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  libgtk-3-dev \
  build-essential \
  curl
```

**macOS:**
```bash
xcode-select --install
```

**Windows:**
- Visual Studio 2022 with C++ tools
- Or VS Build Tools 2022

### Initial Setup

**1. Clone Repository:**
```bash
git clone https://github.com/yourusername/cortex.git
cd cortex
```

**2. Install Dependencies:**
```bash
# Install frontend dependencies
npm install

# Verify Rust installation
rustc --version
cargo --version
```

**3. Build Backend:**
```bash
cd src-tauri
cargo build
```

**4. Run Development Server:**
```bash
cd ..
npm run dev
```

Application should open at `http://localhost:1420`

### Development Workflow

**Terminal 1 - Frontend:**
```bash
npm run dev
```
Runs SvelteKit dev server with hot reload.

**Terminal 2 - Backend Testing:**
```bash
cd src-tauri
cargo test --lib
cargo run --bin load_test
```

**Terminal 3 - Logs:**
```bash
tail -f ~/.cortex/cortex.log
```

---

## Project Structure

```
cortex/
├── src/                      # Frontend source
│   ├── lib/
│   │   ├── components/       # Svelte components
│   │   ├── stores/           # State management (if needed)
│   │   └── utils/            # Helper functions
│   ├── routes/               # SvelteKit routes
│   │   ├── +page.svelte      # Main search page
│   │   └── +layout.svelte    # Root layout
│   └── app.html              # HTML template
│
├── src-tauri/                # Backend source
│   ├── src/
│   │   ├── commands/         # Tauri commands
│   │   │   ├── indexing.rs   # Indexing commands
│   │   │   └── search.rs     # Search commands
│   │   ├── db/               # Database layer
│   │   │   ├── mod.rs        # Database manager
│   │   │   ├── schema.rs     # Schema definitions
│   │   │   └── operations.rs # CRUD operations
│   │   ├── indexer/          # Indexing pipeline
│   │   │   ├── scanner.rs    # File scanner
│   │   │   ├── types.rs      # IndexJob, Priority
│   │   │   ├── watcher.rs    # File watcher
│   │   │   └── extractors/   # Content extractors
│   │   │       ├── text.rs   # Plain text
│   │   │       ├── markdown.rs # Markdown
│   │   │       ├── pdf.rs    # PDF
│   │   │       └── docx.rs   # Word docs
│   │   ├── search/           # Search engine (future)
│   │   ├── error.rs          # Error types
│   │   ├── state.rs          # App state
│   │   └── main.rs           # Entry point
│   │
│   ├── tests/                # Integration tests
│   │   ├── commands_test.rs
│   │   ├── search_commands_test.rs
│   │   └── extraction_test.rs
│   │
│   ├── benches/              # Performance benchmarks
│   │   ├── db_benchmark.rs
│   │   ├── indexing_benchmark.rs
│   │   ├── search_benchmark.rs
│   │   └── load_test.rs
│   │
│   └── Cargo.toml            # Rust dependencies
│
├── docs/                     # Documentation
│   ├── USER_GUIDE.md
│   ├── API_REFERENCE.md
│   ├── DEVELOPER_GUIDE.md (this file)
│   └── DEPLOYMENT.md
│
├── .github/                  # GitHub workflows
│   └── workflows/
│       ├── test.yml          # CI tests
│       └── release.yml       # Release builds
│
├── README.md                 # Project overview
├── SETUP.md                  # Setup instructions
├── TESTING.md                # Testing guide
├── PERFORMANCE.md            # Performance docs
├── STATUS.md                 # Development status
├── package.json              # Frontend dependencies
├── tailwind.config.js        # Tailwind config
├── svelte.config.js          # Svelte config
└── vite.config.ts            # Vite config
```

---

## Backend Development

### Module Overview

**src-tauri/src/commands/**
Tauri command handlers exposed to frontend.

**src-tauri/src/db/**
Database layer with SQLite operations.

**src-tauri/src/indexer/**
File scanning, content extraction, indexing pipeline.

**src-tauri/src/state.rs**
Global application state shared across commands.

**src-tauri/src/error.rs**
User-friendly error types.

### Adding a New Command

**1. Define in `commands/` module:**

```rust
// src-tauri/src/commands/my_module.rs

use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn my_command(
    param: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Access database
    let db = state.db.read().await;
    let conn = db.get_connection();

    // Your logic here

    Ok("Success".to_string())
}
```

**2. Register in `main.rs`:**

```rust
mod commands;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::my_module::my_command,  // Add here
            // ... other commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**3. Call from Frontend:**

```typescript
import { invoke } from '@tauri-apps/api';

const result = await invoke('my_command', { param: 'value' });
```

### Database Operations

**Adding a new query:**

```rust
// src-tauri/src/db/operations.rs

pub fn my_query(
    conn: &Connection,
    param: &str,
) -> Result<Vec<MyResult>, CortexError> {
    let mut stmt = conn.prepare("
        SELECT id, name FROM my_table
        WHERE name LIKE ?
    ")?;

    let results = stmt
        .query_map([param], |row| {
            Ok(MyResult {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(results)
}
```

**Schema Migrations:**

For schema changes, update `db/schema.rs`:

```rust
pub fn create_tables(conn: &Connection) -> Result<(), CortexError> {
    conn.execute("
        CREATE TABLE IF NOT EXISTS my_new_table (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        )
    ", [])?;

    Ok(())
}
```

### Content Extractors

**Adding a new file type:**

**1. Create extractor:**

```rust
// src-tauri/src/indexer/extractors/my_format.rs

use crate::error::CortexError;
use super::ExtractedContent;

pub struct MyFormatExtractor;

impl MyFormatExtractor {
    pub fn extract(path: &std::path::Path) -> Result<ExtractedContent, CortexError> {
        // Read file
        let content = std::fs::read_to_string(path)?;

        // Parse format-specific content
        let text = parse_my_format(&content)?;

        Ok(ExtractedContent::new(text))
    }
}

fn parse_my_format(content: &str) -> Result<String, CortexError> {
    // Your parsing logic
    Ok(content.to_string())
}
```

**2. Register in extractor module:**

```rust
// src-tauri/src/indexer/extractors/mod.rs

pub mod my_format;
use my_format::MyFormatExtractor;

pub fn extract(path: &Path) -> Result<ExtractedContent, CortexError> {
    match extension {
        "myext" => MyFormatExtractor::extract(path),
        // ... other formats
        _ => TextExtractor::extract(path),
    }
}
```

### Event Emission

**Emit custom events:**

```rust
use tauri::{AppHandle, Emitter};

#[derive(Clone, serde::Serialize)]
struct MyEvent {
    message: String,
    count: usize,
}

fn emit_my_event(app: &AppHandle, message: String, count: usize) {
    let _ = app.emit("my_event", MyEvent { message, count });
}
```

**Listen in frontend:**

```typescript
import { listen } from '@tauri-apps/api/event';

await listen('my_event', (event) => {
  console.log(event.payload.message, event.payload.count);
});
```

---

## Frontend Development

### Component Structure

**Basic Search Component:**

```svelte
<!-- src/lib/components/Search.svelte -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api';
  import type { SearchResults } from '$lib/types/api';

  let query = $state('');
  let results = $state<SearchResults | null>(null);
  let isSearching = $state(false);

  async function handleSearch() {
    if (!query.trim()) return;

    isSearching = true;
    try {
      results = await invoke('search_files', { query });
    } catch (error) {
      console.error('Search failed:', error);
    } finally {
      isSearching = false;
    }
  }
</script>

<div class="search-container">
  <input
    type="text"
    bind:value={query}
    onkeyup={(e) => e.key === 'Enter' && handleSearch()}
    placeholder="Search files..."
  />

  <button onclick={handleSearch} disabled={isSearching}>
    {isSearching ? 'Searching...' : 'Search'}
  </button>

  {#if results}
    <div class="results">
      <p>Found {results.total} results in {results.query_time_ms}ms</p>

      {#each results.results as result}
        <div class="result-card">
          <h3>{result.filename}</h3>
          <p>{@html result.snippet}</p>
        </div>
      {/each}
    </div>
  {/if}
</div>
```

### State Management

**Using Svelte 5 Runes:**

```typescript
// src/lib/stores/indexing.svelte.ts

import { invoke, listen } from '@tauri-apps/api';
import type { IndexStatus } from '$lib/types/api';

class IndexingStore {
  status = $state<IndexStatus | null>(null);
  isIndexing = $derived(this.status?.is_indexing ?? false);

  async init() {
    // Listen for progress events
    await listen('indexing:progress', (event) => {
      if (this.status) {
        this.status.progress = event.payload;
      }
    });

    // Load initial status
    await this.refresh();
  }

  async refresh() {
    this.status = await invoke('get_index_status');
  }

  async start(paths: string[]) {
    await invoke('start_indexing', { paths });
    await this.refresh();
  }

  async stop() {
    await invoke('stop_indexing');
    await this.refresh();
  }
}

export const indexingStore = new IndexingStore();
```

**Usage in component:**

```svelte
<script lang="ts">
  import { indexingStore } from '$lib/stores/indexing.svelte';
  import { onMount } from 'svelte';

  onMount(() => {
    indexingStore.init();
  });
</script>

{#if indexingStore.isIndexing}
  <div>
    <p>Indexing: {indexingStore.status?.progress?.percentage}%</p>
    <button onclick={() => indexingStore.stop()}>Stop</button>
  </div>
{/if}
```

### Styling with Tailwind

**Neural Gold Theme:**

```javascript
// tailwind.config.js

export default {
  theme: {
    extend: {
      colors: {
        'neural-gold': '#D4AF37',
        'neural-dark': '#1A1A1A',
        'neural-gray': '#2D2D2D',
      },
    },
  },
};
```

**Component styling:**

```svelte
<div class="bg-neural-dark text-white p-4 rounded-lg">
  <h1 class="text-neural-gold text-2xl font-bold">Cortex</h1>
  <p class="text-gray-400">Search your files</p>
</div>
```

---

## Testing

### Unit Tests (Rust)

**Test database operations:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_file() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        let file_id = insert_file(
            &conn,
            "/test/file.txt",
            "file.txt",
            "txt",
            1024,
            "2025-11-29T00:00:00Z",
            "2025-11-29T00:00:00Z",
            None,
            "/test",
        ).unwrap();

        assert!(file_id > 0);
    }
}
```

**Run unit tests:**

```bash
cd src-tauri
cargo test --lib
```

### Integration Tests (Rust)

**Test full pipelines:**

```rust
// src-tauri/tests/my_test.rs

use cortex_lib::commands::search::search_files;
use cortex_lib::state::AppState;
use std::sync::Arc;

#[tokio::test]
async fn test_full_search_pipeline() {
    let state = Arc::new(AppState::new().await.unwrap());

    // Setup test data
    // ... insert files

    // Test search
    let results = search_files(
        "test query".to_string(),
        None,
        None,
        None,
        tauri::State::from(state.as_ref()),
    )
    .await
    .unwrap();

    assert_eq!(results.total, 1);
}
```

**Run integration tests:**

```bash
cargo test --tests
```

### Frontend Tests

**Vitest for unit tests:**

```typescript
// src/lib/utils/format.test.ts

import { describe, it, expect } from 'vitest';
import { formatFileSize } from './format';

describe('formatFileSize', () => {
  it('formats bytes correctly', () => {
    expect(formatFileSize(1024)).toBe('1.00 KB');
    expect(formatFileSize(1048576)).toBe('1.00 MB');
  });
});
```

**Run frontend tests:**

```bash
npm test
```

### Benchmarks

**Run performance benchmarks:**

```bash
cd src-tauri

# Database benchmarks
cargo run --release --bin db_benchmark

# Indexing benchmarks
cargo run --release --bin indexing_benchmark

# Search benchmarks
cargo run --release --bin search_benchmark

# Load test
cargo run --release --bin load_test
```

See [PERFORMANCE.md](../PERFORMANCE.md) for details.

---

## Building

### Development Build

```bash
# Frontend only (hot reload)
npm run dev

# Full Tauri dev mode
npm run tauri dev
```

### Production Build

```bash
# Build frontend
npm run build

# Build backend
cd src-tauri
cargo build --release

# Build Tauri application
npm run tauri build
```

**Output:**
- Linux: `src-tauri/target/release/bundle/deb/cortex_*.deb`
- macOS: `src-tauri/target/release/bundle/dmg/Cortex_*.dmg`
- Windows: `src-tauri/target/release/bundle/msi/Cortex_*.msi`

### Release Checklist

Before creating a release:

- [ ] All tests passing: `cargo test`
- [ ] Benchmarks meet targets: `cargo run --release --bin load_test`
- [ ] Frontend builds: `npm run build`
- [ ] Backend builds: `cargo build --release`
- [ ] Version bumped in `Cargo.toml` and `package.json`
- [ ] CHANGELOG.md updated
- [ ] Documentation updated
- [ ] Git tag created: `git tag v0.1.0`

---

## Contributing Workflow

### Development Process

**1. Create Branch:**
```bash
git checkout -b feature/my-feature
```

**2. Make Changes:**
- Write code
- Add tests
- Update documentation

**3. Run Tests:**
```bash
cargo test
npm test
```

**4. Commit:**
```bash
git add .
git commit -m "feat: add my feature"
```

**5. Push:**
```bash
git push origin feature/my-feature
```

**6. Create Pull Request:**
- Open PR on GitHub
- Fill in PR template
- Request review

### Commit Message Convention

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Tests
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `chore`: Maintenance

**Examples:**
```
feat(search): add date range filtering
fix(indexing): handle permission errors gracefully
docs(api): update search command documentation
test(db): add FTS5 ranking tests
```

---

## Code Style

### Rust Style

**Follow Rust conventions:**
```bash
# Format code
cargo fmt

# Check lints
cargo clippy -- -D warnings
```

**Key guidelines:**
- Use `Result<T, CortexError>` for errors
- No `unwrap()` in production code
- Comprehensive error messages
- Document public APIs

**Example:**
```rust
/// Search files using FTS5 full-text search.
///
/// # Arguments
/// * `conn` - Database connection
/// * `query` - Search query string
/// * `limit` - Maximum results to return
///
/// # Returns
/// Vector of search results ranked by relevance
///
/// # Errors
/// Returns `CortexError::InvalidQuery` if query is empty
pub fn search_files_fts(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>, CortexError> {
    // Implementation
}
```

### TypeScript Style

**Use Prettier and ESLint:**
```bash
npm run format
npm run lint
```

**Key guidelines:**
- Type everything (no `any`)
- Use async/await (not `.then()`)
- Destructure where appropriate
- Functional style preferred

**Example:**
```typescript
// Good
async function searchFiles(query: string): Promise<SearchResults> {
  const results = await invoke('search_files', { query });
  return results;
}

// Bad
function searchFiles(query: any) {
  return invoke('search_files', { query }).then((results: any) => {
    return results;
  });
}
```

### Documentation

**Rust:**
- Use `///` for doc comments
- Include examples in docs
- Document errors

**TypeScript:**
- Use JSDoc for complex functions
- Type definitions are primary docs
- Comment non-obvious logic

---

## Troubleshooting

### Build Issues

**Problem: "cannot find -lwebkit2gtk-4.1"**

**Solution:**
```bash
sudo apt-get install libwebkit2gtk-4.1-dev
```

**Problem: "linker `cc` not found"**

**Solution:**
```bash
sudo apt-get install build-essential
```

**Problem: Frontend build fails**

**Solution:**
```bash
rm -rf node_modules package-lock.json
npm install
```

### Runtime Issues

**Problem: Database locked**

**Solution:**
```bash
# Kill all Cortex processes
pkill -9 cortex

# Remove lock files
rm ~/.cortex/db.sqlite-wal
rm ~/.cortex/db.sqlite-shm
```

**Problem: High memory usage during indexing**

**Solution:**
- Reduce batch size in code
- Index fewer files at once
- Check for memory leaks with valgrind

**Problem: Tests failing intermittently**

**Causes:**
- Race conditions
- Timing-dependent tests
- Shared state between tests

**Solutions:**
- Use isolated test databases
- Add proper cleanup
- Use `tokio::test` for async tests

### Development Environment

**Problem: Hot reload not working**

**Solution:**
```bash
# Restart dev server
npm run dev

# Clear Vite cache
rm -rf .svelte-kit
```

**Problem: Tauri commands not found**

**Verify:**
1. Command registered in `main.rs`
2. Command module imported
3. Frontend using correct name

**Problem: Events not received**

**Check:**
1. Event name matches exactly
2. Listener created before event emitted
3. Cleanup listeners on unmount

---

## Additional Resources

### Documentation

- [Tauri Docs](https://tauri.app/v2/guides/)
- [Svelte 5 Docs](https://svelte-5-preview.vercel.app/docs)
- [SQLite FTS5](https://www.sqlite.org/fts5.html)
- [Rust Book](https://doc.rust-lang.org/book/)

### Project Docs

- [User Guide](USER_GUIDE.md) - End-user documentation
- [API Reference](API_REFERENCE.md) - Command reference
- [Performance Guide](../PERFORMANCE.md) - Optimization
- [Testing Guide](../TESTING.md) - Test suite

### Tools

- [Rust Analyzer](https://rust-analyzer.github.io/) - IDE support
- [Tauri CLI](https://tauri.app/v2/reference/cli/) - Build tools
- [SQLite Browser](https://sqlitebrowser.org/) - Database inspection

---

**Cortex Developer Guide v0.1.0** | [Report Issue](https://github.com/yourusername/cortex/issues) | [Edit on GitHub](https://github.com/yourusername/cortex/edit/main/docs/DEVELOPER_GUIDE.md)
