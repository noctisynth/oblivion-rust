//! # Oblivion Packets Encapsulation
use crate::utils::gear::Socket;

use crate::exceptions::OblivionException;

use super::super::utils::decryptor::decrypt_bytes;
use super::super::utils::encryptor::{encrypt_bytes, encrypt_plaintext};
use super::super::utils::generator::{generate_random_salt, generate_shared_key};
use super::super::utils::parser::length;

use p256::ecdh::EphemeralSecret;
use p256::PublicKey;
use rand::Rng;
use serde_json::Value;

pub struct ACK {
    sequence: String,
}

impl ACK {
    pub fn new() -> Result<Self, OblivionException> {
        let mut rng = rand::thread_rng();
        let random_number: u16 = rng.gen_range(1000..=9999);
        Ok(Self {
            sequence: random_number.to_string(),
        })
    }

    pub fn equal_bytes(&mut self, __value: &[u8]) -> bool {
        __value == self.sequence.clone().into_bytes()
    }

    pub async fn from_stream(&mut self, stream: &mut Socket) -> Result<Self, OblivionException> {
        Ok(Self {
            sequence: stream.recv_str(4).await?,
        })
    }

    pub async fn to_stream(&mut self, stream: &mut Socket) -> Result<(), OblivionException> {
        stream.send(&self.plain_data()).await?;
        Ok(())
    }

    pub fn plain_data(&mut self) -> Vec<u8> {
        self.sequence.clone().into_bytes()
    }
}

pub struct OSC {
    pub status_code: i32,
}

impl OSC {
    pub fn from_int(status_code: i32) -> Result<Self, OblivionException> {
        Ok(Self {
            status_code: status_code,
        })
    }

    pub async fn from_stream(stream: &mut Socket) -> Result<Self, OblivionException> {
        let status_code = stream.recv_int(3).await?;
        Ok(Self {
            status_code: status_code,
        })
    }

    pub async fn to_stream(&mut self, stream: &mut Socket) -> Result<(), OblivionException> {
        stream.send(&self.plain_data()).await?;
        Ok(())
    }

