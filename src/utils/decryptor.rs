use ring::aead::Aad;
use ring::aead::BoundKey;
use ring::aead::OpeningKey;
use ring::aead::UnboundKey;
use ring::aead::AES_128_GCM;
use rsa::{Oaep, RsaPrivateKey};
use sha2::Sha256;

use super::gear::RandNonceSequence;

pub(crate) fn decrypt_aes_key(data: &[u8], private_key: RsaPrivateKey) -> Vec<u8> {
    // 使用 RSA 加密 AES_KEY
    let dec_data = private_key
        .decrypt(Oaep::new::<Sha256>(), &data)
        .expect("failed to decrypt");
    dec_data
}

pub(crate) fn decrypt_message(ciphertext: Vec<u8>, tag: &[u8], aes_key: &[u8], nonce: &[u8]) -> String {
    // 使用 AES_KEY 加密
    let unbound_key = match UnboundKey::new(&AES_128_GCM, &aes_key) {
        Ok(result) => result,
        Err(_) => return String::new(),
    };
    let nonce_sequence = RandNonceSequence::new(nonce.to_vec());

    let associated_data = Aad::empty();

    let mut opening_key = OpeningKey::new(unbound_key, nonce_sequence);

    let mut in_out = [ciphertext.clone(), tag.to_vec()].concat(); // 复制一份

    let decrypted_data = opening_key.open_in_place(associated_data, &mut in_out).expect("Failed to decrypt");

    // 示例代码给了一个必然报错的代码不知道有什么用`assert_eq!(data, decrypted_data);`
    String::from_utf8(decrypted_data.to_vec()).unwrap()
}
