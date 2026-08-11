//! Local vector generation pipeline using `fastembed` (all-MiniLM-L6-v2 ONNX runtime).
//!
//! Generates 384-dimensional dense vector representations completely on-device
//! without making external API calls. Downloads model weights (~90MB) on first initialization.

use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

pub struct Embedder {
    model: TextEmbedding,
}

impl Embedder {
    /// Initializes the ONNX embedding engine, downloading weights on first run if missing.
    pub fn new() -> Result<Self> {
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true),
        )
        .context("Failed to initialize ONNX embedding runtime")?;
        Ok(Self { model })
    }

    /// Batches text chunk embedding generation for improved execution throughput.
    pub fn embed_batch(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let embeddings = self
            .model
            .embed(texts.to_vec(), None)
            .context("Batch inference failed")?;
        Ok(embeddings)
    }

    /// Embeds a single query string for runtime similarity matching.
    pub fn embed_query(&mut self, text: &str) -> Result<Vec<f32>> {
        let mut result = self.embed_batch(&[text.to_string()])?;
        result
            .pop()
            .context("Inference engine returned empty vector output")
    }
}

/// Evaluates normalized cosine similarity between two equal-length f32 vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Vectors must share identical dimensions");
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}