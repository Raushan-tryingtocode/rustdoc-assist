//! # Local Rustdoc RAG CLI Assistant
//! 
//! An offline RAG utility with fuzzy keyword matching and official doc link outputs.
//! Retrieves local documentation chunks and synthesizes explanations without external APIs.

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::io::{self, Write};
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

// --- Data Structures ---

#[derive(Debug, Clone)]
pub struct DocChunk {
    pub chapter: String,
    pub section: String,
    pub keywords: Vec<&'static str>,
    pub text: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub raw_text: String,
    pub explanation: String,
    pub chapter: String,
    pub section: String,
    pub url: String,
    pub score: u32,
}

// --- Local Knowledge Store ---

pub struct DocStore {
    chunks: Arc<RwLock<Vec<DocChunk>>>,
}

impl DocStore {
    pub fn new(seed_data: Vec<DocChunk>) -> Self {
        Self {
            chunks: Arc::new(RwLock::new(seed_data)),
        }
    }

    pub async fn search(&self, query: &str) -> Option<(DocChunk, u32)> {
        let query_tokens = tokenize(query);
        let raw_numbers = extract_section_numbers(query);

        if query_tokens.is_empty() && raw_numbers.is_empty() {
            return None;
        }

        self.chunks
            .read()
            .await
            .iter()
            .map(|chunk| (chunk.clone(), score_chunk(chunk, &query_tokens, &raw_numbers)))
            .filter(|(_, score)| *score > 0)
            .max_by_key(|(_, score)| *score)
    }
}

// --- Helper Search & Fuzzy Matching Logic ---

fn stopwords() -> &'static HashSet<&'static str> {
    static WORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    WORDS.get_or_init(|| {
        [
            "a", "an", "the", "is", "are", "was", "were", "be", "do", "does", "did", 
            "how", "what", "which", "who", "why", "when", "where", "of", "in", "on", 
            "at", "to", "for", "with", "about", "from", "and", "or", "but", "if", 
            "so", "that", "this", "can", "could", "will", "would", "should", "give", 
            "me", "link", "url", "please", "tell",
        ]
        .into_iter()
        .collect()
    })
}

fn tokenize(input: &str) -> Vec<String> {
    input
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| word.len() > 1 && !stopwords().contains(word))
        .map(String::from)
        .collect()
}

fn extract_section_numbers(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .filter(|s| s.chars().any(|c| c.is_ascii_digit()))
        .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric() && c != '.').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn singular(word: &str) -> &str {
    word.strip_suffix('s').unwrap_or(word)
}

// Simple Levenshtein distance for fuzzy matching (typo handling)
fn edit_distance(a: &str, b: &str) -> usize {
    if a == b { return 0; }
    if a.is_empty() { return b.len(); }
    if b.is_empty() { return a.len(); }

    let mut cache: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut result = i + 1;
        let mut dist_b = i;
        for (j, cb) in b.chars().enumerate() {
            let dist_a = result;
            result = if ca == cb { dist_b } else { 1 + dist_a.min(result).min(dist_b) };
            dist_b = dist_a;
            cache[j + 1] = result;
        }
    }
    cache[b.len()]
}

fn tokens_match(a: &str, b: &str) -> bool {
    let sa = singular(a);
    let sb = singular(b);
    if sa == sb {
        return true;
    }
    // Allow max 2 character differences if words are longer than 4 chars
    if sa.len() > 4 && sb.len() > 4 {
        return edit_distance(sa, sb) <= 2;
    }
    false
}

fn score_chunk(chunk: &DocChunk, query_tokens: &[String], section_nums: &[String]) -> u32 {
    let mut score = 0u32;

    // Direct section number match (e.g., searching "1.1")
    for num in section_nums {
        if chunk.section.starts_with(num) || chunk.chapter == *num {
            score += 10;
        }
    }

    let section_tokens = tokenize(&chunk.section);
    let text_tokens = tokenize(&chunk.text);

    for qt in query_tokens {
        if chunk.keywords.iter().any(|k| tokens_match(k, qt)) {
            score += 5;
        } else if section_tokens.iter().any(|s| tokens_match(s, qt)) {
            score += 3;
        } else if text_tokens.iter().any(|t| tokens_match(t, qt)) {
            score += 1;
        }
    }

    score
}

// --- RAG Pipeline ---

pub struct RagPipeline {
    store: Arc<DocStore>,
}

impl RagPipeline {
    pub fn new(store: Arc<DocStore>) -> Self {
        Self { store }
    }

    pub async fn process_query(&self, query: &str) -> Result<QueryResult> {
        let (chunk, score) = self
            .store
            .search(query)
            .await
            .with_context(|| format!("No matching documentation found for '{}'", query))?;

        let explanation = synthesize_explanation(query, &chunk, score);

        Ok(QueryResult {
            raw_text: chunk.text,
            explanation,
            chapter: chunk.chapter,
            section: chunk.section,
            url: chunk.url,
            score,
        })
    }
}

fn synthesize_explanation(query: &str, chunk: &DocChunk, score: u32) -> String {
    let query_clean = query.trim().trim_matches('?');
    let text_clean = chunk.text.trim_end_matches('.');
    
    if score >= 5 {
        format!(
            "In Rust, regarding '{}': {}. This is covered under Section {} (Chapter {}).",
            query_clean, text_clean, chunk.section, chunk.chapter
        )
    } else {
        format!(
            "Based on Section {} (Chapter {}), {}. This relates to your query regarding '{}'.",
            chunk.section, chunk.chapter, text_clean, query_clean
        )
    }
}

