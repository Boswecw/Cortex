//! Similarity Search and Vector Operations
//!
//! Provides functions for semantic similarity search using embeddings.

use anyhow::Result;

/// Calculate cosine similarity between two vectors
///
/// Cosine similarity ranges from -1 to 1:
/// - 1.0: Identical direction (most similar)
/// - 0.0: Orthogonal (unrelated)
/// - -1.0: Opposite direction (dissimilar)
///
/// # Arguments
///
/// * `a` - First vector
/// * `b` - Second vector
///
/// # Returns
///
/// Cosine similarity score between -1.0 and 1.0
///
/// # Example
///
/// ```
/// use cortex_lib::ai::cosine_similarity;
///
/// let vec1 = vec![1.0, 0.0, 0.0];
/// let vec2 = vec![1.0, 0.0, 0.0];
/// let similarity = cosine_similarity(&vec1, &vec2);
/// assert!((similarity - 1.0).abs() < 0.001);
/// ```
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");

    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|y| y * y).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

/// Find top-k most similar vectors to a query vector
///
/// # Arguments
///
/// * `query` - Query vector to find similarities for
/// * `candidates` - Vector of (id, embedding) pairs to search through
/// * `top_k` - Number of top results to return
/// * `threshold` - Minimum similarity score (0.0-1.0)
///
/// # Returns
///
/// Vector of (id, score) pairs sorted by score descending
///
/// # Example
///
/// ```ignore
/// let query = vec![1.0, 0.0, 0.0];
/// let candidates = vec![
///     (1, vec![0.9, 0.1, 0.0]),
///     (2, vec![0.0, 1.0, 0.0]),
///     (3, vec![1.0, 0.0, 0.0]),
/// ];
/// let results = find_top_k(&query, &candidates, 2, 0.5);
/// assert_eq!(results.len(), 2);
/// assert_eq!(results[0].0, 3); // Most similar
/// ```
pub fn find_top_k<T: Clone>(
    query: &[f32],
    candidates: &[(T, Vec<f32>)],
    top_k: usize,
    threshold: f32,
) -> Vec<(T, f32)> {
    let mut scored: Vec<(T, f32)> = candidates
        .iter()
        .map(|(id, vec)| (id.clone(), cosine_similarity(query, vec)))
        .filter(|(_, score)| *score >= threshold)
        .collect();

    // Sort by score descending
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take top K
    scored.truncate(top_k);

    scored
}

/// Semantic search function (placeholder - to be integrated with database)
///
/// This will be implemented once we have the database schema for embeddings.
pub async fn semantic_search(
    query: &str,
    _limit: usize,
    _threshold: f32,
) -> Result<Vec<(i64, f32)>> {
    // TODO: Implement full semantic search with database integration
    // 1. Generate query embedding
    // 2. Load file embeddings from database
    // 3. Calculate similarities
    // 4. Return top results
    log::warn!("semantic_search not yet implemented with database");
    Err(anyhow::anyhow!(
        "Semantic search requires database integration (coming in next step)"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&vec1, &vec2);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&vec1, &vec2);
        assert!(sim.abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![-1.0, 0.0, 0.0];
        let sim = cosine_similarity(&vec1, &vec2);
        assert!((sim + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_find_top_k() {
        let query = vec![1.0, 0.0, 0.0];
        let candidates = vec![
            (1, vec![0.9, 0.1, 0.0]),
            (2, vec![0.0, 1.0, 0.0]),
            (3, vec![1.0, 0.0, 0.0]),
            (4, vec![0.5, 0.5, 0.0]),
        ];

        let results = find_top_k(&query, &candidates, 2, 0.5);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 3); // Identical vector
        assert!(results[0].1 > 0.99); // Near 1.0
    }

    #[test]
    fn test_find_top_k_with_threshold() {
        let query = vec![1.0, 0.0, 0.0];
        let candidates = vec![
            (1, vec![0.9, 0.1, 0.0]),
            (2, vec![0.0, 1.0, 0.0]), // Orthogonal, will be filtered
            (3, vec![1.0, 0.0, 0.0]),
        ];

        let results = find_top_k(&query, &candidates, 10, 0.7);

        // Only vectors with similarity >= 0.7 should be returned
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|(_, score)| *score >= 0.7));
    }
}
