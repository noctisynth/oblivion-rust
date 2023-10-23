extern crate rand;
extern crate ring;

use ring::rand::{SecureRandom, SystemRandom};
use rsa::{RsaPrivateKey, RsaPublicKey};
use ring::aead::AES_256_GCM;

pub(crate) fn generate_aes_key() -> Vec<u8> {
    // 生成随机的 AES_KEY
    let rand = SystemRandom::new();
    let mut key_bytes = vec![0; AES_256_GCM.key_len()];
    match rand.fill(&mut key_bytes) {
        Ok(_) => {},
        Err(_) => {}
    };
    key_bytes
}

pub(crate) fn generate_key_pair() -> (RsaPrivateKey, RsaPublicKey) {
    // 生成 RSA 密钥对
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);
    (private_key, public_key)
}
