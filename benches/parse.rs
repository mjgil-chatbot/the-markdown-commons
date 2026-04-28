use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn corpus() -> [(&'static str, &'static str); 3] {
    [
        ("small", include_str!("../benchmarks/corpus/small.md")),
        ("tables", include_str!("../benchmarks/corpus/tables.md")),
        ("stress", include_str!("../benchmarks/corpus/stress.md")),
    ]
}

fn bench_parse_markdown(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_markdown");
    for (name, markdown) in corpus() {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &markdown,
            |b, markdown| {
                b.iter(|| tmc::parse_markdown(markdown));
            },
        );
    }
    group.finish();
}

fn bench_parse_markdown_fragment(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_markdown_fragment");
    for (name, markdown) in corpus() {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &markdown,
            |b, markdown| {
                b.iter(|| tmc::parse_markdown_fragment(markdown));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_parse_markdown, bench_parse_markdown_fragment);
criterion_main!(benches);
