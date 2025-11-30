# Cortex Phase 2: AI Features - Progress Summary

**Last Updated:** 2025-11-30
**Status:** 🚀 0% Complete (0/8 tasks) - PHASE 2 STARTING!
**Time Invested:** ~0 hours
**Code Written:** ~0 lines

---

## 📋 Phase 2 Overview

Phase 2 builds on the solid foundation of Phase 1's Desktop UI by adding intelligent features powered by AI and machine learning. The goal is to make Cortex not just fast, but smart—understanding content semantically, automatically organizing files, and providing insights beyond keyword matching.

### Core Objectives

1. **Semantic Search** - Understand meaning, not just keywords
2. **Smart Collections** - Auto-organize files by content similarity
3. **Auto-Tagging** - Intelligent file categorization
4. **Saved Searches** - Persistent query management
5. **Advanced Analytics** - Usage insights and file relationships

---

## 📊 Task Breakdown (8 Tasks)

### AI-201: Embedding Model Integration ⏳
**Priority:** P0 - Foundation for all AI features
**Estimated Time:** 4-6 hours

**Goal:** Integrate a lightweight embedding model for semantic understanding

**Implementation Options:**
1. **Local Model** (Recommended):
   - `sentence-transformers/all-MiniLM-L6-v2` (22MB, fast)
   - Run via ONNX Runtime in Rust
   - 100% offline, privacy-first

2. **Alternative:**
   - `fastembed-rs` crate (Rust-native embeddings)
   - Pre-quantized models for speed

**Technical Requirements:**
- Add dependencies: `ort` (ONNX Runtime), `ndarray`, `tokenizers`
- Download model files to `~/.cortex/models/`
- Create `EmbeddingService` in Rust backend
- Batch processing for efficiency (32-64 files at once)
- Vector dimension: 384 (MiniLM-L6-v2)

**Database Schema Changes:**
```sql
-- New table for embeddings
CREATE TABLE file_embeddings (
    file_id INTEGER PRIMARY KEY,
    embedding BLOB NOT NULL,  -- 384 floats * 4 bytes = 1.5KB per file
    model_version TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

-- Index for faster lookups
CREATE INDEX idx_embeddings_model ON file_embeddings(model_version);
```

**Tauri Commands:**
- `generate_embeddings(file_ids: Vec<i64>)` - Generate embeddings for files
- `get_embedding_status()` - Check how many files have embeddings

**Success Criteria:**
- ✅ Model loads in <2 seconds on startup
- ✅ Embedding generation: >10 files/second
- ✅ Vector storage: <2KB per file
- ✅ Zero network calls (100% offline)

---

### AI-202: Semantic Search Implementation ⏳
**Priority:** P0 - Core AI feature
**Estimated Time:** 3-4 hours

**Goal:** Enable "find files similar to this" semantic search

**Features:**
- **Similarity Search:**
  - Input: Query text or reference file ID
  - Output: Files ranked by cosine similarity
  - Threshold: Configurable (default 0.7)

- **Hybrid Search:**
  - Combine FTS5 (keyword) + embeddings (semantic)
  - Weighted scoring: 60% semantic, 40% keyword
  - Fallback to FTS5 if no embeddings

**Implementation:**
```rust
// Similarity calculation
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|y| y * y).sum::<f32>().sqrt();
    dot / (norm_a * norm_b)
}

// Search function
pub async fn semantic_search(
    query: &str,
    limit: usize,
    threshold: f32
) -> Result<Vec<SearchResult>> {
    // 1. Generate query embedding
    let query_vec = embedding_service.embed(query).await?;

    // 2. Get all file embeddings from DB
    let file_embeddings = db.get_all_embeddings().await?;

    // 3. Calculate similarities
    let mut scored: Vec<(i64, f32)> = file_embeddings
        .iter()
        .map(|(id, vec)| (*id, cosine_similarity(&query_vec, vec)))
        .filter(|(_, score)| *score >= threshold)
        .collect();

    // 4. Sort by score descending
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // 5. Take top N and fetch file details
    let file_ids: Vec<i64> = scored.iter().take(limit).map(|(id, _)| *id).collect();
    db.get_files_by_ids(file_ids).await
}
```

**UI Integration:**
- Add "Semantic" toggle to search bar
- Show similarity scores (0-100%)
- "Find Similar" button on each search result
- Visual indicator when semantic search is active

