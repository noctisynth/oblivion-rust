use criterion::{criterion_group, criterion_main, Criterion};
use oblivion::{models::packet::OED, utils::generator::generate_random_salt};

fn criterion_benchmark_oed(c: &mut Criterion) {
    let aes_key = generate_random_salt();
    c.bench_function("oed", |b| {
        b.iter(|| {
            OED::new(&aes_key)
                .from_bytes([0u8; 1e6 as usize].to_vec())
                .unwrap()
                .plain_data()
                .unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark_oed);
criterion_main!(benches);
