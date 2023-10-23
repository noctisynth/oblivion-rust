pub mod models;
pub mod exceptions;
pub mod sessions;
pub mod api;
mod utils {
    pub mod generator;
    pub mod encryptor;
    pub mod decryptor;
    pub mod parser;
    pub mod gear;
}
use utils::generator::{generate_key_pair, generate_aes_key};
use utils::encryptor::{encrypt_aes_key, encrypt_message};
use utils::decryptor::{decrypt_aes_key, decrypt_message};
use utils::parser::length;

use api::{request, get, post, forward};

fn test_encrypt() {
    println!("Begin!");
    let msg = "Hello World!".to_string();

    let aes_key = generate_aes_key();

    let (private_key, public_key) = generate_key_pair();
    let enc_aes_key = encrypt_aes_key(&aes_key, public_key);
    let dec_aes_key = decrypt_aes_key(&enc_aes_key, private_key);

    let (message, tag, nonce) = encrypt_message(msg, &dec_aes_key);
    println!("messgae: {:?}", message);
    println!("{}", decrypt_message(message, &tag, &aes_key, &nonce));
    println!("nonce: {:?}", nonce);
    println!("tag: {:?}", tag);
    println!("aes_key: {:?}", aes_key);
    println!("End!");
}

fn test_length() {
    let input_string = b"example";
    let result = length(&input_string.to_vec());
    println!("{:?}", result.len());
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
    test_length();
    test_api();
    // test_just_encrypt()
}
