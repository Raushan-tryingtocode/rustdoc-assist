//! Corpus generation module for retrieving and parsing raw markdown source files from GitHub.
//!
//! Extracts documentation sections, strips non-prose elements (code blocks, blockquotes), 
//! and caches structured chunks locally as JSON.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocChunk {
    pub chapter_file: String,
    pub heading: String,
    pub text: String,
    pub url: String,
}

const CHAPTER_FILES: &[&str] = &[
    "ch01-01-installation",
    "ch01-03-hello-cargo",
    "ch03-01-variables-and-mutability",
    "ch04-01-what-is-ownership",
    "ch04-02-references-and-borrowing",
    "ch05-01-defining-structs",
    "ch05-03-method-syntax",
    "ch09-02-recoverable-errors-with-result",
    "ch10-02-traits",
    "ch10-03-lifetime-syntax",
    "ch16-01-threads",
    "ch18-01-what-is-oo",
];

const RAW_BASE: &str = "https://raw.githubusercontent.com/rust-lang/book/main/src";
const BOOK_BASE: &str = "https://doc.rust-lang.org/book";

fn fetch_chapter(file: &str) -> Result<String> {
    let url = format!("{RAW_BASE}/{file}.md");
    let body = ureq::get(&url)
        .call()
        .with_context(|| format!("Failed to fetch chapter from {url}"))?
        .body_mut()
        .read_to_string()
        .with_context(|| format!("Failed to read response stream for {url}"))?;
    Ok(body)
}

/// Parses raw Markdown content and extracts chunks split by h2 (`##`) headers.
/// Ignores code fences and quote blocks to ensure high semantic signal during embedding.
fn chunk_markdown(file: &str, markdown: &str) -> Vec<DocChunk> {
    let mut chunks = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut current_text = String::new();
    let mut in_code_block = false;

    let flush = |heading: &Option<String>, text: &str, chunks: &mut Vec<DocChunk>| {
        if let Some(h) = heading {
            let cleaned = text.trim();
            if !cleaned.is_empty() {
                let anchor = h
                    .to_lowercase()
                    .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
                    .replace(' ', "-");
                chunks.push(DocChunk {
                    chapter_file: file.to_string(),
                    heading: h.clone(),
                    text: cleaned.to_string(),
                    url: format!("{BOOK_BASE}/{file}.html#{anchor}"),
                });
            }
        }
    };

    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        if trimmed.starts_with("> ") || trimmed == ">" {
            continue;
        }
        if let Some(heading) = trimmed.strip_prefix("## ") {
            flush(&current_heading, &current_text, &mut chunks);
            current_heading = Some(heading.trim().to_string());
            current_text.clear();
        } else if trimmed.starts_with('#') {
            continue;
        } else {
            current_text.push_str(line);
            current_text.push('\n');
        }
    }
    flush(&current_heading, &current_text, &mut chunks);
    chunks
}

/// Loads corpus from disk if available; otherwise fetches upstream markdown and updates cache.
pub fn load_corpus(cache_path: &Path) -> Result<Vec<DocChunk>> {
    if cache_path.exists() {
        let raw = std::fs::read_to_string(cache_path)
            .with_context(|| format!("Failed to read corpus cache at {}", cache_path.display()))?;
        let chunks: Vec<DocChunk> =
            serde_json::from_str(&raw).context("Failed to deserialize cached corpus JSON")?;
        println!("Loaded {} cached chunks from {}", chunks.len(), cache_path.display());
        return Ok(chunks);
    }

    println!("No cache found — fetching {} chapters from rust-lang/book...", CHAPTER_FILES.len());
    let mut all_chunks = Vec::new();
    for file in CHAPTER_FILES {
        let markdown = fetch_chapter(file)?;
        let chunks = chunk_markdown(file, &markdown);
        println!("  {file}: {} chunks", chunks.len());
        all_chunks.extend(chunks);
    }

    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let serialized = serde_json::to_string_pretty(&all_chunks)?;
    std::fs::write(cache_path, serialized)
        .with_context(|| format!("Failed to write corpus cache to {}", cache_path.display()))?;
    println!("Fetched and cached {} total chunks to {}", all_chunks.len(), cache_path.display());

    Ok(all_chunks)
}