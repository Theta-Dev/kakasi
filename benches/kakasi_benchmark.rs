mod perf;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

const RUST_ARTICLE: &str = include_str!("../tests/rust_article.txt");

fn benchmark(c: &mut Criterion) {
    c.bench_function("short", |b| b.iter(|| {
        convert("安定版となるRust 1.0がリリースされた[84]。1.0版の後、安定版およびベータ版が6週間おきに定期リリースされている[85]。")
    }));

    c.bench_function("rust_article", |b| b.iter(|| {
        convert(RUST_ARTICLE)
    }));
}

fn convert(text: &str) {
    kakasi::convert(black_box(text));
}

criterion_group!{
    name = benches;
    config = Criterion::default().with_profiler(perf::FlamegraphProfiler::new(100));
    targets = benchmark
}
criterion_main!(benches);
