use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
fn corpus() -> [(&'static str, &'static str); 3] {
    [
        ("small", include_str!("../benchmarks/corpus/small.md")),
        ("tables", include_str!("../benchmarks/corpus/tables.md")),
        ("stress", include_str!("../benchmarks/corpus/stress.md")),
    ]
}
fn main() {}
