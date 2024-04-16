//! # Oblivion Packets Encapsulation
use crate::exceptions::OblivionException;
use crate::utils::decryptor::decrypt_bytes;
use crate::utils::encryptor::{encrypt_bytes, encrypt_plaintext};
use crate::utils::gear::Socket;
use crate::utils::generator::{generate_random_salt, generate_shared_key};
use crate::utils::parser::length;

use anyhow::{Error, Result};
use p256::ecdh::EphemeralSecret;
use p256::PublicKey;
use rand::Rng;
use serde_json::Value;

const STOP_FLAG: [u8; 4] = u32::MIN.to_be_bytes();

pub struct ACK {
    sequence: u32,
}

impl ACK {
    pub fn new() -> Self {
        Self {
            sequence: rand::thread_rng().gen_range(1000..=9999),
        }
    }

    pub async fn from_stream(&mut self, stream: &mut Socket) -> Result<Self> {
        Ok(Self {
            sequence: stream.recv_u32().await?,
        })
    }

    pub async fn to_stream(&mut self, stream: &mut Socket) -> Result<()> {
        stream.send(&self.plain_data()).await?;
        Ok(())
    }

    pub fn plain_data(&mut self) -> [u8; 4] {
        self.sequence.to_be_bytes()
    }
}

pub struct OSC {
    pub status_code: u32,
}

impl OSC {
    pub fn from_u32(status_code: u32) -> Self {
        Self { status_code }
    }

    pub async fn from_stream(stream: &mut Socket) -> Result<Self> {
        let status_code = stream.recv_u32().await?;
        Ok(Self { status_code })
    }

    pub async fn to_stream(&mut self, stream: &mut Socket) -> Result<()> {
        stream.send(&self.plain_data()).await?;
        Ok(())
    }

    pub fn plain_data(&mut self) -> [u8; 4] {
        let status_code = self.status_code as u32;
        status_code.to_be_bytes()
    }
}

pub struct OKE<'a> {
    public_key: Option<PublicKey>,
    private_key: Option<&'a EphemeralSecret>,
    salt: Option<Vec<u8>>,
    remote_public_key: Option<PublicKey>,
    shared_aes_key: Option<Vec<u8>>,
}

impl<'a> OKE<'a> {
    pub fn new(
        private_key: Option<&'a EphemeralSecret>,
        public_key: Option<PublicKey>,
    ) -> Result<Self, OblivionException> {
        Ok(Self {
            public_key,
            private_key,
            salt: Some(generate_random_salt()),
            remote_public_key: None,
            shared_aes_key: None,
        })
    }

    pub fn from_public_key_bytes(&mut self, public_key_bytes: &[u8]) -> Result<&mut Self> {
        self.public_key = Some(PublicKey::from_sec1_bytes(public_key_bytes)?);
        Ok(self)
    }

    pub async fn from_stream(&mut self, stream: &mut Socket) -> Result<&mut Self> {
        let remote_public_key_length = stream.recv_usize().await?;
        let remote_public_key_bytes = stream.recv(remote_public_key_length).await?;
        self.remote_public_key = Some(PublicKey::from_sec1_bytes(&remote_public_key_bytes)?);
        self.shared_aes_key = Some(generate_shared_key(
            self.private_key.as_ref().unwrap(),
            self.remote_public_key.as_ref().unwrap(),
            &self.salt.as_mut().unwrap(),
        )?);
        Ok(self)
    }

    pub async fn from_stream_with_salt(&mut self, stream: &mut Socket) -> Result<&mut Self> {
        let remote_public_key_length = stream.recv_usize().await?;
        let remote_public_key_bytes = stream.recv(remote_public_key_length).await?;
        self.remote_public_key = Some(PublicKey::from_sec1_bytes(&remote_public_key_bytes)?);
        let salt_length = stream.recv_usize().await?;
        self.salt = Some(stream.recv(salt_length).await?);
        self.shared_aes_key = Some(generate_shared_key(
            self.private_key.unwrap(),
            &self.remote_public_key.unwrap(),
            self.salt.as_mut().unwrap(),
        )?);
        Ok(self)
    }

    pub async fn to_stream(&mut self, stream: &mut Socket) -> Result<()> {
        stream.send(&self.plain_data()?).await?;
        Ok(())
    }

    pub async fn to_stream_with_salt(&mut self, stream: &mut Socket) -> Result<()> {
        stream.send(&self.plain_data()?).await?;
        stream.send(&self.plain_salt()?).await?;
        Ok(())
    }

    pub fn plain_data(&mut self) -> Result<Vec<u8>> {
        let public_key_bytes = self.public_key.unwrap().to_sec1_bytes().to_vec();
        let mut plain_data_bytes = length(&public_key_bytes)?.to_vec();
        plain_data_bytes.extend(public_key_bytes);
        Ok(plain_data_bytes)
    }

    pub fn plain_salt(&mut self) -> Result<Vec<u8>> {
        let salt_bytes = self.salt.as_ref().unwrap();
        let mut plain_salt_bytes = length(&salt_bytes)?.to_vec();
        plain_salt_bytes.extend(salt_bytes);
        Ok(plain_salt_bytes)
    }

    pub fn get_aes_key(&mut self) -> Vec<u8> {
        self.shared_aes_key.clone().unwrap()
    }
}

