# Rustdoc-assist

A local RAG (retrieval-augmented generation) CLI for querying the Rust Book —
built to learn real vector search and local ML inference in Rust, scoped to
match what I actually understand deeply rather than what sounds most
impressive.

## What this does

- Fetches real chapters from [rust-lang/book](https://github.com/rust-lang/book)
  on GitHub and chunks them by section heading (cached locally after first run).
- Embeds each chunk into a 384-dimensional vector using `all-MiniLM-L6-v2`,
  run locally via [`fastembed`](https://github.com/Anush008/fastembed-rs) —
  no API key, no external inference service.
- Ranks chunks by cosine similarity against the query's embedding
  (brute-force vector search — see Design notes below).
- Interactive CLI: type a question, get the closest matching Rust Book
  section back with a direct link.

## Architecture

```
rust-lang/book (GitHub)
        |
   corpus.rs   — fetch chapters (sync, ureq), chunk by ## heading,
        |         strip code fences and blockquotes
        |
embeddings.rs  — fastembed (all-MiniLM-L6-v2), local ONNX inference
        |         -> 384-dim vectors
        |
  search.rs    — brute-force cosine similarity over all chunk vectors
        |
  main.rs      — interactive CLI loop
```

## Running it

First run needs internet access once, to fetch the corpus and download the
embedding model (~90MB, cached afterward — every run after that is fully
offline).

```bash
git clone https://github.com/Raushan-tryingtocode/rustdoc-assist.git
cd rustdoc-assist
cargo run
```

## Design notes

**Why sync, not async?** Fetching the corpus and embedding it both happen
once at startup, and embedding itself is CPU-bound model inference, not I/O —
none of that benefits from an async runtime. A REST API version (Axum/Tokio,
serving concurrent queries) is a natural next step, and one I'm actively
building toward — but I'd rather ship a smaller version I fully understand
than a bigger one I can't yet explain end to end.

**Why brute-force cosine search instead of an ANN index?** At the current
corpus size (a few dozen chunks across 12 chapters), scanning every vector
per query is O(n) and genuinely fast — an ANN index (e.g. HNSW) would add
complexity with no measurable benefit here. Documenting that tradeoff here
instead of over-engineering it is deliberate.

## Benchmarks

_Fill in after running locally — methodology matters more than the numbers:_

- **Corpus size:** 16 chunks across 12 Rust Book chapters (confirmed from an actual run)
- **Indexing time:** printed on startup — e.g. 16 chunks in ~1.15s on a first-gen test run (varies by machine)
- **Per-query latency:** printed after each query in the CLI — observed ~4-8ms per query in testing
- **Hardware:** the machine you ran this on

## What changed from the first version

The original version was a keyword-matching CLI (tokenize + Levenshtein
fuzzy match against a hardcoded array of 12 doc snippets). This version
replaces that with real local embeddings, real cosine similarity vector
search, and a real corpus fetched from source. An async REST API version
(Tokio + Axum) is in progress in a separate branch as I get more comfortable
with async Rust — I'd rather build that depth for real than bolt it on early.

## Contributing

Feel free to open an issue or submit a pull request if you run into bugs or have suggestions.
