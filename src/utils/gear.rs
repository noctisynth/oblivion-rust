// use std::collections::HashMap;

use ring::{
    aead::{Nonce, NonceSequence},
    error::Unspecified,
};

pub struct RandNonceSequence {
    nonce: Vec<u8>,
}

impl NonceSequence for RandNonceSequence {
    // called once for each seal operation
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        Nonce::try_assume_unique_for_key(&self.nonce)
    }
}

impl RandNonceSequence {
    pub fn new(nonce: Vec<u8>) -> Self {
        Self { nonce: nonce }
    }
}

// pub struct Kwargs {
//     args: HashMap<String, String>,
// }

// impl Kwargs {
//     pub fn new(map: HashMap<String, String>) -> Self {
//         Self { args: map }
//     }

//     fn get_args(&self) -> &HashMap<String, String> {
//         &self.args
//     }

//     pub fn get(&self, key: &str, default: &str) -> String {
//         match self.args.get(key) {
//             Some(value) => value.clone(),
//             None => default.to_string(),
//         }
//     }

//     pub fn insert(&mut self, key: &str, value: &str) {
//         self.args.insert(key.to_string(), value.to_string());
//     }

//     pub fn remove(&mut self, key: &str) -> bool {
//         self.args.remove(key).is_some()
//     }

//     pub fn haskey(&self, key: &str) -> bool {
//         self.args.contains_key(key)
//     }
// }
