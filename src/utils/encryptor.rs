extern crate ring;

use rand::rngs::OsRng;
use ring::aead::Aad;
use ring::aead::BoundKey;
use ring::aead::SealingKey;
use ring::aead::UnboundKey;
use ring::aead::AES_256_GCM;
use ring::aead::NONCE_LEN;
use ring::rand::SecureRandom;
use ring::rand::SystemRandom;
use rsa::{Oaep, RsaPublicKey};
use sha2::Sha256;

use super::gear::RandNonceSequence;

pub(crate) fn encrypt_message(message: String, aes_key: &[u8]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let data = message.as_bytes().to_owned();
    // 使用 AES 加密数据
    let unbound_key =
        UnboundKey::new(&AES_256_GCM, &aes_key).expect("Failed to generate an unboundkey");

    let mut nonce_bytes = vec![0; NONCE_LEN];
    let rand = SystemRandom::new();
    match rand.fill(&mut nonce_bytes) {
        Ok(_) => {}
        Err(_) => {}
    };
    let nonce_sequence = RandNonceSequence::new(nonce_bytes.clone()); // Nonce 生成方法
    let mut sealing_key = SealingKey::new(unbound_key, nonce_sequence);

    let associated_data = Aad::empty();
    let mut in_out = data.clone();
    let tag = match sealing_key.seal_in_place_separate_tag(associated_data, &mut in_out) {
        Ok(result) => result,
        Err(_) => return (Vec::new(), Vec::new(), nonce_bytes),
    };

    (in_out, tag.as_ref().to_owned(), nonce_bytes)
}

pub(crate) fn encrypt_aes_key(data: &[u8], public_key: RsaPublicKey) -> Vec<u8> {
    // Encrypt
    let mut rng = OsRng;
    let enc_data = public_key
        .encrypt(&mut rng, Oaep::new::<Sha256>(), &data[..])
        .expect("Failed to encrypt");
    enc_data
}
