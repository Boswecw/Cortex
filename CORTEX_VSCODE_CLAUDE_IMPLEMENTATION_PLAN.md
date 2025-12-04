# CORTEX VS CODE CLAUDE EXPORT FEATURE
## Implementation Plan: Context Files & Prompt Downloads

**Project:** Cortex Local File Intelligence System  
**Feature:** VS Code Claude Integration Export  
**Status:** 📋 Planning  
**Owner:** Charles Boswell  
**Date:** December 4, 2025  
**Estimated Effort:** 40-60 hours

---

## 🎯 EXECUTIVE SUMMARY

This feature transforms Cortex from a local search tool into a **development acceleration platform** by enabling users to export:

1. **Context Files** — Comprehensive project knowledge bundles for VS Code Claude
2. **Prompt Downloads** — Pre-built, feature-specific implementation prompts
3. **Rake Integration Packages** — Export local knowledge directly to the Forge ecosystem

This creates a powerful flywheel: Cortex indexes your projects → exports context to Claude → Claude implements features faster → including improvements to Cortex itself.

---

## 🏗️ ARCHITECTURE OVERVIEW

### Export Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CORTEX DESKTOP APP                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────────┐   │
│  │   Indexed   │───▶│   Export     │───▶│  Generated Files    │   │
│  │   Files     │    │   Engine     │    │                     │   │
│  │             │    │              │    │  • CONTEXT.md       │   │
│  │  • Code     │    │  • Selector  │    │  • STARTER_PROMPT   │   │
│  │  • Docs     │    │  • Bundler   │    │  • FEATURE_PROMPTS  │   │
│  │  • Config   │    │  • Formatter │    │  • .claude/         │   │
│  └─────────────┘    └──────────────┘    └─────────────────────┘   │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    SMART COLLECTIONS                         │   │
│  │  "Backend Code"  "Frontend Components"  "Documentation"     │   │
│  │       ↓                  ↓                    ↓              │   │
│  │   Export as          Export as           Export as          │   │
│  │   API Context       UI Context         Knowledge Base       │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      VS CODE CLAUDE                                 │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  Load CONTEXT.md → Full project understanding               │    │
│  │  Load STARTER_PROMPT.md → Begin implementation              │    │
│  │  Load FEATURE_PROMPTS/ → Task-specific guidance             │    │
│  └────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      RAKE PIPELINE (Optional)                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  Cortex exports pre-chunked, pre-embedded content          │    │
│  │  Rake receives → validates → stores to DataForge           │    │
│  └────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 📦 DELIVERABLES

### 1. Context File Generator

A Rust module that generates a comprehensive `CONTEXT.md` containing:

```markdown
# PROJECT CONTEXT: {project_name}

## Project Overview
- Type: {detected_type} (Rust/SvelteKit Desktop App)
- Framework: {frameworks}
- Entry Points: {main_files}

## Architecture Summary
{auto_generated_architecture_summary}

## Key Files
{critical_files_with_summaries}

## Dependencies
{parsed_cargo_toml_and_package_json}

## Code Patterns
{detected_patterns}

## Current State
- Total Files: {count}
- Last Indexed: {timestamp}
- Coverage: {percentage}%
```

### 2. Starter Prompt Generator

Creates `STARTER_PROMPT.md` following the Rake pattern:

```markdown
# {PROJECT_NAME} IMPLEMENTATION - INITIAL PROMPT FOR VS CODE CLAUDE

## Context
I'm working on **{project_name}**, a {description}.
{architecture_summary}

## Your Role
You are my implementation partner. I have comprehensive context
from Cortex that gives you full project understanding.

## Critical Requirements
{project_specific_requirements}

## Implementation Workflow
{workflow_instructions}

## Project Structure
{directory_tree}

## Let's Begin!
{starting_point}
```

### 3. Feature Prompt Library

Individual prompt files for common tasks:

```
.cortex-export/
├── CONTEXT.md
├── STARTER_PROMPT.md
├── prompts/
│   ├── ADD_FEATURE.md
│   ├── FIX_BUG.md
│   ├── REFACTOR.md
│   ├── ADD_TESTS.md
│   ├── DOCUMENTATION.md
│   └── RAKE_INTEGRATION.md
└── .claude/
    └── settings.json
```

