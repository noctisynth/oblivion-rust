//! Oblivion Abstract Gear
use crate::exceptions::OblivionException;
use ring::{
    aead::{Nonce, NonceSequence},
    error::Unspecified,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

/// Absolute Nonce Sequence Structure
///
/// This structure is used to pass in pre-generated Nonce directly.
///
/// Warning: this is not a generalized generation scheme and should not be used in production environments,
/// you should make sure that the Nonce you pass in is a sufficiently garbled byte string.
pub struct AbsoluteNonceSequence {
    nonce: Vec<u8>,
}

impl NonceSequence for AbsoluteNonceSequence {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        Nonce::try_assume_unique_for_key(&self.nonce)
    }
}

impl AbsoluteNonceSequence {
    pub fn new(nonce: Vec<u8>) -> Self {
        Self { nonce: nonce }
    }
}

/// Socket Abstract Structure
///
/// Used to abstract Oblivion's handling of transmitted data, wrapping all data type conversions.
pub struct Socket {
    pub tcp: TcpStream,
}

impl Socket {
    pub fn new(tcp: TcpStream) -> Self {
        Self { tcp }
    }

    pub fn set_ttl(&mut self, ttl: u32) {
        self.tcp.set_ttl(ttl).unwrap()
    }

    pub async fn recv_len(&mut self) -> Result<usize, OblivionException> {
        let mut len_bytes: Vec<u8> = vec![0; 4];
        match self.tcp.read_exact(&mut len_bytes).await {
            Ok(_) => {}
            Err(_) => return Err(OblivionException::UnexpectedDisconnection),
        };

        match std::str::from_utf8(&len_bytes) {
            Ok(len_int) => match len_int.parse() {
                Ok(len) => Ok(len),
                Err(_) => Err(OblivionException::BadBytes),
            },
            Err(_) => return Err(OblivionException::BadBytes),
        }
    }

    pub async fn recv_int(&mut self, len: usize) -> Result<i32, OblivionException> {
        let mut len_bytes: Vec<u8> = vec![0; len];
        match self.tcp.read_exact(&mut len_bytes).await {
            Ok(_) => {}
            Err(_) => return Err(OblivionException::UnexpectedDisconnection),
        };

        match std::str::from_utf8(&len_bytes) {
            Ok(len_int) => match len_int.parse() {
                Ok(len) => Ok(len),
                Err(_) => Err(OblivionException::BadBytes),
            },
            Err(_) => return Err(OblivionException::BadBytes),
        }
    }

    pub async fn recv(&mut self, len: usize) -> Result<Vec<u8>, OblivionException> {
        let mut recv_bytes: Vec<u8> = vec![0; len];
        match self.tcp.read_exact(&mut recv_bytes).await {
            Ok(_) => Ok(recv_bytes),
            Err(_) => Err(OblivionException::UnexpectedDisconnection),
        }
    }

    pub async fn recv_str(&mut self, len: usize) -> Result<String, OblivionException> {
        let mut recv_bytes: Vec<u8> = vec![0; len];
        match self.tcp.read_exact(&mut recv_bytes).await {
            Ok(_) => {}
            Err(_) => return Err(OblivionException::UnexpectedDisconnection),
        };

        match String::from_utf8(recv_bytes) {
            Ok(len_int) => Ok(len_int.trim().to_string()),
            Err(_) => Err(OblivionException::BadBytes),
        }
    }

    pub async fn send(&mut self, data: &[u8]) -> Result<(), OblivionException> {
        match self.tcp.write_all(&data).await {
            Ok(_) => match self.tcp.flush().await {
                Ok(_) => Ok(()),
                Err(e) => Err(OblivionException::TCPWriteFailed {
                    message: e.to_string(),
                }),
            },
            Err(e) => Err(OblivionException::TCPWriteFailed {
                message: e.to_string(),
            }),
        }
    }

    pub async fn close(&mut self) -> Result<(), OblivionException> {
        match self.tcp.shutdown().await {
            Ok(_) => Ok(()),
            Err(_) => Err(OblivionException::UnexpectedDisconnection),
        }
    }
}
