use criterion::{black_box, criterion_group, criterion_main, Criterion};
use embedder::file_loader::{load_codebase_into_chunks, FileChunk};

fn benchmark_load_codebase_into_chunks(c: &mut Criterion) {
    c.bench_function("load_codebase_into_chunks", |b| {
        b.iter(|| {
            let root_dir = black_box("app/tests/resources/sample");
            let max_chunk_size = black_box(1024);
            load_codebase_into_chunks(root_dir, max_chunk_size).unwrap();
        })
    });
}

criterion_group!(benches, benchmark_load_codebase_into_chunks);
criterion_main!(benches);