### 4. Rake Export Package

For syncing to Forge ecosystem:

```json
{
  "export_version": "1.0",
  "source": "cortex_local",
  "tenant_id": "{user_configured}",
  "chunks": [
    {
      "id": "chunk_uuid",
      "content": "...",
      "embedding": [0.123, ...],
      "metadata": {
        "file_path": "/src/main.rs",
        "file_type": "rust",
        "modified_at": "2025-12-04T...",
        "collection": "backend_code"
      }
    }
  ]
}
```

---

## 🔧 IMPLEMENTATION PHASES

### Phase 1: Core Export Engine (12-16 hours)

#### 1.1 Export Module Structure

```
src-tauri/src/
├── export/
│   ├── mod.rs              # Module root
│   ├── context_builder.rs  # CONTEXT.md generator
│   ├── prompt_builder.rs   # Prompt generators
│   ├── bundler.rs          # File packager
│   └── rake_exporter.rs    # Rake format exporter
```

#### 1.2 Rust Commands (Tauri IPC)

```rust
// src-tauri/src/commands/export.rs

#[tauri::command]
pub async fn export_vscode_context(
    collection_id: Option<String>,
    include_embeddings: bool,
    output_path: String,
) -> Result<ExportResult, String> {
    // Generate context bundle
}

#[tauri::command]
pub async fn export_rake_package(
    collection_id: Option<String>,
    tenant_id: String,
    output_path: String,
) -> Result<RakeExportResult, String> {
    // Generate Rake-compatible export
}

#[tauri::command]
pub async fn get_export_preview(
    collection_id: Option<String>,
) -> Result<ExportPreview, String> {
    // Preview what will be exported
}

#[tauri::command]
pub async fn list_prompt_templates() -> Result<Vec<PromptTemplate>, String> {
    // List available prompt templates
}
```

#### 1.3 Context Builder Implementation

```rust
// src-tauri/src/export/context_builder.rs

pub struct ContextBuilder {
    db: Database,
    config: ExportConfig,
}

impl ContextBuilder {
    pub async fn build_context(&self, params: ContextParams) -> Result<String> {
        let mut context = String::new();
        
        // 1. Project header
        context.push_str(&self.build_header(&params)?);
        
        // 2. Architecture summary (auto-detected)
        context.push_str(&self.detect_architecture(&params).await?);
        
        // 3. Key files with summaries
        context.push_str(&self.build_file_summaries(&params).await?);
        
        // 4. Dependencies
        context.push_str(&self.parse_dependencies(&params).await?);
        
        // 5. Code patterns
        context.push_str(&self.detect_patterns(&params).await?);
        
        Ok(context)
    }
    
    async fn detect_architecture(&self, params: &ContextParams) -> Result<String> {
        // Analyze file structure to detect:
        // - Framework (Tauri, Next.js, FastAPI, etc.)
        // - Architecture pattern (MVC, Clean, Hexagonal)
        // - Entry points
        // - Layer separation
    }
}
```

### Phase 2: UI Components (8-12 hours)

#### 2.1 Export Panel Component

