pub mod api;
pub mod exceptions;
pub mod models;
pub mod sessions;
mod utils {
    pub mod decryptor;
    pub mod encryptor;
    pub mod gear;
    pub mod generator;
    pub mod parser;
}
use utils::decryptor::{decrypt_aes_key, decrypt_message};
use utils::encryptor::{encrypt_aes_key, encrypt_message};
use utils::generator::{generate_aes_key, generate_key_pair};

use api::get;

fn test_encrypt() {
    println!("Begin!");
    let msg = "Hello World!".to_string();

    let aes_key = generate_aes_key();

    let (private_key, public_key) = generate_key_pair();
    let enc_aes_key = encrypt_aes_key(&aes_key, public_key);
    let dec_aes_key = decrypt_aes_key(&enc_aes_key, private_key);

    let (message, tag, nonce) = encrypt_message(msg, &dec_aes_key);
    println!("{}", decrypt_message(message, &tag, &aes_key, &nonce));
    println!("End!");
}

fn test_api() {
    // let req = request("get", "oblivion://127.0.0.1:80/test");
    let req = get("oblivion://127.0.0.1:80/test");
    // let req = post("oblivion://127.0.0.1:80/test");
    // let req = forward("oblivion://127.0.0.1:80/test");
    println!("{}", req);
}

fn main() {
    test_encrypt();
    test_api();
}
