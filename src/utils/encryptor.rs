extern crate ring;

use ring::aead::Aad;
use ring::aead::BoundKey;
use ring::aead::SealingKey;
use ring::aead::UnboundKey;
use ring::aead::AES_128_GCM;
use ring::aead::NONCE_LEN;
use ring::rand::SecureRandom;
use ring::rand::SystemRandom;

use super::gear::RandNonceSequence;

pub fn encrypt_message(message: String, aes_key: &[u8]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    // 使用 AES 加密数据
    let data = message.as_bytes().to_owned();
    encrypt_bytes(data, aes_key)
}

pub fn encrypt_bytes(bytes: Vec<u8>, aes_key: &[u8]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    // 使用 AES 加密数据
    let unbound_key =
        UnboundKey::new(&AES_128_GCM, &aes_key).expect("Failed to generate an unboundkey");

    let mut nonce_bytes = vec![0; NONCE_LEN];
    let rand = SystemRandom::new();
    match rand.fill(&mut nonce_bytes) {
        Ok(_) => {}
        Err(_) => {}
    };
    let nonce_sequence = RandNonceSequence::new(nonce_bytes.clone()); // Nonce 生成方法
    let mut sealing_key = SealingKey::new(unbound_key, nonce_sequence);

    let associated_data = Aad::empty();
    let mut in_out = bytes.clone();
    let tag = match sealing_key.seal_in_place_separate_tag(associated_data, &mut in_out) {
        Ok(result) => result,
        Err(_) => return (Vec::new(), Vec::new(), nonce_bytes),
    };

    (in_out, tag.as_ref().to_owned(), nonce_bytes)
}