```svelte
<!-- src/lib/components/ExportPanel.svelte -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { save } from '@tauri-apps/api/dialog';
  
  export let collectionId: string | null = null;
  
  let exportType: 'vscode' | 'rake' = 'vscode';
  let includeEmbeddings = false;
  let tenantId = '';
  let preview: ExportPreview | null = null;
  let exporting = false;
  
  async function loadPreview() {
    preview = await invoke('get_export_preview', { collectionId });
  }
  
  async function handleExport() {
    exporting = true;
    const outputPath = await save({
      defaultPath: exportType === 'vscode' 
        ? '.cortex-export' 
        : 'cortex-rake-export.json',
      filters: exportType === 'vscode'
        ? [{ name: 'Folder', extensions: [''] }]
        : [{ name: 'JSON', extensions: ['json'] }]
    });
    
    if (outputPath) {
      if (exportType === 'vscode') {
        await invoke('export_vscode_context', {
          collectionId,
          includeEmbeddings,
          outputPath
        });
      } else {
        await invoke('export_rake_package', {
          collectionId,
          tenantId,
          outputPath
        });
      }
    }
    exporting = false;
  }
</script>

<div class="export-panel neural-card">
  <h3>Export for AI Development</h3>
  
  <div class="export-type-selector">
    <button 
      class:active={exportType === 'vscode'}
      on:click={() => exportType = 'vscode'}
    >
      <span class="icon">💻</span>
      VS Code Claude
    </button>
    <button 
      class:active={exportType === 'rake'}
      on:click={() => exportType = 'rake'}
    >
      <span class="icon">🔥</span>
      Rake / Forge
    </button>
  </div>
  
  {#if preview}
    <div class="preview-stats">
      <div class="stat">
        <span class="label">Files</span>
        <span class="value">{preview.fileCount}</span>
      </div>
      <div class="stat">
        <span class="label">Chunks</span>
        <span class="value">{preview.chunkCount}</span>
      </div>
      <div class="stat">
        <span class="label">Est. Size</span>
        <span class="value">{preview.estimatedSize}</span>
      </div>
    </div>
  {/if}
  
  {#if exportType === 'vscode'}
    <div class="options">
      <label>
        <input type="checkbox" bind:checked={includeEmbeddings} />
        Include embeddings (for semantic search in prompts)
      </label>
    </div>
    <p class="description">
      Generates CONTEXT.md, STARTER_PROMPT.md, and feature prompts
      for VS Code Claude development sessions.
    </p>
  {:else}
    <div class="options">
      <label>
        Forge Tenant ID
        <input type="text" bind:value={tenantId} placeholder="your-tenant-id" />
      </label>
    </div>
    <p class="description">
      Exports pre-embedded chunks for direct ingestion into
      the Rake pipeline and DataForge storage.
    </p>
  {/if}
  
  <button 
    class="export-btn neural-gold"
    on:click={handleExport}
    disabled={exporting}
  >
    {#if exporting}
      Exporting...
    {:else}
      Export {exportType === 'vscode' ? 'Context Bundle' : 'Rake Package'}
    {/if}
  </button>
</div>
```

#### 2.2 Prompt Template Selector

```svelte
<!-- src/lib/components/PromptSelector.svelte -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { writeText } from '@tauri-apps/api/clipboard';
  
  let templates: PromptTemplate[] = [];
  let selectedTemplate: PromptTemplate | null = null;
  let customVariables: Record<string, string> = {};
  let generatedPrompt = '';
  
  onMount(async () => {
    templates = await invoke('list_prompt_templates');
  });
  
  async function generatePrompt() {
    generatedPrompt = await invoke('generate_prompt', {
      templateId: selectedTemplate.id,
      variables: customVariables,
      collectionId
    });
  }
  
  async function copyToClipboard() {
    await writeText(generatedPrompt);
  }
</script>

<div class="prompt-selector">
  <h4>Quick Prompts</h4>
  
  <div class="template-grid">
    {#each templates as template}
      <button 
        class="template-card"
        class:selected={selectedTemplate?.id === template.id}
        on:click={() => selectedTemplate = template}
      >
        <span class="icon">{template.icon}</span>
        <span class="name">{template.name}</span>
        <span class="description">{template.shortDescription}</span>
      </button>
    {/each}
  </div>
  
  {#if selectedTemplate}
    <div class="template-config">
      <h5>{selectedTemplate.name}</h5>
      <p>{selectedTemplate.description}</p>
      
      {#each selectedTemplate.variables as variable}
        <label>
          {variable.label}
          <input 
            type="text" 
            bind:value={customVariables[variable.key]}
            placeholder={variable.placeholder}
          />
        </label>
      {/each}
      
      <button on:click={generatePrompt}>Generate Prompt</button>
    </div>
  {/if}
  
  {#if generatedPrompt}
    <div class="generated-prompt">
      <div class="prompt-header">
        <span>Generated Prompt</span>
        <button on:click={copyToClipboard}>📋 Copy</button>
      </div>
      <pre>{generatedPrompt}</pre>
    </div>
  {/if}
</div>
```

