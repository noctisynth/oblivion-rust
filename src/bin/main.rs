use oblivion::utils::decryptor::decrypt_bytes;
use oblivion::utils::generator::{generate_key_pair, generate_random_salt, generate_shared_key};
use oblivion::utils::encryptor::encrypt_message;

#[tokio::main]
async fn main() {
    let (alice_secret, alice_public) = generate_key_pair();
    let (bob_secret, bob_public) = generate_key_pair();

    let salt = generate_random_salt();

    let alice_aes_key = generate_shared_key(&alice_secret, bob_public, &salt);
    let bob_aes_key = generate_shared_key(&bob_secret, alice_public, &salt);

    let msg = "Hello world!";
    assert_eq!(alice_aes_key, bob_aes_key);
    let (cipherbytes, tag, nonce) = encrypt_message(msg.to_string(), &alice_aes_key);
    assert_eq!(
        decrypt_bytes(cipherbytes, &tag, &bob_aes_key, &nonce).unwrap(),
        msg.as_bytes()
    );
}
