//! # Oblivion Generator
extern crate rand;
extern crate ring;

use anyhow::Result;
use elliptic_curve::rand_core::OsRng;
use p256::ecdh::SharedSecret;
use p256::{ecdh::EphemeralSecret, EncodedPoint, PublicKey};
use ring::aead::AES_128_GCM;
use ring::rand::{SecureRandom, SystemRandom};
use scrypt::{scrypt, Params};
use sha2::Sha256;

use crate::exceptions::OblivionException;

/// Create an ECC key
///
/// `generate_key_pair` will create an ECC key and return a (private key, public key) pair of `(EphemeralSecret, PublicKey)`.
///
/// ```rust
/// use oblivion::utils::generator::generate_key_pair;
///
/// let (private_key, public_key) = generate_key_pair().unwrap();
/// ```
pub fn generate_key_pair() -> Result<(EphemeralSecret, PublicKey), OblivionException> {
    let private_key = EphemeralSecret::random(&mut OsRng);
    let pk_bytes = EncodedPoint::from(private_key.public_key());
    match PublicKey::from_sec1_bytes(pk_bytes.as_ref()) {
        Ok(public_key) => Ok((private_key, public_key)),
        Err(error) => Err(OblivionException::PublicKeyInvalid { error }),
    }
}

/// Create an ECDH Shared Key
///
/// ```rust
/// use oblivion::utils::generator::{generate_key_pair, generate_random_salt, SharedKey};
///
/// let salt = generate_random_salt();
/// let (private_key, public_key) = generate_key_pair().unwrap();
///
/// let mut shared_key = SharedKey::new(&private_key, &public_key);
/// 
/// shared_key.hkdf(&salt);
/// shared_key.scrypt(&salt);
/// ```
pub struct SharedKey {
    shared_key: SharedSecret,
}

impl SharedKey {
    pub fn new(private_key: &EphemeralSecret, public_key: &PublicKey) -> Self {
        Self {
            shared_key: private_key.diffie_hellman(&public_key),
        }
    }

    pub fn scrypt(&mut self, salt: &[u8]) -> Result<Vec<u8>> {
        let mut aes_key = [0u8; 16];
        match scrypt(
            &self.shared_key.raw_secret_bytes().to_vec(),
            &salt,
            &Params::new(12, 8, 1, 16).unwrap(),
            &mut aes_key,
        ) {
            Ok(()) => Ok(aes_key.to_vec()),
            Err(error) => Err(OblivionException::InvalidOutputLen { error }.into()),
        }
    }

    pub fn hkdf(&mut self, salt: &[u8]) -> Result<Vec<u8>> {
        let key = self.shared_key.extract::<Sha256>(Some(salt));
        let mut aes_key = [0u8; 16];
        key.expand(&[], &mut aes_key).unwrap();
        Ok(aes_key.to_vec())
    }
}

/// Generate a Randomized Salt
/// ```rust
/// use oblivion::utils::generator::generate_random_salt;
///
/// let salt = generate_random_salt();
/// ```
pub fn generate_random_salt() -> Vec<u8> {
    let rand = SystemRandom::new();
    let mut key_bytes = vec![0; AES_128_GCM.key_len()];
    rand.fill(&mut key_bytes).unwrap();
    key_bytes
}