### Phase 3: Prompt Templates (6-8 hours)

#### 3.1 Template Storage Structure

```
src-tauri/resources/prompts/
├── manifest.json           # Template registry
├── vscode_claude/
│   ├── starter.md.hbs      # Handlebars template
│   ├── add_feature.md.hbs
│   ├── fix_bug.md.hbs
│   ├── refactor.md.hbs
│   ├── add_tests.md.hbs
│   └── rake_integration.md.hbs
└── project_types/
    ├── rust_tauri.md.hbs
    ├── sveltekit.md.hbs
    ├── python_fastapi.md.hbs
    └── generic.md.hbs
```

#### 3.2 Template Examples

**Starter Template (`starter.md.hbs`):**

```handlebars
# {{project_name}} IMPLEMENTATION - VS CODE CLAUDE STARTER

**Generated by Cortex on {{generated_date}}**

---

## Context

I'm working on **{{project_name}}**, {{project_description}}.

### Technology Stack
{{#each tech_stack}}
- **{{this.name}}**: {{this.version}} — {{this.purpose}}
{{/each}}

### Architecture
{{architecture_summary}}

## Your Role

You are my implementation partner. Cortex has indexed this project
and provided you with comprehensive context. Your job is to help me
implement features, fix bugs, and improve the codebase.

## Critical Requirements

**Every file you create or modify must:**
{{#each requirements}}
{{@index}}. {{this}}
{{/each}}

## Project Structure

```
{{directory_tree}}
```

## Key Files Reference

{{#each key_files}}
### {{this.path}}
{{this.summary}}
{{/each}}

## Current Focus

{{#if current_focus}}
{{current_focus}}
{{else}}
Ready to begin implementation. What would you like to work on?
{{/if}}

---

**Quick Commands:**
- "Show me the architecture" → I'll explain the project structure
- "Let's implement [feature]" → I'll create the implementation plan
- "Review this code" → I'll analyze and suggest improvements
- "Help me debug [issue]" → I'll investigate and propose fixes
```

**Add Feature Template (`add_feature.md.hbs`):**

```handlebars
# FEATURE IMPLEMENTATION: {{feature_name}}

## Feature Request

{{feature_description}}

## Existing Context

Based on the indexed codebase, here's what's relevant:

### Related Files
{{#each related_files}}
- `{{this.path}}` — {{this.relevance}}
{{/each}}

### Existing Patterns
{{#each existing_patterns}}
- {{this.pattern}}: Used in {{this.location}}
{{/each}}

## Implementation Plan

Please implement this feature following these steps:

1. **Understand**: Review the related files above
2. **Plan**: Outline the changes needed
3. **Implement**: Create/modify files following project patterns
4. **Test**: Include test coverage
5. **Document**: Update any affected documentation

## Constraints

{{#each constraints}}
- {{this}}
{{/each}}

## Expected Deliverables

{{#each deliverables}}
- [ ] {{this}}
{{/each}}

---

**Begin implementation when ready. Ask clarifying questions if needed.**
```

**Rake Integration Template (`rake_integration.md.hbs`):**

```handlebars
# RAKE INTEGRATION IMPLEMENTATION

## Objective

Implement sync functionality between Cortex and the Rake ingestion pipeline.

## Architecture Context

**Cortex** (Local Desktop App - Rust/Tauri/SvelteKit):
- SQLite database with FTS5
- Local embeddings (OpenAI API)
- Smart Collections feature

**Rake** (Cloud Service - Python/FastAPI):
- 5-stage pipeline: Fetch → Clean → Chunk → Embed → Store
- Multi-tenant support
- DataForge integration

## Integration Points

### 1. Cortex Export Format

```typescript
interface CortexExport {
  version: "1.0";
  source: "cortex_local";
  tenant_id: string;
  export_timestamp: string;
  chunks: CortexChunk[];
}

