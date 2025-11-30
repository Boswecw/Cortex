//! AI Features Module
//!
//! This module provides intelligent features for Cortex:
//! - Semantic embeddings for content understanding
//! - Similarity search beyond keyword matching
//! - Smart collections through clustering
//! - Auto-tagging based on content analysis
//!
//! ## Architecture
//!
//! - `embeddings.rs` - ONNX-based embedding generation
//! - `similarity.rs` - Cosine similarity and semantic search
//! - `clustering.rs` - K-means clustering for collections
//! - `tagging.rs` - Auto-tagging logic
//!
//! ## Privacy & Performance
//!
//! All AI operations run 100% offline using local ONNX models:
//! - Model: `sentence-transformers/all-MiniLM-L6-v2` (22MB, 384-dim)
//! - No network calls, all data stays local
//! - Target: >10 files/second embedding generation
//! - Target: <500ms semantic search for 10K files

pub mod embeddings;
pub mod model_downloader;
pub mod similarity;

// Re-exports
pub use embeddings::{EmbeddingService, EmbeddingConfig};
pub use model_downloader::{download_model, ensure_model_downloaded, is_model_downloaded};
pub use similarity::{cosine_similarity, find_top_k, semantic_search};
