//! Simple exact vector search over embedded document chunks.
//!
//! For small to medium corpus sizes (hundreds of items), exact O(n) dot-product 
//! cosine similarity search provides exact nearest neighbors with negligible latency. 
//! If the corpus scales to $10^5+$ chunks, this implementation can be replaced with 
//! graph-based or quantized ANN indexing 

use crate::corpus::DocChunk;
use crate::embeddings::cosine_similarity;

pub struct VectorIndex {
    chunks: Vec<DocChunk>,
    vectors: Vec<Vec<f32>>,
}

pub struct SearchHit<'a> {
    pub chunk: &'a DocChunk,
    pub score: f32, // Cosine similarity in range [-1.0, 1.0]
}

impl VectorIndex {
    pub fn new(chunks: Vec<DocChunk>, vectors: Vec<Vec<f32>>) -> Self {
        assert_eq!(
            chunks.len(),
            vectors.len(),
            "Chunk count must match vector count"
        );
        Self { chunks, vectors }
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// Ranks all indexed chunks by cosine similarity to `query_vector` and returns the top `k` hits.
    pub fn search(&self, query_vector: &[f32], k: usize) -> Vec<SearchHit<'_>> {
        let mut scored: Vec<SearchHit> = self
            .chunks
            .iter()
            .zip(self.vectors.iter())
            .map(|(chunk, vec)| SearchHit {
                chunk,
                score: cosine_similarity(query_vector, vec),
            })
            .collect();

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        scored.truncate(k);
        scored
    }
}