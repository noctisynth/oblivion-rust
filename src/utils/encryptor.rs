//! # Oblivion Encryptor
extern crate ring;

use ring::aead::Aad;
use ring::aead::BoundKey;
use ring::aead::SealingKey;
use ring::aead::UnboundKey;
use ring::aead::AES_128_GCM;
use ring::aead::NONCE_LEN;
use ring::rand::SecureRandom;
use ring::rand::SystemRandom;

use crate::exceptions::OblivionException;

use super::gear::AbsoluteNonceSequence;

/// Encrypt plaintext using AES
pub fn encrypt_plaintext(
    string: String,
    aes_key: &[u8],
) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), OblivionException> {
    let data = string.as_bytes().to_owned();
    encrypt_bytes(data, aes_key)
}

/// Encrypt binary data using AES
pub fn encrypt_bytes(
    mut bytes: Vec<u8>,
    aes_key: &[u8],
) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), OblivionException> {
    let unbound_key = match UnboundKey::new(&AES_128_GCM, &aes_key) {
        Ok(key) => key,
        Err(error) => return Err(OblivionException::EncryptError { error }),
    };

    let mut nonce_bytes = vec![0; NONCE_LEN];
    let rand = SystemRandom::new();
    rand.fill(&mut nonce_bytes).unwrap();

    let nonce_sequence = AbsoluteNonceSequence::new(nonce_bytes.clone());
    let mut sealing_key = SealingKey::new(unbound_key, nonce_sequence);

    let associated_data = Aad::empty();

    let tag = match sealing_key.seal_in_place_separate_tag(associated_data, &mut bytes) {
        Ok(result) => result,
        Err(error) => return Err(OblivionException::EncryptError { error }),
    };

    Ok((bytes, tag.as_ref().to_owned(), nonce_bytes))
}
