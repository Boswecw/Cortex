# CORTEX FEATURE PROMPTS

These prompts are designed to be used with VS Code Claude after loading the CONTEXT.md and STARTER_PROMPT.md files.

---

## 📁 PROMPT: RAKE_INTEGRATION.md

```markdown
# IMPLEMENT: Rake Integration for Cortex

## Objective
Add the ability for Cortex to export indexed files and embeddings to the Rake ingestion pipeline for storage in DataForge.

## Context from CONTEXT.md
- Cortex stores embeddings in SQLite using OpenAI text-embedding-3-small
- Rake expects a specific JSON format with chunks, embeddings, and metadata
- Multi-tenant support requires tenant_id configuration

## Requirements

### 1. Rust Export Module
Create `src-tauri/src/export/rake_exporter.rs`:

```rust
// Key structures needed:
pub struct RakeExport {
    version: String,          // "1.0"
    source: String,           // "cortex_local"
    tenant_id: String,        // User-configured
    export_timestamp: DateTime<Utc>,
    metadata: RakeExportMetadata,
    chunks: Vec<RakeChunk>,
}

pub struct RakeChunk {
    id: String,
    document_id: String,
    content: String,
    embedding: Option<Vec<f32>>,  // Include if pre-computed
    position: u32,
    token_count: u32,
    metadata: RakeChunkMetadata,
}
```

### 2. Tauri Commands
Add to `src-tauri/src/commands/export.rs`:
- `export_rake_package(collection_id, tenant_id, output_path)`
- `get_export_preview(collection_id)`

### 3. UI Component
Create `src/lib/components/RakeExportPanel.svelte`:
- Tenant ID input
- Collection selector (or "All Files")
- Include embeddings toggle
- Export preview (file count, chunk count, estimated size)
- Export button with progress

### 4. Integration Points
- Wire into existing Sidebar.svelte
- Add to Command Palette: "Export: Rake Package"

## Deliverables
- [ ] `src-tauri/src/export/mod.rs`
- [ ] `src-tauri/src/export/rake_exporter.rs`
- [ ] `src-tauri/src/commands/export.rs`
- [ ] `src/lib/components/RakeExportPanel.svelte`
- [ ] Updated `src-tauri/src/main.rs` (register commands)

## Testing
1. Index a test directory with mixed file types
2. Export with embeddings included
3. Validate JSON structure matches Rake spec
4. Import into Rake (if available) to verify

Begin implementation with the Rust export module.
```

---

## 📁 PROMPT: SMART_COLLECTIONS.md

```markdown
# IMPLEMENT: Smart Collections Engine

## Objective
Implement dynamic, rule-based file collections that automatically group files based on criteria like file type, path patterns, modification date, and content.

## Context from CONTEXT.md
- Sidebar currently has hardcoded nav items
- SQLite database already has files table with metadata
- This is the "killer feature" differentiating Cortex from basic search

## Requirements

### 1. Database Schema
Add to `src-tauri/src/db/schema.rs`:

```sql
CREATE TABLE collections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT,
    rules TEXT NOT NULL,  -- JSON array of rules
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE collection_files (
    collection_id TEXT,
    file_id TEXT,
    added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (collection_id, file_id),
    FOREIGN KEY (collection_id) REFERENCES collections(id),
    FOREIGN KEY (file_id) REFERENCES files(id)
);
```

### 2. Rule Engine
Create `src-tauri/src/collections/rules.rs`:

```rust
#[derive(Serialize, Deserialize)]
pub enum CollectionRule {
    FileType(Vec<String>),           // ["rs", "ts", "svelte"]
    PathContains(String),            // "src/components"
    PathMatches(String),             // Glob pattern
    ModifiedAfter(DateTime<Utc>),
    ModifiedBefore(DateTime<Utc>),
    SizeGreaterThan(u64),
    SizeLessThan(u64),
    ContentContains(String),         // FTS search
    HasEmbedding(bool),
}

pub struct RuleEngine {
    pub fn evaluate(&self, file: &File, rules: &[CollectionRule]) -> bool;
    pub fn refresh_collection(&self, collection_id: &str) -> Result<usize>;
}
```

### 3. Tauri Commands
- `create_collection(name, rules) -> CollectionId`
- `get_collections() -> Vec<Collection>`
- `get_collection_files(id) -> Vec<File>`
- `update_collection(id, name, rules)`
- `delete_collection(id)`
- `refresh_collection(id) -> RefreshResult`

### 4. UI Components
- `CollectionList.svelte` — Sidebar section showing all collections
- `SmartCollectionEditor.svelte` — Modal for creating/editing
- `CollectionView.svelte` — Page showing collection files
- Update `Sidebar.svelte` to include collections section

### 5. Built-in Templates
Pre-configured collections:
- "Source Code" — .rs, .ts, .js, .svelte, .py
- "Documentation" — .md, .txt, .pdf
- "Recently Modified" — Modified in last 7 days
- "Large Files" — > 1MB
- "Needs Embedding" — Has no embedding yet

## Deliverables
- [ ] Database migration for collections tables
- [ ] `src-tauri/src/collections/mod.rs`
- [ ] `src-tauri/src/collections/rules.rs`
- [ ] Tauri commands in `src-tauri/src/commands/collections.rs`
- [ ] `src/lib/components/CollectionList.svelte`
- [ ] `src/lib/components/SmartCollectionEditor.svelte`
- [ ] `src/routes/collections/[id]/+page.svelte`
- [ ] Updated Sidebar with collections section

Begin with the database schema and rule engine.
```

---

## 📁 PROMPT: COMMAND_PALETTE.md

