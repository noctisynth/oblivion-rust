//! # Oblivion exception
//! All exceptions to the Oblivion function return `OblivionException`.
use ring::error::Unspecified;
use scrypt::errors::InvalidOutputLen;
use thiserror::Error;
#[cfg(feature = "python")]
use pyo3::prelude::*;

/// ## Oblivion exception iterator
/// Use an iterator as the type of exception returned by a function.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum OblivionException {
    #[error("Request not yet pre-processed")]
    ErrorNotPrepared,
    #[error("Incorrect protocol header: {header}")]
    BadProtocol { header: String },
    #[error("Link requests to the server are denied, either due to insufficient privileges or an attack on the server.")]
    ConnectionRefusedError,
    #[error("Wrong Oblivion address: {olps}")]
    InvalidOblivion { olps: String },
    #[error("Destination address [{ipaddr}:{port}] is already occupied.")]
    AddressAlreadyInUse { ipaddr: String, port: i32 },
    #[error("Unexpected disconnection from the remote host, possibly due to manual disconnection or network censorship.")]
    UnexpectedDisconnection,
    #[error("Failed to decode the transmitted byte stream.")]
    BadBytes,
    #[error(
        "The request was timed out, either due to a network problem or an attack on the server."
    )]
    ConnectTimedOut,
    #[error("Exceeded expected packet size: {size}")]
    DataTooLarge { size: usize },
    #[error("All request attempts failed: {times}")]
    AllAttemptsRetryFailed { times: i32 },
    #[error("Method [{method}] is not supported yet.")]
    UnsupportedMethod { method: String },
    #[error("Oblivion/1.1 {method} From {ipaddr} {olps} {status_code}")]
    ServerError {
        method: String,
        ipaddr: String,
        olps: String,
        status_code: i32,
    },
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
}

#[cfg(feature = "python")]
#[pyclass]
pub struct PyOblivionException {
    pub message: String,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyOblivionException {
    #[new]
    fn new(message: String) -> Self {
        Self { message }
    }
}