**Tauri Commands:**
- `semantic_search(query: String, limit: usize, threshold: f32)`
- `find_similar(file_id: i64, limit: usize)`

**Success Criteria:**
- ✅ Search latency: <500ms for 10K files
- ✅ Relevant results for concept queries ("authentication code", "meeting notes")
- ✅ Smooth UI toggle between keyword/semantic modes

---

### AI-203: Smart Collections ⏳
**Priority:** P1 - Organizational feature
**Estimated Time:** 5-6 hours

**Goal:** Automatically group similar files into collections

**Features:**
- **Auto-Collections:**
  - Clustering algorithm (K-means or DBSCAN)
  - Group files by embedding similarity
  - Suggest collection names using LLM or heuristics

- **Manual Collections:**
  - User can create/edit/delete collections
  - Drag-and-drop files into collections
  - Color-coded badges

**Database Schema:**
```sql
CREATE TABLE collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT,  -- hex color code
    auto_generated BOOLEAN DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE collection_files (
    collection_id INTEGER NOT NULL,
    file_id INTEGER NOT NULL,
    added_at INTEGER NOT NULL,
    PRIMARY KEY (collection_id, file_id),
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

CREATE INDEX idx_collection_files_collection ON collection_files(collection_id);
CREATE INDEX idx_collection_files_file ON collection_files(file_id);
```

**Clustering Algorithm:**
```rust
use smartcore::cluster::kmeans::KMeans;

pub fn create_auto_collections(
    embeddings: Vec<(i64, Vec<f32>)>,
    num_clusters: usize
) -> Vec<Collection> {
    // 1. Prepare data matrix
    let data: Vec<f32> = embeddings.iter()
        .flat_map(|(_, vec)| vec.iter().copied())
        .collect();

    // 2. Run K-means
    let kmeans = KMeans::fit(&data, num_clusters, Default::default())?;

    // 3. Group files by cluster
    let labels = kmeans.predict(&data)?;

    // 4. Create collections
    // ...
}
```

**UI Components:**
- Collections sidebar panel (left side)
- Collection detail view
- "Generate Collections" button with cluster count slider
- Drag-and-drop interface

**Tauri Commands:**
- `generate_collections(num_clusters: usize)`
- `create_collection(name: String, color: String)`
- `add_to_collection(collection_id: i64, file_ids: Vec<i64>)`
- `get_collections()` → `Vec<Collection>`

**Success Criteria:**
- ✅ Clustering completes in <3 seconds for 1K files
- ✅ Collections have meaningful groupings (≥70% semantic coherence)
- ✅ UI is intuitive and responsive

---

### AI-204: Auto-Tagging System ⏳
**Priority:** P1 - Content understanding
**Estimated Time:** 4-5 hours

**Goal:** Automatically generate tags for files based on content

**Approaches:**
1. **Keyword Extraction** (Simple):
   - TF-IDF for top keywords
   - Extract from headings, titles

2. **Topic Modeling** (Advanced):
   - LDA (Latent Dirichlet Allocation)
   - Extract topics as tags

3. **Embedding-Based** (Recommended):
   - Pre-defined tag embeddings ("code", "documentation", "meeting", etc.)
   - Match file embeddings to tag embeddings
   - Dynamic tag discovery via clustering

**Database Schema:**
```sql
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    color TEXT,
    auto_generated BOOLEAN DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE TABLE file_tags (
    file_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    confidence REAL,  -- 0.0-1.0 for auto-generated tags
    added_at INTEGER NOT NULL,
    PRIMARY KEY (file_id, tag_id),
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX idx_file_tags_file ON file_tags(file_id);
CREATE INDEX idx_file_tags_tag ON file_tags(tag_id);
```

**Implementation:**
```rust
// Predefined tags with embeddings
const PREDEFINED_TAGS: &[(&str, &str)] = &[
    ("code", "programming source code implementation"),
    ("documentation", "readme guide tutorial documentation"),
    ("meeting", "meeting notes discussion minutes"),
    ("research", "research paper article study"),
    ("configuration", "config settings yaml json"),
    // ... 20-30 more
];

pub async fn auto_tag_file(file_id: i64, content: &str) -> Vec<Tag> {
    let file_embedding = embedding_service.embed(content).await?;

    let mut tag_scores: Vec<(String, f32)> = PREDEFINED_TAGS
        .iter()
        .map(|(tag, desc)| {
            let tag_embedding = embedding_service.embed(desc).await?;
            let score = cosine_similarity(&file_embedding, &tag_embedding);
            (tag.to_string(), score)
        })
        .filter(|(_, score)| *score >= 0.65)  // Confidence threshold
        .collect();

    tag_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    tag_scores.truncate(5);  // Max 5 tags per file

    tag_scores.into_iter().map(|(name, conf)| Tag { name, confidence: conf }).collect()
}
```

