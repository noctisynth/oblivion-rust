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
        match self.tcp.read_exact(&mut len_bytes).await {
            Ok(_) => {}
            Err(_) => return Err(OblivionException::UnexpectedDisconnection),
        };

        let len_int: i32 = match std::str::from_utf8(&len_bytes) {
            Ok(len_int) => len_int,
            Err(_) => return Err(OblivionException::BadBytes),
        }
        .parse()
        .expect("Failed to receieve length");

        let len: usize = len_int.try_into().expect("Failed to generate unsize value");
        Ok(len)
    }

    pub async fn recv_int(&mut self, len: usize) -> Result<i32, OblivionException> {
        let mut len_bytes: Vec<u8> = vec![0; len];
        match self.tcp.read_exact(&mut len_bytes).await {
            Ok(_) => {}
            Err(_) => return Err(OblivionException::UnexpectedDisconnection),
        };

        let int: i32 = match std::str::from_utf8(&len_bytes) {
            Ok(len_int) => len_int,
            Err(_) => return Err(OblivionException::BadBytes),
        }
        .parse()
        .expect("Failed to receieve length");

        Ok(int)
    }

    pub async fn recv(&mut self, len: usize) -> Result<Vec<u8>, OblivionException> {
        let mut recv_bytes: Vec<u8> = vec![0; len];
        match self.tcp.read_exact(&mut recv_bytes).await {
            Ok(_) => {}
            Err(_) => return Err(OblivionException::UnexpectedDisconnection),
        };
        Ok(recv_bytes)
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
        match self.tcp.write(data).await {
            Ok(_) => Ok(()),
            Err(_) => Err(OblivionException::UnexpectedDisconnection),
        }
    }

    pub async fn close(&mut self) -> Result<(), OblivionException> {
        match self.tcp.shutdown().await {
            Ok(_) => Ok(()),
            Err(_) => Err(OblivionException::UnexpectedDisconnection),
        }
    }
}
