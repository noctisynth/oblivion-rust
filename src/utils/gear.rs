use crate::exceptions::OblivionException;
use ring::{
    aead::{Nonce, NonceSequence},
    error::Unspecified,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct RandNonceSequence {
    nonce: Vec<u8>,
}

impl NonceSequence for RandNonceSequence {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        Nonce::try_assume_unique_for_key(&self.nonce)
    }
}

impl RandNonceSequence {
    pub fn new(nonce: Vec<u8>) -> Self {
        Self { nonce: nonce }
    }
}

pub struct Socket {
    tcp: TcpStream,
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
        let _ = self.tcp.read_exact(&mut len_bytes).await.unwrap();

        let len_int: i32 = std::str::from_utf8(&len_bytes)
            .unwrap()
            .parse()
            .expect("Failed to receieve length");

        let len: usize = len_int.try_into().expect("Failed to generate unsize value");
        Ok(len)
    }

    pub async fn recv_int(&mut self, len: usize) -> Result<i32, OblivionException> {
        let mut len_bytes: Vec<u8> = vec![0; len];
        let _ = self.tcp.read_exact(&mut len_bytes).await.unwrap();

        let int: i32 = std::str::from_utf8(&len_bytes)
            .unwrap()
            .parse()
            .expect("Failed to receieve length");

        Ok(int)
    }

    pub async fn recv(&mut self, len: usize) -> Vec<u8> {
        let mut recv_bytes: Vec<u8> = vec![0; len];
        let _ = self.tcp.read_exact(&mut recv_bytes).await.unwrap();
        recv_bytes
    }

    pub async fn recv_str(&mut self, len: usize) -> Result<String, OblivionException> {
        let mut recv_bytes: Vec<u8> = vec![0; len];
        let _ = self.tcp.read_exact(&mut recv_bytes).await.unwrap();

        let recv_str = String::from_utf8(recv_bytes.clone())
            .unwrap()
            .trim()
            .to_string();
        Ok(recv_str)
    }

    pub async fn send(&mut self, data: &[u8]) {
        let _ = self.tcp.write(data).await.unwrap();
    }

    pub async fn close(&mut self) {
        let _ = self.tcp.shutdown().await.unwrap();
    }
}