// --- Helpers & Seed Database ---

async fn read_line() -> Result<String> {
    tokio::task::spawn_blocking(|| {
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).context("Failed to read stdin")?;
        Ok(buf.trim().to_string())
    })
    .await
    .context("Input reader task panicked")?
}

fn create_chunk(
    chapter: &str, 
    section: &str, 
    keywords: &[&'static str], 
    text: &str, 
    url: &str
) -> DocChunk {
    DocChunk {
        chapter: chapter.into(),
        section: section.into(),
        keywords: keywords.to_vec(),
        text: text.into(),
        url: url.into(),
    }
}

fn seed_rust_docs() -> Vec<DocChunk> {
    vec![
        create_chunk(
            "1", 
            "1.1 Getting Started with Rust", 
            &["rust", "language", "overview", "introduction", "getting"], 
            "Rust is a systems programming language focused on safety, speed, and concurrency.",
            "https://doc.rust-lang.org/book/ch01-01-installation.html"
        ),
        create_chunk(
            "1", 
            "1.3 Hello Cargo", 
            &["cargo", "package", "manager", "build", "tool", "dependencies"], 
            "Cargo is Rust's build system and package manager that manages building code and downloading dependencies.",
            "https://doc.rust-lang.org/book/ch01-03-hello-cargo.html"
        ),
        create_chunk(
            "3", 
            "3.1 Variables & Mutability", 
            &["variable", "let", "mut", "mutability"], 
            "Variables are immutable by default in Rust; add `mut` to allow reassignment.",
            "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html"
        ),
        create_chunk(
            "4", 
            "4.1 What is Ownership", 
            &["ownership", "owner", "drop", "move", "memory"], 
            "Each value in Rust has one owner at a time. When out of scope, the value is dropped.",
            "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html"
        ),
        create_chunk(
            "4", 
            "4.2 References & Borrowing", 
            &["borrow", "reference", "mutable", "borrowing"], 
            "Borrowing lets you refer to data without taking ownership. Either 1 mutable ref or many immutable refs.",
            "https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html"
        ),
        create_chunk(
            "5", 
            "5.1 Defining and Instantiating Structs", 
            &["struct", "structs", "field", "data", "object"], 
            "A struct is a custom data type that lets you package together and name multiple related values.",
            "https://doc.rust-lang.org/book/ch05-01-defining-structs.html"
        ),
        create_chunk(
            "5", 
            "5.3 Method Syntax & Impl Blocks", 
            &["impl", "method", "self", "function"], 
            "Methods are defined within an impl block. They are functions associated with a specific struct, enum, or trait.",
            "https://doc.rust-lang.org/book/ch05-03-method-syntax.html"
        ),
        create_chunk(
            "9", 
            "9.2 Recoverable Errors with Result", 
            &["error", "errors", "handling", "result", "panic", "unwrap"], 
            "Error handling in Rust relies on the Result enum for recoverable errors or panic! for unrecoverable errors.",
            "https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html"
        ),
        create_chunk(
            "10", 
            "10.2 Traits: Defining Shared Behavior", 
            &["trait", "traits", "interface", "polymorphism", "behavior"], 
            "Traits define shared functionality across different types, similar to interfaces in other languages.",
            "https://doc.rust-lang.org/book/ch10-02-traits.html"
        ),
        create_chunk(
            "10", 
            "10.3 Validating References with Lifetimes", 
            &["lifetime", "lifetimes", "reference", "validity"], 
            "Lifetimes ensure references remain valid for as long as needed to prevent dangling references.",
            "https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html"
        ),
        create_chunk(
            "16", 
            "16.1 Tokio & Concurrency", 
            &["async", "tokio", "concurrency", "concurrent", "task", "future", "thread"], 
            "Concurrency in Rust allows multi-threaded scheduling or async tasks using runtimes like Tokio.",
            "https://doc.rust-lang.org/book/ch16-01-threads.html"
        ),
        create_chunk(
            "17", 
            "17.1 Characteristics of Object-Oriented Languages", 
            &["object", "oriented", "oop", "encapsulation", "inheritance"], 
            "Rust incorporates object-oriented features like structs (data) and traits (shared behavior) instead of traditional class inheritance.",
            "https://doc.rust-lang.org/book/ch17-01-what-is-oo.html"
        ),
    ]
}

// --- Main Program ---

#[tokio::main]
async fn main() -> Result<()> {
    let app_description = "Rustdoc RAG CLI Assistant (Offline)\nRetrieves documentation chunks and links directly to official Rust book sections.";
    println!("{}\n", app_description);

    let store = Arc::new(DocStore::new(seed_rust_docs()));
    let pipeline = RagPipeline::new(store.clone());

    println!("Type a query or 'exit' to quit.\n");

    loop {
        print!("query> ");
        io::stdout().flush()?;

        let line = read_line().await?;
        if line.is_empty() {
            continue;
        }
        if line.eq_ignore_ascii_case("exit") || line.eq_ignore_ascii_case("quit") {
            println!("Exiting assistant.");
            break;
        }

        match pipeline.process_query(&line).await {
            Ok(result) => {
                println!("\nSource      : Chapter {} ({})", result.chapter, result.section);
                println!("Score       : {}", result.score);
                println!("Doc Link    : {}", result.url);
                println!("Passage     : {}", result.raw_text);
                println!("Explanation : {}\n", result.explanation);
            }
            Err(err) => println!("\nError: {}\n", err),
        }
    }

    Ok(())
}
