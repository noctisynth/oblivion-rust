//! Oblivion Abstract Gear
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
#[derive(Debug)]
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

    pub async fn recv(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut recv_bytes: Vec<u8> = vec![0; len];
        self.tcp.read_exact(&mut recv_bytes).await?;
        Ok(recv_bytes)
    }

    pub async fn recv_str(&mut self, len: usize) -> Result<String> {
        let mut recv_bytes: Vec<u8> = vec![0; len];
        self.tcp.read_exact(&mut recv_bytes).await?;

        Ok(String::from_utf8(recv_bytes)?.trim().to_string())
    }

    pub async fn send(&mut self, data: &[u8]) -> Result<()> {
        self.tcp.write_all(&data).await?;
        self.tcp.flush().await?;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.tcp.shutdown().await?;
        Ok(())
    }
}
