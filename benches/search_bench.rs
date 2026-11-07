use criterion::{criterion_group, criterion_main, Criterion};
use rustdoc_assist::corpus;
use rustdoc_assist::embeddings::Embedder;
use rustdoc_assist::search::VectorIndex;
use std::hint::black_box;
use std::path::Path;

fn bench_search(c: &mut Criterion) {
    let chunks = corpus::load_corpus(Path::new("docs_cache/corpus.json")).unwrap();
    let texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
    let mut embedder = Embedder::new().unwrap();
    let vectors = embedder.embed_batch(&texts).unwrap();
    let index = VectorIndex::new(chunks, vectors);
    let query_vec = embedder.embed_query("what is ownership").unwrap();

    c.bench_function("search_top3", |b| {
        b.iter(|| index.search(black_box(&query_vec), 3))
    });
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
