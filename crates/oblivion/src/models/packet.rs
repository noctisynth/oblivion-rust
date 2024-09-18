//! # Oblivion Packets Encapsulation
use crate::exceptions::Exception;
use crate::utils::decryptor::decrypt_bytes;
use crate::utils::encryptor::{encrypt_bytes, encrypt_plaintext};
use crate::utils::gear::Socket;
use crate::utils::generator::{generate_random_salt, SharedKey};
use crate::utils::parser::length;

use anyhow::Result;
use serde_json::Value;

use ring::agreement::{EphemeralPrivateKey, UnparsedPublicKey, X25519};

const STOP_FLAG: [u8; 4] = u32::MIN.to_be_bytes();

pub struct OSC {
    pub status_code: u32,
}

impl OSC {
    pub fn from_u32(status_code: u32) -> Self {
        Self { status_code }
    }

    pub async fn from_stream(stream: &Socket) -> Result<Self> {
        Ok(Self {
            status_code: stream.recv_u32().await?,
        })
    }

    pub async fn to_stream(&self, stream: &Socket) -> Result<()> {
        stream.send(&self.status_code.to_be_bytes()).await?;
        Ok(())
    }
}

pub struct OKE {
    public_key: UnparsedPublicKey<Vec<u8>>,
    private_key: Option<EphemeralPrivateKey>,
    salt: Vec<u8>,
    remote_public_key: Option<UnparsedPublicKey<Vec<u8>>>,
    shared_aes_key: Option<[u8; 16]>,
}

impl OKE {
    pub fn new(
        private_key: Option<EphemeralPrivateKey>,
        public_key: UnparsedPublicKey<Vec<u8>>,
    ) -> Self {
        Self {
            public_key,
            private_key,
            salt: generate_random_salt(),
            remote_public_key: None,
            shared_aes_key: None,
        }
    }

    pub fn from_public_key_bytes(&mut self, public_key_bytes: &[u8]) -> Result<&mut Self> {
        self.public_key = UnparsedPublicKey::new(&X25519, public_key_bytes.to_owned());
        Ok(self)
    }

    pub async fn from_stream(&mut self, stream: &Socket) -> Result<&mut Self> {
        let remote_public_key_length = stream.recv_usize().await?;
        let remote_public_key_bytes = stream.recv(remote_public_key_length).await?;
        self.remote_public_key = Some(UnparsedPublicKey::new(&X25519, remote_public_key_bytes));
        let mut shared_key = SharedKey::new(
            self.private_key.take().unwrap(),
            self.remote_public_key.as_ref().unwrap(),
        )?;
        self.shared_aes_key = Some(shared_key.hkdf(&self.salt));
        Ok(self)
    }

    pub async fn from_stream_with_salt(&mut self, stream: &Socket) -> Result<&mut Self> {
        let remote_public_key_length = stream.recv_usize().await?;
        let remote_public_key_bytes = stream.recv(remote_public_key_length).await?;
        self.remote_public_key = Some(UnparsedPublicKey::new(&X25519, remote_public_key_bytes));
        let salt_length = stream.recv_usize().await?;
        self.salt = stream.recv(salt_length).await?;
        let mut shared_key = SharedKey::new(
            self.private_key.take().unwrap(),
            self.remote_public_key.as_ref().unwrap(),
        )?;
        self.shared_aes_key = Some(shared_key.hkdf(&self.salt));
        Ok(self)
    }

    pub async fn to_stream(&self, stream: &Socket) -> Result<()> {
        stream.send(&self.plain_data()?).await?;
        Ok(())
    }

    pub async fn to_stream_with_salt(&self, stream: &Socket) -> Result<()> {
        stream.send(&self.plain_data()?).await?;
        stream.send(&self.plain_salt()?).await?;
        Ok(())
    }

    pub fn plain_data(&self) -> Result<Vec<u8>> {
        let public_key_bytes = self.public_key.as_ref();
        let mut plain_data_bytes = length(public_key_bytes)?.to_vec();
        plain_data_bytes.extend_from_slice(public_key_bytes);
        Ok(plain_data_bytes)
    }

