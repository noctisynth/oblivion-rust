//! # Oblivion Generator
extern crate rand;
extern crate ring;

use anyhow::Result;
use hkdf::Hkdf;

#[cfg(feature = "unsafe")]
use elliptic_curve::rand_core::OsRng;
#[cfg(feature = "unsafe")]
use p256::{ecdh::EphemeralSecret, PublicKey};
#[cfg(not(feature = "unsafe"))]
use ring::agreement::{agree_ephemeral, EphemeralPrivateKey, PublicKey, UnparsedPublicKey, X25519};

use ring::rand::SystemRandom;
use ring::{aead::AES_128_GCM, rand::SecureRandom};
use scrypt::{scrypt, Params};
use sha2::Sha256;

use crate::exceptions::Exception;

/// Create an ECC key
///
/// `generate_key_pair` will create an ECC key and return a (private key, public key) pair of `(EphemeralSecret, PublicKey)`.
/// 
/// We use `X25519` curve for ECC operations.
///
/// ```rust
/// # use oblivion::utils::generator::generate_key_pair;
/// let (private_key, public_key) = generate_key_pair();
/// ```
#[cfg(not(feature = "unsafe"))]
pub fn generate_key_pair() -> (EphemeralPrivateKey, PublicKey) {
    let rng = SystemRandom::new();
    let private_key = EphemeralPrivateKey::generate(&X25519, &rng).unwrap();
    let public_key = private_key.compute_public_key().unwrap();
    (private_key, public_key)
}

#[cfg(feature = "unsafe")]
pub fn generate_key_pair() -> Result<(EphemeralSecret, PublicKey), Exception> {
    let private_key = EphemeralSecret::random(&mut OsRng);
    let public_key = private_key.public_key();
    Ok((private_key, public_key))
}

/// Generate a Shared Key
///
/// `SharedKey` is a struct that can generate a shared key using HKDF or Scrypt.
///
/// The shared key is generated using the private key and the public key of the other party.
///
/// # Examples
///
/// ```rust
/// # use oblivion::utils::generator::{generate_key_pair, generate_random_salt, SharedKey};
/// # use ring::agreement::{UnparsedPublicKey, X25519};
/// # #[cfg(not(feature = "unsafe"))]
/// # {
/// let salt = generate_random_salt();
///
/// let (private_key, public_key) = generate_key_pair();
///
/// let public_key: UnparsedPublicKey<Vec<u8>> = {
///     // Convert the public key to UnparsedPublicKey
///     let public_key_bytes = public_key.as_ref().to_vec();
///     UnparsedPublicKey::new(&X25519, public_key_bytes)
/// };
///
/// let mut shared_key = SharedKey::new(private_key, &public_key).unwrap();
///
/// shared_key.hkdf(&salt); // Generate a shared key using HKDF
/// shared_key.scrypt(&salt).unwrap(); // Generate a shared key using Scrypt
/// # }
/// ```
///
/// Now oblivion uses `ring` instead of `p256` for ECC operations. The `SharedKey` struct is updated to use `ring` instead of `p256`.
///
/// If you still want to use a deprecated version of the library, you can use the following code:
///
/// ```rust
/// # use oblivion::utils::generator::{generate_key_pair, generate_random_salt, SharedKey};
/// # use ring::agreement::{UnparsedPublicKey, X25519};
/// # #[cfg(feature = "unsafe")]
/// # {
/// let (private_key, public_key) = generate_key_pair().unwrap();
/// let mut shared_key = SharedKey::new(&private_key, &public_key);
///
/// shared_key.hkdf(&salt); // Generate a shared key using HKDF
/// shared_key.scrypt(&salt).unwrap(); // Generate a shared key using Scrypt
/// # }
/// ```
pub struct SharedKey {
    shared_key: Vec<u8>,
}

impl SharedKey {
    #[cfg(feature = "unsafe")]
    pub fn new(private_key: &EphemeralSecret, public_key: &PublicKey) -> Self {
        Self {
            shared_key: private_key
                .diffie_hellman(&public_key)
                .raw_secret_bytes()
                .to_vec(),
        }
    }

    #[cfg(not(feature = "unsafe"))]
    pub fn new(
        private_key: EphemeralPrivateKey,
        public_key: &UnparsedPublicKey<Vec<u8>>,
    ) -> Result<Self> {
        match agree_ephemeral(private_key, public_key, |key| key.to_vec()) {
            Ok(shared_key) => Ok(Self { shared_key }),
            Err(error) => Err(Exception::DecryptError { error }.into()),
        }
    }

    pub fn scrypt(&mut self, salt: &[u8]) -> Result<Vec<u8>> {
        let mut aes_key = [0u8; 16];
        match scrypt(
            &self.shared_key,
            &salt,
            &Params::new(12, 8, 1, 16).unwrap(),
            &mut aes_key,
        ) {
            Ok(()) => Ok(aes_key.to_vec()),
            Err(error) => Err(Exception::InvalidOutputLen { error }.into()),
        }
    }

    pub fn hkdf(&mut self, salt: &[u8]) -> [u8; 16] {
        let key = Hkdf::<Sha256>::new(Some(salt), &self.shared_key);
        let mut aes_key = [0u8; 16];
        key.expand(&[], &mut aes_key).unwrap();
        aes_key
    }
}

/// Generate a Randomized Salt
///
/// `generate_random_salt` will generate a random salt using the `ring` library.
///
/// The length of the salt is 16 bytes, which is the length of the key used for AES-GCM encryption.
///
/// # Example
/// ```rust
/// # use oblivion::utils::generator::generate_random_salt;
/// let salt = generate_random_salt();
/// ```
pub fn generate_random_salt() -> Vec<u8> {
    let rng = SystemRandom::new();
    let mut key_bytes = vec![0; AES_128_GCM.key_len()];
    rng.fill(&mut key_bytes).unwrap();
    key_bytes
}
