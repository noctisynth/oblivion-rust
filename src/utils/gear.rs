use ring::{
    aead::{Nonce, NonceSequence},
    error::Unspecified,
};
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct RandNonceSequence {
    nonce: Vec<u8>,
}

impl NonceSequence for RandNonceSequence {
    // called once for each seal operation
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

    pub fn recv_len(&mut self) -> usize {
        let mut len_bytes: Vec<u8> = vec![0; 4];
        let _ = self.tcp.read_exact(&mut len_bytes); // 捕获RSA_KEY长度

        let len_int: i32 = std::str::from_utf8(&len_bytes)
            .unwrap()
            .parse()
            .expect("Failed to receieve length");

        let len: usize = len_int
            .try_into()
            .expect("Failed to generate unsize value");
        len
    }

    pub fn recv(&mut self, len: usize) -> Vec<u8> {
        let mut recv_bytes: Vec<u8> = vec![0; len];
        let _ = self.tcp
            .read_exact(&mut recv_bytes)
            .expect("Failed to recv RSA_KEY");
        recv_bytes
    }

    pub fn recv_str(&mut self, len: usize) -> String {
        let mut recv_bytes: Vec<u8> = vec![0; len];
        let _ = self.tcp
            .read_exact(&mut recv_bytes)
            .expect("Failed to recv RSA_KEY");

        let recv_str = String::from_utf8(recv_bytes.clone())
            .unwrap()
            .trim()
            .to_string();
        recv_str
    }

    pub fn send(&mut self, data: &[u8]) {
        let _ = self.tcp.write(data);
    }
}
