//! Oblivion Abstract Gear
use std::net::SocketAddr;

use anyhow::Result;
use ring::aead::{Nonce, NonceSequence};
use ring::error::Unspecified;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

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
    pub reader: Mutex<OwnedReadHalf>,
    pub writer: Mutex<OwnedWriteHalf>,
}

impl Socket {
    pub fn new(tcp: TcpStream) -> Self {
        let (reader, writer) = tcp.into_split();
        Self {
            reader: Mutex::new(reader),
            writer: Mutex::new(writer),
        }
    }

    pub async fn peer_addr(&self) -> Result<SocketAddr> {
        Ok(self.writer.lock().await.peer_addr()?)
    }

    pub async fn recv_usize(&self) -> Result<usize> {
        let mut len_bytes = [0; 4];
        #[cfg(not(feature = "perf"))]
        self.reader.lock().await.read_exact(&mut len_bytes).await?;
        #[cfg(feature = "perf")]
        #[cfg(feature = "perf")]
        {
            use colored::Colorize;
            let now = tokio::time::Instant::now();
            let mut reader = self.reader.lock().await;
            println!(
                "夺锁时长: {}μs",
                now.elapsed().as_micros().to_string().bright_magenta()
            );
            reader.read_exact(&mut len_bytes).await?;
        }
        Ok(u32::from_be_bytes(len_bytes) as usize)
    }

    pub async fn recv_u32(&self) -> Result<u32> {
        let mut len_bytes = [0; 4];
        self.reader.lock().await.read_exact(&mut len_bytes).await?;
        Ok(u32::from_be_bytes(len_bytes))
    }

    pub async fn recv(&self, len: usize) -> Result<Vec<u8>> {
        let mut recv_bytes: Vec<u8> = vec![0; len];
        self.reader.lock().await.read_exact(&mut recv_bytes).await?;
        Ok(recv_bytes)
    }

    pub async fn recv_str(&self, len: usize) -> Result<String> {
        let mut recv_bytes: Vec<u8> = vec![0; len];
        self.reader.lock().await.read_exact(&mut recv_bytes).await?;
        Ok(String::from_utf8(recv_bytes)?)
    }

    pub async fn send(&self, data: &[u8]) -> Result<()> {
        let mut writer = self.writer.lock().await;
        writer.write_all(&data).await?;
        writer.flush().await?;
        Ok(())
    }

    pub async fn close(&self) -> Result<()> {
        self.writer.lock().await.shutdown().await?;
        Ok(())
    }
}