```markdown
# IMPLEMENT: Command Palette

## Objective
Add a Cmd+P / Ctrl+P command palette for power users to quickly access any Cortex functionality.

## Context from CONTEXT.md
- Currently only Cmd+K for search focus exists
- Plan specifies full command palette with all commands
- Pattern: Similar to VS Code, Raycast, Spotlight

## Requirements

### 1. Command Registry
Create `src/lib/stores/commands.ts`:

```typescript
interface Command {
    id: string;
    label: string;
    shortcut?: string;
    icon?: string;
    category: 'navigation' | 'search' | 'file' | 'collection' | 'export' | 'settings';
    action: () => void | Promise<void>;
    keywords?: string[];  // For fuzzy search
}

export const commands = writable<Command[]>([
    {
        id: 'search:focus',
        label: 'Focus Search',
        shortcut: '⌘K',
        category: 'search',
        action: () => focusSearch()
    },
    {
        id: 'search:semantic',
        label: 'Semantic Search',
        shortcut: '⌘⇧S',
        category: 'search',
        action: () => openSemanticSearch()
    },
    // ... more commands
]);
```

### 2. CommandPalette Component
Create `src/lib/components/CommandPalette.svelte`:

Features:
- Opens with Cmd+P / Ctrl+P
- Fuzzy search through commands
- Arrow key navigation
- Enter to execute
- Escape to close
- Category grouping
- Recent commands section
- Keyboard shortcut display

```svelte
<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { commands } from '$lib/stores/commands';
    
    let isOpen = false;
    let query = '';
    let selectedIndex = 0;
    let inputElement: HTMLInputElement;
    
    // Fuzzy filter logic
    $: filteredCommands = filterCommands($commands, query);
    
    function handleKeydown(e: KeyboardEvent) {
        if ((e.metaKey || e.ctrlKey) && e.key === 'p') {
            e.preventDefault();
            isOpen = !isOpen;
            if (isOpen) {
                setTimeout(() => inputElement?.focus(), 0);
            }
        }
    }
</script>
```

### 3. Full Command List

Navigation:
- Go to Search (/)
- Go to Recent (/recent)
- Go to Starred (/starred)
- Go to Collection... (/collections/[id])
- Go to Settings (/settings)

Search:
- Focus Search (Cmd+K)
- Semantic Search (Cmd+Shift+S)
- Clear Filters
- Save Current Filter

File:
- Open in Default App
- Reveal in Finder
- Copy Path
- Star/Unstar
- Add to Collection...

Collection:
- New Collection
- Edit Collection
- Delete Collection
- Refresh Collection

Export:
- Export VS Code Context
- Export Rake Package
- Copy Quick Prompt

Settings:
- Toggle Theme
- Open Settings
- Reindex All Files
- Clear Database

### 4. Integration
- Add to `+layout.svelte` for global keyboard binding
- Position as modal overlay with backdrop blur

## Deliverables
- [ ] `src/lib/stores/commands.ts`
- [ ] `src/lib/components/CommandPalette.svelte`
- [ ] Updated `src/routes/+layout.svelte`
- [ ] Fuzzy search utility in `src/lib/utils/fuzzy.ts`

Begin with the command store and basic palette UI.
```

---

## 📁 PROMPT: CONTEXT_GENERATOR.md

```markdown
# IMPLEMENT: Context File Generator

## Objective
Build the Rust module that generates CONTEXT.md files from indexed project files.

## Requirements

### 1. Context Builder Module
Create `src-tauri/src/export/context_builder.rs`:

```rust
pub struct ContextBuilder {
    db: Database,
}

impl ContextBuilder {
    pub async fn build_context(&self, params: ContextParams) -> Result<String> {
        let mut sections = Vec::new();
        
        // 1. Project header
        sections.push(self.build_header(&params)?);
        
        // 2. Architecture detection
        sections.push(self.detect_architecture(&params).await?);
        
        // 3. Project structure tree
        sections.push(self.build_directory_tree(&params).await?);
        
        // 4. Key files with summaries
        sections.push(self.build_file_summaries(&params).await?);
        
        // 5. Dependencies (parse Cargo.toml, package.json, etc.)
        sections.push(self.parse_dependencies(&params).await?);
        
        // 6. Code patterns detected
        sections.push(self.detect_patterns(&params).await?);
        
        Ok(sections.join("\n\n---\n\n"))
    }
}
```

### 2. Architecture Detection
Analyze file structure to detect:
- Framework (Tauri, Next.js, FastAPI, SvelteKit, etc.)
- Language (Rust, TypeScript, Python, etc.)
- Architecture pattern (MVC, Clean Architecture, etc.)
- Entry points (main.rs, index.ts, app.py)

### 3. File Summarization
For each key file:
- Extract first docstring/comment block
- Identify exports/public API
- Count lines of code
- Detect purpose from filename/path

### 4. Dependency Parsing
Parse and summarize:
- Cargo.toml → Rust dependencies
- package.json → Node dependencies
- requirements.txt → Python dependencies
- go.mod → Go dependencies

### 5. Pattern Detection
Identify common patterns:
- Tauri commands (`#[tauri::command]`)
- React/Svelte components
- API routes
- Database models
- Test files

## Deliverables
- [ ] `src-tauri/src/export/context_builder.rs`
- [ ] `src-tauri/src/export/architecture_detector.rs`
- [ ] `src-tauri/src/export/dependency_parser.rs`
- [ ] Tests in `src-tauri/src/export/tests.rs`

Begin with the main ContextBuilder struct and header generation.
```

---

## Usage Instructions

1. **Load Context First**: Always start by loading CONTEXT.md into VS Code Claude
2. **Load Starter Prompt**: Copy STARTER_PROMPT.md to establish the working relationship
3. **Choose Feature Prompt**: Select the appropriate feature prompt for your task
4. **Iterate**: Use follow-up prompts to refine and complete the implementation

These prompts are designed to work together as a coherent development workflow.