interface CortexChunk {
  id: string;
  content: string;
  embedding: number[] | null;  // Pre-computed if available
  metadata: {
    file_path: string;
    file_type: string;
    modified_at: string;
    collection_id?: string;
    collection_name?: string;
  };
}
```

### 2. Rake Source Adapter

Create `rake/sources/cortex_local.py`:
- Accept CortexExport JSON
- Skip EMBED stage if embeddings present
- Map metadata to Rake format
- Handle incremental syncs

### 3. Sync Modes

- **Full Export**: All indexed files
- **Collection Export**: Specific Smart Collection
- **Incremental**: Only changes since last sync

## Implementation Files

### Cortex Side (Rust)
{{#each cortex_files}}
- `{{this.path}}` — {{this.purpose}}
{{/each}}

### Rake Side (Python)
{{#each rake_files}}
- `{{this.path}}` — {{this.purpose}}
{{/each}}

## Your Task

{{task_description}}

---

**Reference the RAKE_DEVELOPMENT_GUIDE.md for Rake architecture details.**
```

### Phase 4: Rake Exporter (8-10 hours)

#### 4.1 Rake Export Format Handler

```rust
// src-tauri/src/export/rake_exporter.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize)]
pub struct RakeExport {
    version: String,
    source: String,
    tenant_id: String,
    export_timestamp: DateTime<Utc>,
    metadata: RakeExportMetadata,
    chunks: Vec<RakeChunk>,
}

#[derive(Serialize)]
pub struct RakeExportMetadata {
    cortex_version: String,
    collection_id: Option<String>,
    collection_name: Option<String>,
    total_files: u32,
    total_chunks: u32,
    has_embeddings: bool,
    embedding_model: Option<String>,
}

#[derive(Serialize)]
pub struct RakeChunk {
    id: String,
    document_id: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    embedding: Option<Vec<f32>>,
    position: u32,
    token_count: u32,
    metadata: RakeChunkMetadata,
}

#[derive(Serialize)]
pub struct RakeChunkMetadata {
    file_path: String,
    file_type: String,
    file_name: String,
    modified_at: DateTime<Utc>,
    collection_id: Option<String>,
}

impl RakeExporter {
    pub async fn export(
        &self,
        collection_id: Option<String>,
        tenant_id: String,
        include_embeddings: bool,
    ) -> Result<RakeExport> {
        let files = match &collection_id {
            Some(id) => self.get_collection_files(id).await?,
            None => self.get_all_indexed_files().await?,
        };
        
        let mut chunks = Vec::new();
        
        for file in files {
            let file_chunks = self.get_file_chunks(&file.id).await?;
            
            for chunk in file_chunks {
                let embedding = if include_embeddings {
                    self.get_chunk_embedding(&chunk.id).await?
                } else {
                    None
                };
                
                chunks.push(RakeChunk {
                    id: chunk.id,
                    document_id: file.id.clone(),
                    content: chunk.content,
                    embedding,
                    position: chunk.position,
                    token_count: chunk.token_count,
                    metadata: RakeChunkMetadata {
                        file_path: file.path.clone(),
                        file_type: file.file_type.clone(),
                        file_name: file.name.clone(),
                        modified_at: file.modified_at,
                        collection_id: collection_id.clone(),
                    },
                });
            }
        }
        
        Ok(RakeExport {
            version: "1.0".to_string(),
            source: "cortex_local".to_string(),
            tenant_id,
            export_timestamp: Utc::now(),
            metadata: RakeExportMetadata {
                cortex_version: env!("CARGO_PKG_VERSION").to_string(),
                collection_id,
                collection_name: None, // Populated if collection specified
                total_files: files.len() as u32,
                total_chunks: chunks.len() as u32,
                has_embeddings: include_embeddings,
                embedding_model: if include_embeddings {
                    Some("text-embedding-3-small".to_string())
                } else {
                    None
                },
            },
            chunks,
        })
    }
}
```

### Phase 5: Testing & Polish (6-8 hours)

#### 5.1 Test Cases