**UI Features:**
- Tag cloud visualization
- Filter by tags
- Tag suggestions on file preview
- Bulk tagging interface

**Tauri Commands:**
- `auto_tag_files(file_ids: Vec<i64>)`
- `get_tags()` → `Vec<Tag>`
- `search_by_tag(tag_ids: Vec<i64>)`

**Success Criteria:**
- ✅ Tagging: >20 files/second
- ✅ Tag accuracy: ≥75% (manual validation)
- ✅ Max 5 tags per file

---

### AI-205: Saved Searches ⏳
**Priority:** P2 - User convenience
**Estimated Time:** 2-3 hours

**Goal:** Allow users to save and quickly re-run complex searches

**Features:**
- Save current search query + filters
- Quick access dropdown
- Edit/delete saved searches
- Search history tracking

**Database Schema:**
```sql
CREATE TABLE saved_searches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    query TEXT NOT NULL,
    filters TEXT,  -- JSON: {"file_type": "md", "size_min": 1000, ...}
    is_semantic BOOLEAN DEFAULT 0,
    created_at INTEGER NOT NULL,
    last_used_at INTEGER
);

CREATE TABLE search_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query TEXT NOT NULL,
    filters TEXT,
    result_count INTEGER,
    query_time_ms INTEGER,
    timestamp INTEGER NOT NULL
);

CREATE INDEX idx_search_history_timestamp ON search_history(timestamp DESC);
```

**UI Components:**
- "Save Search" button in search bar
- Saved searches dropdown (⭐ icon)
- Search history panel (recent 50 searches)
- Edit saved search modal

**Tauri Commands:**
- `save_search(name: String, query: String, filters: SearchFilters)`
- `get_saved_searches()` → `Vec<SavedSearch>`
- `delete_saved_search(id: i64)`
- `get_search_history(limit: usize)` → `Vec<SearchHistory>`

**Success Criteria:**
- ✅ Saves persist across app restarts
- ✅ One-click search execution
- ✅ History tracks last 100 searches

---

### AI-206: File Relationships & Graph ⏳
**Priority:** P2 - Insights
**Estimated Time:** 4-5 hours

**Goal:** Discover and visualize relationships between files

**Features:**
- **Relationship Types:**
  - Similar content (embedding similarity)
  - Shared tags
  - Same collection
  - Temporal proximity (edited around same time)

- **Graph Visualization:**
  - Interactive network graph (D3.js or Cytoscape.js)
  - Node: File (sized by importance)
  - Edge: Relationship (thickness = strength)
  - Clusters by collection/tag

**Database Schema:**
```sql
CREATE TABLE file_relationships (
    file_a_id INTEGER NOT NULL,
    file_b_id INTEGER NOT NULL,
    relationship_type TEXT NOT NULL,  -- 'similar', 'tag', 'collection', 'temporal'
    strength REAL NOT NULL,  -- 0.0-1.0
    created_at INTEGER NOT NULL,
    PRIMARY KEY (file_a_id, file_b_id, relationship_type),
    FOREIGN KEY (file_a_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY (file_b_id) REFERENCES files(id) ON DELETE CASCADE
);

CREATE INDEX idx_relationships_file_a ON file_relationships(file_a_id);
CREATE INDEX idx_relationships_file_b ON file_relationships(file_b_id);
```

**UI Components:**
- Graph view mode toggle
- Interactive graph canvas
- Related files panel on preview
- "Explore connections" button

**Tauri Commands:**
- `get_related_files(file_id: i64, relationship_types: Vec<String>)`
- `build_file_graph(file_ids: Vec<i64>)` → Graph data structure

**Success Criteria:**
- ✅ Graph renders 100+ nodes smoothly
- ✅ Shows meaningful relationships
- ✅ Interactive exploration (click to navigate)

---

