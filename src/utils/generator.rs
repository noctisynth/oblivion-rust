extern crate rand;
extern crate ring;

use elliptic_curve::rand_core::OsRng;
use p256::{ecdh::EphemeralSecret, EncodedPoint, PublicKey};
use ring::aead::AES_128_GCM;
use ring::rand::{SecureRandom, SystemRandom};
use scrypt::{scrypt, Params};

pub fn generate_key_pair() -> (EphemeralSecret, PublicKey) {
    // 生成 ECC 密钥对
    let private_key = EphemeralSecret::random(&mut OsRng);
    let pk_bytes = EncodedPoint::from(private_key.public_key());
    let public_key =
        PublicKey::from_sec1_bytes(pk_bytes.as_ref()).expect("bob's public key is invalid!");
    (private_key, public_key)
}

pub fn generate_shared_key(
    private_key: &EphemeralSecret,
    public_key: PublicKey,
    salt: &[u8],
) -> Vec<u8> {
    // 生成共享密钥对
    let shared_key = private_key.diffie_hellman(&public_key);
    let mut aes_key = [0u8; 16];
    let _ = scrypt(
        &shared_key.raw_secret_bytes().to_vec(),
        &salt,
        &Params::new(12, 8, 1, 16).unwrap(),
        &mut aes_key,
    );
    aes_key.to_vec()
}

pub fn generate_random_salt() -> Vec<u8> {
    // 生成随机的盐值
    let rand = SystemRandom::new();
    let mut key_bytes = vec![0; AES_128_GCM.key_len()];
    match rand.fill(&mut key_bytes) {
        Ok(_) => {}
        Err(_) => {}
    };
    key_bytes
}
