use anyhow::Result;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oblivion::utils::generator::{generate_key_pair, generate_random_salt, SharedKey};
use tokio::runtime::Runtime;

use ring::agreement::{EphemeralPrivateKey, PublicKey, UnparsedPublicKey, X25519};

async fn hkdf(
    private_key: EphemeralPrivateKey,
    public_key: PublicKey,
    salt: Vec<u8>,
) -> Result<()> {
    let mut shared_key = SharedKey::new(
        private_key,
        &UnparsedPublicKey::new(&X25519, public_key.as_ref().to_vec()),
    )?;
    shared_key.hkdf(&salt);
    Ok(())
}

async fn scrypt(
    private_key: EphemeralPrivateKey,
    public_key: PublicKey,
    salt: Vec<u8>,
) -> Result<()> {
    let mut shared_key = SharedKey::new(
        private_key,
        &UnparsedPublicKey::new(&X25519, public_key.as_ref().to_vec()),
    )?;
    shared_key.scrypt(&salt)?;
    Ok(())
}

fn criterion_benchmark_keygen(c: &mut Criterion) {
    c.bench_function("keygen", |b| b.iter(generate_key_pair));
}

fn criterion_benchmark_kdf(c: &mut Criterion) {
    c.bench_function("kdf", |b| {
        b.to_async(Runtime::new().unwrap()).iter(|| async {
            let (prk, puk) = generate_key_pair();
            let salt = generate_random_salt();
            hkdf(black_box(prk), black_box(puk), black_box(salt.to_vec())).await
        })
    });
}

fn criterion_benchmark_scrypt(c: &mut Criterion) {
    c.bench_function("scrypt", |b| {
        b.to_async(Runtime::new().unwrap()).iter(|| async {
            let (prk, puk) = generate_key_pair();
            let salt = generate_random_salt();
            scrypt(black_box(prk), black_box(puk), black_box(salt)).await
        })
    });
}

criterion_group!(
    benches,
    criterion_benchmark_keygen,
    criterion_benchmark_kdf,
    criterion_benchmark_scrypt
);
criterion_main!(benches);