    pub fn plain_salt(&self) -> Result<Vec<u8>> {
        let mut plain_salt_bytes = length(&self.salt)?.to_vec();
        plain_salt_bytes.extend_from_slice(&self.salt);
        Ok(plain_salt_bytes)
    }

    pub fn get_aes_key(&self) -> [u8; 16] {
        self.shared_aes_key.unwrap()
    }
}

pub struct OED<'a> {
    aes_key: &'a [u8],
    data: Option<Vec<u8>>,
    encrypted_data: Vec<u8>,
    tag: Vec<u8>,
    nonce: Vec<u8>,
    chunk_count: u32,
}

impl<'a> OED<'a> {
    pub fn new(aes_key: &'a [u8]) -> Self {
        Self {
            aes_key,
            data: None,
            encrypted_data: Vec::new(),
            tag: Vec::new(),
            nonce: Vec::new(),
            chunk_count: 0,
        }
    }

    pub fn from_json_or_string(&mut self, json_or_str: String) -> Result<&mut Self, Exception> {
        (self.encrypted_data, self.tag, self.nonce) = encrypt_plaintext(json_or_str, self.aes_key)?;
        Ok(self)
    }

    pub fn from_dict(&mut self, dict: Value) -> Result<&mut Self, Exception> {
        (self.encrypted_data, self.tag, self.nonce) =
            encrypt_plaintext(dict.to_string(), self.aes_key)?;
        Ok(self)
    }

    pub fn from_encrypted_data(&mut self, data: Vec<u8>) -> &mut Self {
        self.encrypted_data = data;
        self
    }

    pub fn from_bytes(&mut self, data: Vec<u8>) -> Result<&mut Self, Exception> {
        (self.encrypted_data, self.tag, self.nonce) = encrypt_bytes(data, self.aes_key)?;
        Ok(self)
    }

    pub async fn from_stream(&mut self, stream: &Socket) -> Result<&mut Self> {
        let len_nonce = stream.recv_usize().await?;
        let len_tag = stream.recv_usize().await?;

        self.nonce = stream.recv(len_nonce).await?;
        self.tag = stream.recv(len_tag).await?;

        let mut encrypted_data: Vec<u8> = Vec::new();
        self.chunk_count = 0;

        loop {
            let prefix = stream.recv_usize().await?;
            if prefix == 0 {
                self.encrypted_data = encrypted_data;
                break;
            }

            let mut add: Vec<u8> = Vec::new();
            while add.len() != prefix {
                add.extend(stream.recv(prefix - add.len()).await?)
            }

            encrypted_data.extend(add);
            self.chunk_count += 1;
        }

        match decrypt_bytes(
            self.encrypted_data.clone(),
            &self.tag,
            self.aes_key,
            &self.nonce,
        ) {
            Ok(data) => {
                self.data = Some(data);
                Ok(self)
            }
            Err(error) => Err(Exception::DecryptError { error }.into()),
        }
    }

    pub async fn to_stream(&mut self, stream: &Socket) -> Result<()> {
        stream.send(&self.plain_data()?).await?;

        self.chunk_count = 0;
        let chunks = self.encrypted_data.chunks(1024);
        for chunk in chunks {
            let chunk_size = chunk.len() as u32;
            stream.send(&chunk_size.to_be_bytes()).await?;
            stream.send(chunk).await?;
            self.chunk_count += 1;
        }
        stream.send(&STOP_FLAG).await?;

        Ok(())
    }

    pub fn plain_data(&self) -> Result<Vec<u8>> {
        let mut plain_bytes = length(&self.nonce)?.to_vec();
        plain_bytes.extend_from_slice(&length(&self.tag)?);
        plain_bytes.extend_from_slice(&self.nonce);
        plain_bytes.extend_from_slice(&self.tag);

        Ok(plain_bytes)
    }

    pub fn take(&mut self) -> Vec<u8> {
        self.data.take().unwrap()
    }

    pub fn get_data(&self) -> &[u8] {
        self.data.as_ref().unwrap()
    }
}
