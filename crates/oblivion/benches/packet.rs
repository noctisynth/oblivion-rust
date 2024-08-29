use criterion::{criterion_group, criterion_main, Criterion};
use oblivion::{models::packet::OED, utils::generator::generate_random_salt};

fn criterion_benchmark_oed(c: &mut Criterion) {
    let aes_key = generate_random_salt();
    let long_data = vec![0u8; 1024 * 1024];
    c.bench_function("oed", |b| {
        b.iter(|| {
            OED::new(&aes_key)
                .from_bytes(long_data.clone())
                .unwrap()
                .plain_data()
                .unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark_oed);
criterion_main!(benches);