### AI-207: Analytics Dashboard ⏳
**Priority:** P2 - Insights
**Estimated Time:** 3-4 hours

**Goal:** Provide insights into file usage and search patterns

**Metrics to Track:**
- Most searched queries
- Most viewed files
- Search success rate (clicks after search)
- Indexing trends over time
- Tag distribution
- Collection sizes
- Search performance trends

**Database Schema:**
```sql
CREATE TABLE file_views (
    file_id INTEGER NOT NULL,
    viewed_at INTEGER NOT NULL,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

CREATE INDEX idx_file_views_timestamp ON file_views(viewed_at DESC);
CREATE INDEX idx_file_views_file ON file_views(file_id);

-- search_history already exists from AI-205
```

**UI Components:**
- Analytics page/modal
- Charts: Line (trends), Bar (top items), Pie (distributions)
- Date range selector
- Export to CSV

**Charts:**
1. **Search Activity:** Queries per day (line chart)
2. **Top Searches:** Most frequent queries (bar chart)
3. **Top Files:** Most viewed files (bar chart)
4. **Tag Distribution:** Files per tag (pie chart)
5. **Collection Sizes:** Files per collection (bar chart)
6. **Performance:** Search latency trends (line chart)

**Tauri Commands:**
- `record_file_view(file_id: i64)`
- `get_analytics(date_from: i64, date_to: i64)` → Analytics data

**Success Criteria:**
- ✅ Charts render smoothly
- ✅ Insights are actionable
- ✅ Minimal performance impact (<1% overhead)

---

### AI-208: Advanced Indexing Options ⏳
**Priority:** P3 - Enhancement
**Estimated Time:** 2-3 hours

**Goal:** Give users control over what gets indexed and how

**Features:**
- **Indexing Profiles:**
  - Fast: Text only, no embeddings
  - Balanced: Text + embeddings (default)
  - Deep: Text + embeddings + OCR (future)

- **Selective Embedding:**
  - Only embed files matching criteria
  - "Embed this collection" action
  - Re-embed with new model

- **Incremental Updates:**
  - Watch for file changes
  - Auto-reindex modified files
  - Smart re-embedding (only if content changed significantly)

**UI Components:**
- Indexing profile selector in settings
- "Embed Selected" button
- Re-embedding progress indicator

**Tauri Commands:**
- `set_indexing_profile(profile: IndexingProfile)`
- `embed_files(file_ids: Vec<i64>, force: bool)`

**Success Criteria:**
- ✅ Profiles work as expected
- ✅ Selective embedding saves time
- ✅ Incremental updates are fast (<1s per file)

---

## 🎯 Success Criteria (Phase 2)

### Technical Goals
- [ ] Semantic search finds relevant files even without keyword matches
- [ ] Embedding generation: >10 files/second
- [ ] Semantic search latency: <500ms for 10K files
- [ ] Auto-tagging accuracy: ≥75%
- [ ] Smart collections have ≥70% semantic coherence
- [ ] 100% offline (no API calls)
- [ ] Memory usage increase: <100MB (embeddings stored efficiently)

### User Experience Goals
- [ ] Users can find files by meaning, not just keywords
- [ ] Collections automatically organize files intelligently
- [ ] Tags provide useful categorization
- [ ] Saved searches speed up repetitive tasks
- [ ] Analytics provide actionable insights
- [ ] All AI features feel fast and responsive

---

## 📊 Technology Stack

### Embedding & ML
- **ONNX Runtime** (`ort` crate) - Run ML models in Rust
- **Model:** `all-MiniLM-L6-v2` (22MB, 384-dim vectors)
- **Tokenizers** - Text preprocessing
- **ndarray** - Vector math

### Clustering & ML
- **smartcore** - K-means, DBSCAN clustering
- **linfa** - Alternative ML toolkit

### Vector Storage
- **SQLite BLOB** - Store embeddings (1.5KB each)
- **In-memory cache** - Hot embeddings for speed

### Frontend (Charts & Viz)
- **Chart.js** or **Apache ECharts** - Analytics charts
- **Cytoscape.js** or **D3.js** - Graph visualization

---

## 📁 New File Structure

