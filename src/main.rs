//! Interactive CLI for querying the local Rustdoc RAG index.
//!
//! Loads cached document chunks from the Official Rust Book, generates 
//! local 384-dimensional embeddings, and executes vector similarity search in an 
//! interactive terminal loop.

mod corpus;
mod embeddings;
mod search;

use anyhow::Result;
use embeddings::Embedder;
use search::VectorIndex;
use std::io::{self, Write};
use std::path::Path;
use std::time::Instant;

fn build_index() -> Result<VectorIndex> {
    let cache_path = Path::new("docs_cache/corpus.json");
    let chunks = corpus::load_corpus(cache_path)?;

    println!("Embedding {} chunks locally...", chunks.len());
    let start = Instant::now();

    let texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
    let mut embedder = Embedder::new()?;
    let vectors = embedder.embed_batch(&texts)?;

    let elapsed = start.elapsed();
    println!(
        "Indexed {} chunks in {:.2?} ({:.2} chunks/sec)",
        chunks.len(),
        elapsed,
        chunks.len() as f64 / elapsed.as_secs_f64()
    );

    Ok(VectorIndex::new(chunks, vectors))
}

fn main() -> Result<()> {
    let index = build_index()?;
    println!("Index ready with {} chunks.", index.len());
    let mut embedder = Embedder::new()?;

    println!("\nType a query or 'exit' to quit.\n");
    loop {
        print!("query> ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }
        if line.eq_ignore_ascii_case("exit") || line.eq_ignore_ascii_case("quit") {
            break;
        }

        let start = Instant::now();
        let query_vector = embedder.embed_query(line)?;
        let hits = index.search(&query_vector, 3);
        let elapsed = start.elapsed();

        for hit in &hits {
            println!(
                "\n[{:.3}] {} ({})\n  {}\n  {}",
                hit.score,
                hit.chunk.heading,
                hit.chunk.chapter_file,
                hit.chunk.text.lines().next().unwrap_or(""),
                hit.chunk.url
            );
        }
        println!("\n(query took {:.2?})\n", elapsed);
    }
    Ok(())
}