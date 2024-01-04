//! Oblivion Generator
extern crate rand;
extern crate ring;

use elliptic_curve::rand_core::OsRng;
use p256::{ecdh::EphemeralSecret, EncodedPoint, PublicKey};
use ring::aead::AES_128_GCM;
use ring::rand::{SecureRandom, SystemRandom};
use scrypt::{scrypt, Params};

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
/// use oblivion::utils::generator::{generate_key_pair, generate_shared_key, generate_random_salt};
///
/// let salt = generate_random_salt();
/// let (private_key, public_key) = generate_key_pair().unwrap();
///
/// let shared_key = generate_shared_key(&private_key, &public_key, &salt).unwrap();
/// ```
pub fn generate_shared_key(
    private_key: &EphemeralSecret,
    public_key: &PublicKey,
    salt: &[u8],
) -> Result<Vec<u8>, OblivionException> {
    let shared_key = private_key.diffie_hellman(&public_key);
    let mut aes_key = [0u8; 16];
    match scrypt(
        &shared_key.raw_secret_bytes().to_vec(),
        &salt,
        &Params::new(12, 8, 1, 16).unwrap(),
        &mut aes_key,
    ) {
        Ok(_) => Ok(aes_key.to_vec()),
        Err(error) => Err(OblivionException::InvalidOutputLen { error: error }),
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
