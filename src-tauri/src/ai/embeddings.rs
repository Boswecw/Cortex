//! Embedding Generation Service
//!
//! Provides semantic embeddings for file content using ONNX Runtime.
//! Uses the `all-MiniLM-L6-v2` model for fast, offline embeddings.

use anyhow::{Context, Result};
use ndarray::{Array1, Array2, Array3, Axis};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokenizers::Tokenizer;

/// Configuration for the embedding service
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    /// Path to the ONNX model file
    pub model_path: PathBuf,
    /// Path to the tokenizer JSON file
    pub tokenizer_path: PathBuf,
    /// Maximum sequence length (tokens)
    pub max_length: usize,
    /// Batch size for processing multiple texts
    pub batch_size: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        let model_dir = dirs::home_dir()
            .expect("Failed to get home directory")
            .join(".cortex")
            .join("models")
            .join("all-MiniLM-L6-v2");

        Self {
            model_path: model_dir.join("model.onnx"),
            tokenizer_path: model_dir.join("tokenizer.json"),
            max_length: 128,
            batch_size: 32,
        }
    }
}

/// Embedding service using ONNX Runtime
pub struct EmbeddingService {
    config: EmbeddingConfig,
    session: Session,
    tokenizer: Arc<Tokenizer>,
}

impl EmbeddingService {
    /// Create a new embedding service
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for model and tokenizer paths
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = EmbeddingConfig::default();
    /// let service = EmbeddingService::new(config)?;
    /// ```
    pub fn new(config: EmbeddingConfig) -> Result<Self> {
        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&config.tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {:?}", e))?;

        // Create ONNX session
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(&config.model_path)
            .context("Failed to load ONNX model")?;

        Ok(Self {
            config,
            session,
            tokenizer: Arc::new(tokenizer),
        })
    }

    /// Generate embedding for a single text
    ///
    /// # Arguments
    ///
    /// * `text` - Input text to embed
    ///
    /// # Returns
    ///
    /// A 384-dimensional embedding vector
    ///
    /// # Example
    ///
    /// ```ignore
    /// let embedding = service.embed("Hello world")?;
    /// assert_eq!(embedding.len(), 384);
    /// ```
    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_batch(&[text])?;
        Ok(embeddings.into_iter().next().unwrap())
    }

    /// Generate embeddings for multiple texts (batch processing)
    ///
    /// # Arguments
    ///
    /// * `texts` - Slice of texts to embed
    ///
    /// # Returns
    ///
    /// Vector of 384-dimensional embeddings, one per input text
    ///
    /// # Example
    ///
    /// ```ignore
    /// let texts = vec!["First text", "Second text"];
    /// let embeddings = service.embed_batch(&texts)?;
    /// assert_eq!(embeddings.len(), 2);
    /// ```
    pub fn embed_batch(&mut self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Tokenize all texts
        let mut encodings = Vec::with_capacity(texts.len());
        for text in texts {
            let encoding = self
                .tokenizer
                .encode(*text, true)
                .map_err(|e| anyhow::anyhow!("Failed to tokenize: {:?}", e))?;
            encodings.push(encoding);
        }

        // Prepare input tensors
        let batch_size = texts.len();
        let seq_len = self.config.max_length;

        let mut input_ids = Vec::with_capacity(batch_size * seq_len);
        let mut attention_mask = Vec::with_capacity(batch_size * seq_len);

        for encoding in &encodings {
            let ids = encoding.get_ids();
            let mask = encoding.get_attention_mask();

            // Pad or truncate to max_length
            for i in 0..seq_len {
                input_ids.push(ids.get(i).copied().unwrap_or(0) as i64);
                attention_mask.push(mask.get(i).copied().unwrap_or(0) as i64);
            }
        }

        // Create input arrays
        let input_ids_array = Array2::from_shape_vec((batch_size, seq_len), input_ids)?;
        let attention_mask_array = Array2::from_shape_vec((batch_size, seq_len), attention_mask)?;

        // Create ONNX Values
        let input_ids_value = Value::from_array(input_ids_array)?;
        let attention_mask_value = Value::from_array(attention_mask_array)?;

        // Run inference
        let outputs = self.session.run(ort::inputs![
            "input_ids" => input_ids_value,
            "attention_mask" => attention_mask_value,
        ])?;

        // Extract embeddings from output
        // The model outputs a tensor of shape (batch_size, seq_len, hidden_size)
        // We take the mean across the sequence dimension
        let (shape, data) = outputs["last_hidden_state"]
            .try_extract_tensor::<f32>()?;

        // Reshape flat data into 3D array (batch_size, seq_len, hidden_size)
        let hidden_size = 384; // all-MiniLM-L6-v2 has 384-dim embeddings
        let output_tensor = Array3::from_shape_vec(
            (batch_size, seq_len, hidden_size),
            data.to_vec()
        )?;

        // Mean pooling across sequence dimension
        let mut embeddings = Vec::with_capacity(batch_size);
        for batch_idx in 0..batch_size {
            let batch_output = output_tensor.index_axis(Axis(0), batch_idx);

            // Calculate mean across sequence dimension
            let embedding: Vec<f32> = batch_output
                .mean_axis(Axis(0))
                .expect("Failed to calculate mean")
                .to_vec();

            // Normalize the embedding (L2 normalization)
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            let normalized: Vec<f32> = embedding.iter().map(|x| x / norm).collect();

            embeddings.push(normalized);
        }

        Ok(embeddings)
    }

    /// Get the embedding dimension (should be 384 for all-MiniLM-L6-v2)
    pub fn dimension(&self) -> usize {
        384
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires model files to be downloaded
    fn test_embed_single_text() {
        let config = EmbeddingConfig::default();
        let service = EmbeddingService::new(config).unwrap();

        let embedding = service.embed("Hello, world!").unwrap();

        assert_eq!(embedding.len(), 384);

        // Check that embedding is normalized (L2 norm ≈ 1.0)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }

    #[test]
    #[ignore] // Requires model files to be downloaded
    fn test_embed_batch() {
        let config = EmbeddingConfig::default();
        let service = EmbeddingService::new(config).unwrap();

        let texts = vec!["First text", "Second text", "Third text"];
        let embeddings = service.embed_batch(&texts).unwrap();

        assert_eq!(embeddings.len(), 3);
        for emb in &embeddings {
            assert_eq!(emb.len(), 384);
        }
    }

    #[test]
    #[ignore] // Requires model files to be downloaded
    fn test_similar_texts_have_similar_embeddings() {
        let config = EmbeddingConfig::default();
        let service = EmbeddingService::new(config).unwrap();

        let text1 = "The quick brown fox jumps over the lazy dog";
        let text2 = "A fast brown fox leaps over a sleepy dog";
        let text3 = "Machine learning is a subset of artificial intelligence";

        let embeddings = service.embed_batch(&[text1, text2, text3]).unwrap();

        // Calculate cosine similarities
        let sim_1_2 = cosine_similarity(&embeddings[0], &embeddings[1]);
        let sim_1_3 = cosine_similarity(&embeddings[0], &embeddings[2]);

        // Similar texts should have higher similarity
        assert!(sim_1_2 > sim_1_3);
        assert!(sim_1_2 > 0.7); // Similar texts should have high similarity
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        use crate::ai::similarity::cosine_similarity as cos_sim;
        cos_sim(a, b)
    }
}
