//! # Oblivion exception
//! All exceptions to the Oblivion function return `OblivionException`.
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use ring::error::Unspecified;
use scrypt::errors::InvalidOutputLen;
use thiserror::Error;

/// ## Oblivion exception iterator
/// Use an iterator as the type of exception returned by a function.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum Exception {
    #[error("Invalid header: {0}")]
    InvalidHeader(String),
    #[error("Link requests to the server are denied, either due to insufficient privileges or an attack on the server.")]
    ConnectionRefusedError,
    #[error("Wrong Oblivion address: {entrance}")]
    InvalidOblivion { entrance: String },
    #[error("Exceeded expected packet size: {size}")]
    DataTooLarge { size: usize },
    #[error("Method [{method}] is not supported yet.")]
    UnsupportedMethod { method: String },
    #[cfg(feature = "unsafe")]
    #[error("Invalid public key: {error:?}")]
    PublicKeyInvalid {
        #[from]
        error: elliptic_curve::Error,
    },
    #[error("Exception during shared key generation: {error:?}")]
    InvalidOutputLen {
        #[from]
        error: InvalidOutputLen,
    },
    #[error("Exception while encrypting: {error:?}")]
    EncryptError { error: Unspecified },
    #[error("Exception while decrypting: {error:?}")]
    DecryptError { error: Unspecified },
    #[error("Trying to read or write a closed connection.")]
    ConnectionClosed,
}

#[cfg(feature = "pyo3")]
#[pyclass]
pub struct PyOblivionException {
    pub message: String,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl PyOblivionException {
    #[new]
    fn new(message: String) -> Self {
        Self { message }
    }
}