```
src-tauri/src/
├── ai/
│   ├── mod.rs
│   ├── embeddings.rs       # Embedding service (ONNX)
│   ├── similarity.rs       # Cosine similarity, search
│   ├── clustering.rs       # K-means, collections
│   ├── tagging.rs          # Auto-tagging logic
│   └── models/
│       └── all-MiniLM-L6-v2/  # Model files (downloaded)
├── commands/
│   ├── ai_commands.rs      # AI-specific commands
│   └── analytics.rs        # Analytics commands
└── db/
    ├── ai_schema.sql       # Embeddings, collections, tags tables
    └── ai_operations.rs    # AI DB operations

src/lib/
├── components/
│   ├── SemanticSearch.svelte
│   ├── Collections.svelte
│   ├── TagCloud.svelte
│   ├── FileGraph.svelte
│   └── Analytics.svelte
├── stores/
│   ├── collectionStore.ts
│   └── analyticsStore.ts
└── types/
    └── ai.ts               # AI-related types
```

---

## 🗓️ Implementation Timeline

### Week 1: Foundation (AI-201, AI-202)
- Day 1-2: Embedding model integration
- Day 3-4: Semantic search implementation
- Day 5: Testing and refinement

### Week 2: Organization (AI-203, AI-204)
- Day 1-3: Smart collections with clustering
- Day 4-5: Auto-tagging system

### Week 3: Polish (AI-205, AI-206, AI-207, AI-208)
- Day 1: Saved searches
- Day 2-3: File relationships & graph
- Day 4: Analytics dashboard
- Day 5: Advanced indexing options

**Total Estimated Time:** 30-40 hours (2-3 weeks)

---

## 🎨 Design Considerations

### Neural Gold AI Theme
- **AI Active Indicator:** Pulsing Neural Gold badge
- **Similarity Scores:** Gradient from Neural Gold to Ember Gold
- **Collections:** Color-coded with Neural Gold as primary
- **Graph Nodes:** Neural Gold highlights for active file

### UI/UX Principles
- **Progressive Disclosure:** Hide complexity, reveal on demand
- **Non-Intrusive:** AI features enhance, don't replace keyword search
- **Transparent:** Show confidence scores, explain why files are related
- **Controllable:** Users can disable AI features

---

## 🔧 Dependencies to Add

**Rust (Cargo.toml):**
```toml
[dependencies]
# ML & Embeddings
ort = "2.0"                    # ONNX Runtime
ndarray = "0.16"               # N-dimensional arrays
tokenizers = "0.20"            # Hugging Face tokenizers

# Clustering
smartcore = "0.3"              # K-means, DBSCAN

# Serialization
serde_json = "1.0"             # JSON for filters

# Utilities
reqwest = { version = "0.12", features = ["blocking"] }  # Download models
```

**Frontend (package.json):**
```json
{
  "dependencies": {
    "chart.js": "^4.4.0",
    "cytoscape": "^3.28.0",
    "@sveltejs/kit": "^2.0.0"
  }
}
```

---

## 🐛 Potential Challenges

### 1. Model Size & Loading
- **Challenge:** 22MB model increases bundle size
- **Solution:** Download on first run, not bundled
- **Fallback:** Graceful degradation if model unavailable

### 2. Embedding Performance
- **Challenge:** Generating embeddings for 10K files takes time
- **Solution:** Background processing, batch operations, progress bar

### 3. Memory Usage
- **Challenge:** 10K files × 1.5KB = 15MB in RAM
- **Solution:** Load embeddings on-demand, cache hot ones

### 4. Search Latency
- **Challenge:** Comparing 10K vectors is slow (O(n))
- **Solution:** Use approximate nearest neighbors (ANN) if needed
- **Future:** FAISS or HNSW for large datasets

---

## 💡 Future Enhancements (Phase 3+)

- **Larger Models:** Allow power users to use bigger models
- **OCR Support:** Extract text from images/scanned PDFs
- **LLM Integration:** GPT-4 for summarization (optional, privacy-aware)
- **Cross-File Context:** "What files discuss authentication AND security?"
- **Temporal Analysis:** "Show me files I worked on last week"
- **Export Collections:** Share collections with others

---

## 📝 Next Steps

1. ✅ Create this planning document
2. Start with **AI-201: Embedding Model Integration**
3. Set up ONNX Runtime and download model
4. Implement basic embedding generation
5. Test performance and accuracy
6. Move to AI-202 (Semantic Search)

---

**Phase 2 Goal:** Transform Cortex from a fast search tool into an intelligent knowledge assistant. 🧠

Let's build something amazing! 🚀