    pub fn plain_data(&mut self) -> Vec<u8> {
        let status_code = format!("{}", self.status_code);
        status_code.into_bytes()
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

    pub fn from_public_key_bytes(
        &mut self,
        public_key_bytes: &[u8],
    ) -> Result<&mut Self, OblivionException> {
        self.public_key = Some(PublicKey::from_sec1_bytes(public_key_bytes).unwrap());
        Ok(self)
    }

    pub async fn from_stream(
        &mut self,
        stream: &mut Socket,
    ) -> Result<&mut Self, OblivionException> {
        let remote_public_key_length = stream.recv_len().await?;
        let remote_public_key_bytes = stream.recv(remote_public_key_length).await?;
        self.remote_public_key =
            Some(PublicKey::from_sec1_bytes(&remote_public_key_bytes).unwrap());
        self.shared_aes_key = Some(generate_shared_key(
            self.private_key.as_ref().unwrap(),
            self.remote_public_key.as_ref().unwrap(),
            &self.salt.as_mut().unwrap(),
        )?);
        Ok(self)
    }

    pub async fn from_stream_with_salt(
        &mut self,
        stream: &mut Socket,
    ) -> Result<&mut Self, OblivionException> {
        let remote_public_key_length = stream.recv_len().await?;
        let remote_public_key_bytes = stream.recv(remote_public_key_length).await?;
        self.remote_public_key =
            Some(PublicKey::from_sec1_bytes(&remote_public_key_bytes).unwrap());
        let salt_length = stream.recv_len().await?;
        self.salt = Some(stream.recv(salt_length).await?);
        self.shared_aes_key = Some(generate_shared_key(
            self.private_key.as_ref().unwrap(),
            self.remote_public_key.as_ref().unwrap(),
            &self.salt.as_mut().unwrap(),
        )?);
        Ok(self)
    }

    pub async fn to_stream(&mut self, stream: &mut Socket) -> Result<(), OblivionException> {
        stream.send(&self.plain_data()).await?;
        Ok(())
    }

    pub async fn to_stream_with_salt(
        &mut self,
        stream: &mut Socket,
    ) -> Result<(), OblivionException> {
        stream.send(&self.plain_data()).await?;
        stream.send(&self.plain_salt()).await?;
        Ok(())
    }

    pub fn plain_data(&mut self) -> Vec<u8> {
        let public_key_bytes = self.public_key.unwrap().to_sec1_bytes().to_vec();
        let mut plain_data_bytes = length(&public_key_bytes).unwrap();
        plain_data_bytes.extend(public_key_bytes);
        plain_data_bytes
    }

    pub fn plain_salt(&mut self) -> Vec<u8> {
        let salt_bytes = self.salt.as_ref().unwrap();
        let mut plain_salt_bytes = length(&salt_bytes).unwrap();
        plain_salt_bytes.extend(salt_bytes);
        plain_salt_bytes
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
    chunk_size: i32,
}

impl OED {
    pub fn new(aes_key: Option<Vec<u8>>) -> Self {
        Self {
            aes_key: aes_key,
            data: None,
            encrypted_data: None,
            tag: None,
            nonce: None,
            chunk_size: 0,
        }
    }

    fn serialize_bytes(&self, data: &[u8], size: Option<usize>) -> Vec<Vec<u8>> {
        let size = if size.is_none() {
            let size: usize = 1024;
            size
        } else {
            let size = size.unwrap();
            size
        };

        let mut serialized_bytes = Vec::new();
        let data_size = data.len();

        for i in (0..data_size).step_by(size) {
            let buffer = &data[i..std::cmp::min(i + size, data_size)];
            let buffer_length = length(&buffer.to_vec()).unwrap();
            let mut serialized_chunk = Vec::with_capacity(buffer_length.len() + buffer.len());

            if i + size > data_size {
                serialized_chunk.extend_from_slice(&buffer_length);
                serialized_chunk.extend_from_slice(buffer);
                serialized_bytes.push(serialized_chunk);
                break;
            };

            serialized_chunk.extend_from_slice(&buffer_length);
            serialized_chunk.extend_from_slice(buffer);

            serialized_bytes.push(serialized_chunk);
        }

        serialized_bytes.push(b"0000".to_vec());
        serialized_bytes
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
        total_attemps: i32,
    ) -> Result<&mut Self, OblivionException> {
        let mut attemp = 0;
        let mut ack = false;

        while attemp < total_attemps {
            let mut ack_packet = ACK::new()?;
            let mut ack_packet = ack_packet.from_stream(stream).await?;

            let len_nonce = stream.recv_len().await?;
            let len_tag = stream.recv_len().await?;

            self.nonce = Some(stream.recv(len_nonce).await?);
            self.tag = Some(stream.recv(len_tag).await?);

            let mut encrypted_data: Vec<u8> = Vec::new();
            self.chunk_size = 0;

            loop {
                let prefix = stream.recv_len().await?;
                if prefix == 0 {
                    self.encrypted_data = Some(encrypted_data);
                    break;
                }

                let mut add: Vec<u8> = Vec::new();
                while add.len() != prefix {
                    add.extend(stream.recv(prefix - add.len()).await?)
                }

                encrypted_data.extend(add);
                self.chunk_size += 1;
            }

            match decrypt_bytes(
                self.encrypted_data.clone().unwrap(),
                self.tag.as_ref().unwrap(),
                self.aes_key.as_ref().unwrap(),
                self.nonce.as_ref().unwrap(),
            ) {
                Ok(data) => {
                    self.data = Some(data);
                    stream.send(&ack_packet.plain_data()).await?;
                    ack = true;
                    break;
                }
                Err(error) => {
                    stream.send(b"0000").await?;
                    println!("An error occured: {error}\nRetried {attemp} times.");
                    attemp += 1;
                    continue;
                }
            }
        }
        if !ack {
            stream.close().await?;
            return Err(OblivionException::AllAttemptsRetryFailed {
                times: total_attemps,
            });
        }

        Ok(self)
    }

    pub async fn to_stream(
        &mut self,
        stream: &mut Socket,
        total_attemps: i32,
    ) -> Result<(), OblivionException> {
        let attemp = 0;
        let mut ack = false;

        while attemp <= total_attemps {
            let mut ack_packet = ACK::new()?;
            ack_packet.to_stream(stream).await?;

            stream.send(&self.plain_data()).await?;

            self.chunk_size = 0;
            for bytes in self
                .serialize_bytes(&self.encrypted_data.as_ref().unwrap(), None)
                .iter()
            {
                stream.send(&bytes).await?;
                self.chunk_size += 1;
            }

            if ack_packet.sequence.as_bytes() == stream.recv(4).await? {
                ack = true;
                break;
            }
        }

        if !ack {
            stream.close().await?;
            return Err(OblivionException::AllAttemptsRetryFailed {
                times: total_attemps,
            });
        }

        Ok(())
    }

    pub fn plain_data(&mut self) -> Vec<u8> {
        let nonce_bytes = self.nonce.as_ref().unwrap();
        let tag_bytes = self.tag.as_ref().unwrap();

        let mut plain_bytes = length(nonce_bytes).unwrap();
        plain_bytes.extend(length(tag_bytes).unwrap());
        plain_bytes.extend(nonce_bytes);
        plain_bytes.extend(tag_bytes);

        plain_bytes
    }

    pub fn get_data(&mut self) -> Vec<u8> {
        self.data.clone().unwrap()
    }
}