```rust
// src-tauri/src/export/tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_context_generation() {
        let builder = ContextBuilder::new(test_db()).await;
        let context = builder.build_context(ContextParams::default()).await.unwrap();
        
        assert!(context.contains("# PROJECT CONTEXT"));
        assert!(context.contains("## Architecture Summary"));
        assert!(context.contains("## Key Files"));
    }
    
    #[tokio::test]
    async fn test_rake_export_format() {
        let exporter = RakeExporter::new(test_db()).await;
        let export = exporter.export(
            None,
            "test-tenant".to_string(),
            true
        ).await.unwrap();
        
        assert_eq!(export.version, "1.0");
        assert_eq!(export.source, "cortex_local");
        assert!(export.metadata.has_embeddings);
    }
    
    #[tokio::test]
    async fn test_prompt_template_rendering() {
        let renderer = PromptRenderer::new();
        let result = renderer.render(
            "starter",
            PromptVariables {
                project_name: "TestProject".to_string(),
                ..Default::default()
            }
        ).unwrap();
        
        assert!(result.contains("TestProject"));
        assert!(result.contains("VS CODE CLAUDE STARTER"));
    }
}
```

---

## 📊 INTEGRATION WITH CORTEX PHASES

This feature fits into Cortex Phase 3 (Advanced Features):

| Cortex Feature | Export Integration |
|----------------|-------------------|
| Smart Collections | Export specific collections |
| Duplicate Detection | Dedupe before export |
| Bulk Operations | Bulk export selection |
| Command Palette | Quick export commands |

### Command Palette Integration

```typescript
// Commands to add to CommandPalette.svelte
const exportCommands = [
  {
    id: 'export:vscode',
    label: 'Export: VS Code Claude Context',
    shortcut: 'Cmd+Shift+E',
    action: () => openExportPanel('vscode')
  },
  {
    id: 'export:rake',
    label: 'Export: Rake Package',
    shortcut: 'Cmd+Shift+R',
    action: () => openExportPanel('rake')
  },
  {
    id: 'export:copy-prompt',
    label: 'Export: Copy Quick Prompt',
    shortcut: 'Cmd+Shift+P',
    action: () => openPromptSelector()
  }
];
```

---

## 🚀 IMPLEMENTATION SCHEDULE

| Day | Phase | Hours | Deliverables |
|-----|-------|-------|--------------|
| 1-2 | Core Export Engine | 12 | Rust export module, Tauri commands |
| 3-4 | UI Components | 10 | ExportPanel, PromptSelector |
| 5 | Prompt Templates | 6 | Template system, 5+ templates |
| 6-7 | Rake Exporter | 10 | Full Rake format support |
| 8 | Testing & Polish | 6 | Tests, error handling, UX polish |
| **Total** | | **44** | Complete feature |

---

## 📋 ACCEPTANCE CRITERIA

### Context Export
- [ ] Generates valid CONTEXT.md with project overview
- [ ] Detects tech stack automatically
- [ ] Includes directory tree
- [ ] Summarizes key files
- [ ] Works with Smart Collections

### Prompt System
- [ ] 5+ prompt templates available
- [ ] Templates render with project context
- [ ] Copy to clipboard works
- [ ] Variables are customizable

### Rake Integration
- [ ] Valid JSON export format
- [ ] Embeddings included when requested
- [ ] Metadata properly mapped
- [ ] Tenant ID configurable

### UI/UX
- [ ] Export panel accessible from sidebar
- [ ] Preview shows stats before export
- [ ] Progress indicator during export
- [ ] Success/error feedback

---

## 🔗 RELATED DOCUMENTS

- `RAKE_DEVELOPMENT_GUIDE.md` — Rake architecture
- `RAKE_V1_VSCODE_CLAUDE_GUIDE.md` — VS Code Claude patterns
- `Cortex_Project_Assessment.docx` — Cortex status
- `FORGE_SAAS_ARCHITECTURE_V3.md` — Forge ecosystem

---

*This implementation plan transforms Cortex into a development acceleration platform, creating a powerful synergy between local file intelligence and AI-assisted development.*
