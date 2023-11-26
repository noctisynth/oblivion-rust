pub mod api;
pub mod exceptions;
pub mod sessions;
pub mod utils {
    pub mod decryptor;
    pub mod encryptor;
    pub mod gear;
    pub mod generator;
    pub mod parser;
}

#[allow(dead_code)]
pub mod models {
    pub mod packet;
    pub mod client;
}

// use crate::utils::{
//     decryptor::decrypt_bytes,
//     encryptor::encrypt_message,
//     generator::{generate_key_pair, generate_shared_key},
// };
// use utils::generator::generate_random_salt;

