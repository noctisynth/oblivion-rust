//! Oblivion Abstract Gear
use crate::exceptions::OblivionException;

use anyhow::Result;
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
pub struct AbsoluteNonceSequence<'a> {
    nonce: &'a [u8],
}

impl<'a> NonceSequence for AbsoluteNonceSequence<'a> {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        Nonce::try_assume_unique_for_key(self.nonce)
    }
}

impl<'a> AbsoluteNonceSequence<'a> {
    pub fn new(nonce: &'a [u8]) -> Self {
        Self { nonce }
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

    pub fn set_ttl(&mut self, ttl: u32) -> Result<()> {
        self.tcp.set_ttl(ttl)?;
        Ok(())
    }

    pub async fn recv_usize(&mut self) -> Result<usize> {
        let mut len_bytes = [0; 4];
        self.tcp.read_exact(&mut len_bytes).await?;

        Ok(u32::from_be_bytes(len_bytes) as usize)
    }

    pub async fn recv_u32(&mut self) -> Result<u32> {
        let mut len_bytes = [0; 4];

        self.tcp.read_exact(&mut len_bytes).await?;

        Ok(u32::from_be_bytes(len_bytes))
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