pub struct OED {
    aes_key: Option<Vec<u8>>,
    data: Option<Vec<u8>>,
    encrypted_data: Option<Vec<u8>>,
    tag: Option<Vec<u8>>,
    nonce: Option<Vec<u8>>,
    chunk_count: u32,
}

impl OED {
    pub fn new(aes_key: Option<Vec<u8>>) -> Self {
        Self {
            aes_key,
            data: None,
            encrypted_data: None,
            tag: None,
            nonce: None,
            chunk_count: 0,
        }
    }

    pub fn from_json_or_string(
        &mut self,
        json_or_str: String,
    ) -> Result<&mut Self, OblivionException> {
        let (encrypted_data, tag, nonce) =
            encrypt_plaintext(json_or_str, &self.aes_key.as_ref().unwrap())?;
        (self.encrypted_data, self.tag, self.nonce) =
            (Some(encrypted_data), Some(tag), Some(nonce));
        Ok(self)
    }

    pub fn from_dict(&mut self, dict: Value) -> Result<&mut Self, OblivionException> {
        let (encrypted_data, tag, nonce) =
            encrypt_plaintext(dict.to_string(), &self.aes_key.as_ref().unwrap())?;
        (self.encrypted_data, self.tag, self.nonce) =
            (Some(encrypted_data), Some(tag), Some(nonce));
        Ok(self)
    }

    pub fn from_encrypted_data(&mut self, data: Vec<u8>) -> Result<&mut Self, ()> {
        self.encrypted_data = Some(data);
        Ok(self)
    }

    pub fn from_bytes(&mut self, data: Vec<u8>) -> Result<&mut Self, OblivionException> {
        let (encrypted_data, tag, nonce) = encrypt_bytes(data, &self.aes_key.as_ref().unwrap())?;
        (self.encrypted_data, self.tag, self.nonce) =
            (Some(encrypted_data), Some(tag), Some(nonce));
        Ok(self)
    }

    pub async fn from_stream(
        &mut self,
        stream: &mut Socket,
        total_attemps: u32,
    ) -> Result<&mut Self> {
        let mut attemp = 0;
        let mut ack = false;

        while attemp < total_attemps {
            let mut ack_packet = ACK::new();
            let mut ack_packet = ack_packet.from_stream(stream).await?;

            let len_nonce = stream.recv_usize().await?;
            let len_tag = stream.recv_usize().await?;

            self.nonce = Some(stream.recv(len_nonce).await?);
            self.tag = Some(stream.recv(len_tag).await?);

            let mut encrypted_data: Vec<u8> = Vec::new();
            self.chunk_count = 0;

            loop {
                let prefix = stream.recv_usize().await?;
                if prefix == 0 {
                    self.encrypted_data = Some(encrypted_data);
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
                self.encrypted_data.clone().unwrap(),
                self.tag.as_ref().unwrap(),
                self.aes_key.as_ref().unwrap(),
                self.nonce.as_ref().unwrap(),
            ) {
                Ok(data) => {
                    self.data = Some(data);
                    ack_packet.to_stream(stream).await?;
                    ack = true;
                    break;
                }
                Err(error) => {
                    stream.send(&STOP_FLAG).await?;
                    eprintln!("An error occured: {error}\nRetried {attemp} times.");
                    attemp += 1;
                    continue;
                }
            }
        }
        if !ack {
            stream.close().await?;
            return Err(Error::from(OblivionException::AllAttemptsRetryFailed {
                times: total_attemps,
            }));
        }

        Ok(self)
    }

    pub async fn to_stream(&mut self, stream: &mut Socket, total_attemps: u32) -> Result<()> {
        let attemp = 0;
        let mut ack = false;

        while attemp <= total_attemps {
            let mut ack_packet = ACK::new();
            ack_packet.to_stream(stream).await?;

            stream.send(&self.plain_data()?).await?;

            self.chunk_count = 0;
            let encrypted_data = self.encrypted_data.as_ref().unwrap();
            let mut remaining_data = &encrypted_data[..];
            while !remaining_data.is_empty() {
                let chunk_size = remaining_data.len().min(2048);

                let chunk_length = chunk_size as u32;

                stream.send(&chunk_length.to_be_bytes()).await?;
                stream.send(&remaining_data[..chunk_size]).await?;

                remaining_data = &remaining_data[chunk_size..];
            }
            stream.send(&STOP_FLAG).await?;

            if ack_packet.sequence == stream.recv_u32().await? {
                ack = true;
                break;
            }
        }

        if !ack {
            stream.close().await?;
            return Err(Error::from(OblivionException::AllAttemptsRetryFailed {
                times: total_attemps,
            }));
        }

        Ok(())
    }

    pub fn plain_data(&mut self) -> Result<Vec<u8>> {
        let nonce_bytes = self.nonce.as_ref().unwrap();
        let tag_bytes = self.tag.as_ref().unwrap();

        let mut plain_bytes = length(nonce_bytes)?.to_vec();
        plain_bytes.extend(length(tag_bytes).unwrap());
        plain_bytes.extend(nonce_bytes);
        plain_bytes.extend(tag_bytes);

        Ok(plain_bytes)
    }

    pub fn get_data(&mut self) -> Vec<u8> {
        self.data.clone().unwrap()
    }
}
