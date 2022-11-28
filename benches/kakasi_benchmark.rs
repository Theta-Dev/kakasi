use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SHORT: &str = "安定版となるRust 1.0がリリースされた[84]。1.0版の後、安定版およびベータ版が6週間おきに定期リリースされている[85]。";
const RUST_ARTICLE: &str = include_str!("../tests/rust_article.txt");

fn benchmark(c: &mut Criterion) {
    println!("short length: {}", SHORT.len());
    c.bench_function("short", |b| b.iter(|| convert(SHORT)));

    println!("rust_article length: {}", RUST_ARTICLE.len());
    c.bench_function("rust_article", |b| b.iter(|| convert(RUST_ARTICLE)));
}

fn convert(text: &str) {
    kakasi::convert(black_box(text));
}

criterion_group! {benches, benchmark}
criterion_main!(benches);
