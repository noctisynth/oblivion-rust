//! # Oblivion Decryptor
use ring::aead::Aad;
use ring::aead::BoundKey;
use ring::aead::OpeningKey;
use ring::aead::UnboundKey;
use ring::aead::AES_128_GCM;
use ring::error::Unspecified;

use super::gear::AbsoluteNonceSequence;

/// Decrypts the given cipher bytes using the given key and nonce.
/// Returns the decrypted data as a vector of bytes.
pub fn decrypt_bytes(
    cipher_bytes: Vec<u8>,
    tag: &[u8],
    aes_key: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, Unspecified> {
    let unbound_key = UnboundKey::new(&AES_128_GCM, aes_key)?;
    let nonce_sequence = AbsoluteNonceSequence::new(nonce);

    let mut opening_key = OpeningKey::new(unbound_key, nonce_sequence);
    let mut in_out = [cipher_bytes, tag.to_vec()].concat(); // 复制一份
    let decrypted_data = opening_key.open_in_place(Aad::empty(), &mut in_out)?;

    Ok(decrypted_data.to_vec())
}
