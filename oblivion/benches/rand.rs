use criterion::{criterion_group, criterion_main, Criterion};
use oblivion::utils::generator::generate_random_salt;

fn criterion_benchmark_salt(c: &mut Criterion) {
    c.bench_function("salt", |b| b.iter(|| generate_random_salt()));
}

criterion_group!(benches, criterion_benchmark_salt);
criterion_main!(benches);
