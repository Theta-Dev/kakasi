use criterion::{black_box, criterion_group, criterion_main, Criterion};

const RUST_ARTICLE: &str = include_str!("../tests/rust_article.txt");

fn rust_article(c: &mut Criterion) {
    c.bench_function("rust_article", |b| b.iter(|| {
        kakasi::convert(black_box(RUST_ARTICLE))
    }));
}

criterion_group!(benches, rust_article);
criterion_main!(benches);
