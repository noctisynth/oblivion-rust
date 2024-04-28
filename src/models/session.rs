use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use serde_json::Value;
use tokio::sync::RwLock;

#[cfg(feature = "unsafe")]
use p256::{ecdh::EphemeralSecret, PublicKey};
#[cfg(not(feature = "unsafe"))]
use ring::agreement::{EphemeralPrivateKey, PublicKey, UnparsedPublicKey, X25519};

use crate::exceptions::Exception;
use crate::utils::gear::Socket;
use crate::utils::generator::generate_key_pair;
use crate::utils::parser::{length, OblivionRequest};

use super::client::Response;
use super::packet::{OED, OKE, OSC};
use super::render::BaseResponse;

pub struct Session {
    pub header: String,
    #[cfg(feature = "unsafe")]
    pub(crate) private_key: EphemeralSecret,
    #[cfg(feature = "unsafe")]
    pub(crate) public_key: PublicKey,
    #[cfg(not(feature = "unsafe"))]
    pub(crate) private_key: Option<EphemeralPrivateKey>,
    #[cfg(not(feature = "unsafe"))]
    pub(crate) public_key: PublicKey,
    pub(crate) aes_key: [u8; 16],
    pub request_time: DateTime<Local>,
    pub request: OblivionRequest,
    pub socket: Arc<Socket>,
    closed: RwLock<bool>,
}

impl Session {
    pub fn new(socket: Socket) -> Result<Self> {
        let (private_key, public_key) = generate_key_pair()?;
        Ok(Self {
            header: String::new(),
            #[cfg(feature = "unsafe")]
            private_key,
            #[cfg(not(feature = "unsafe"))]
            private_key: Some(private_key),
            public_key,
            aes_key: Default::default(),
            request_time: Local::now(),
            request: Default::default(),
            socket: Arc::new(socket),
            closed: RwLock::new(false),
        })
    }

    pub fn new_with_header(header: String, socket: Socket) -> Result<Self> {
        let (private_key, public_key) = generate_key_pair()?;
        Ok(Self {
            header,
            #[cfg(feature = "unsafe")]
            private_key,
            #[cfg(not(feature = "unsafe"))]
            private_key: Some(private_key),
            public_key,
            aes_key: Default::default(),
            request_time: Local::now(),
            request: Default::default(),
            socket: Arc::new(socket),
            closed: RwLock::new(false),
        })
    }

    pub async fn first_hand(&mut self) -> Result<()> {
        let socket = Arc::clone(&self.socket);
        let header = self.header.as_bytes();
        #[cfg(feature = "perf")]
        let now = tokio::time::Instant::now();
        socket.send(&length(&header.to_vec())?).await?;
        socket.send(header).await?;
        #[cfg(feature = "perf")]
        println!("发送头时长: {}μs", now.elapsed().as_micros().to_string());

        #[cfg(feature = "unsafe")]
        let mut oke = OKE::new(Some(&self.private_key), Some(self.public_key))?;
        #[cfg(not(feature = "unsafe"))]
        let public_key = UnparsedPublicKey::new(&X25519, self.public_key.as_ref().to_vec());
        #[cfg(not(feature = "unsafe"))]
        let mut oke = OKE::new(self.private_key.take(), public_key);
        oke.from_stream_with_salt(&socket).await?;
        self.aes_key = oke.get_aes_key();
        oke.to_stream(&socket).await?;
        Ok(())
    }

    pub async fn second_hand(&mut self) -> Result<()> {
        #[cfg(feature = "perf")]
        let now = tokio::time::Instant::now();
        #[cfg(feature = "perf")]
        use colored::Colorize;
        let socket = Arc::clone(&self.socket);
        let peer = socket.peer_addr().await?;
        #[cfg(feature = "perf")]
        println!(
            "开始入站时长: {}μs",
            now.elapsed().as_micros().to_string().bright_magenta()
        );
        let len_header = socket.recv_usize().await?;
        #[cfg(feature = "perf")]
        println!(
            "捕获头长度时长: {}μs",
            now.elapsed().as_micros().to_string().bright_magenta()
        );
        let header = socket.recv_str(len_header).await?;
        #[cfg(feature = "perf")]
        println!(
            "入站时长: {}μs",
            now.elapsed().as_micros().to_string().bright_magenta()
        );
        let mut request = OblivionRequest::new(&header)?;
        request.set_remote_peer(&peer);

        #[cfg(feature = "perf")]
        let now = std::time::Instant::now();
        #[cfg(feature = "unsafe")]
        let mut oke = OKE::new(Some(&self.private_key), Some(self.public_key))?;
        #[cfg(not(feature = "unsafe"))]
        let public_key = UnparsedPublicKey::new(&X25519, self.public_key.as_ref().to_vec());
        #[cfg(not(feature = "unsafe"))]
        let mut oke = OKE::new(self.private_key.take(), public_key);
        oke.to_stream_with_salt(&socket).await?;
        oke.from_stream(&socket).await?;
        #[cfg(feature = "perf")]
        println!(
            "密钥交互时长: {}μs",
            now.elapsed().as_micros().to_string().bright_magenta()
        );

        request.aes_key = Some(oke.get_aes_key());
        self.aes_key = oke.get_aes_key();

        self.request = request;
        self.header = header;
        Ok(())
    }

    pub async fn handshake(&mut self, flag: u8) -> Result<()> {
        match flag {
            0 => self.first_hand().await?,
            1 => self.second_hand().await?,
            _ => return Err(anyhow!("Unknown handshake flag")),
        };
        Ok(())
    }

    pub async fn send(&self, data: Vec<u8>, status_code: u32) -> Result<()> {
        if self.closed().await {
            return Err(Exception::ConnectionClosed.into());
        }

        let socket = &self.socket;

        OSC::from_u32(0).to_stream(socket).await?;
        OED::new(&self.aes_key)
            .from_bytes(data)?
            .to_stream(socket)
            .await?;
        OSC::from_u32(status_code).to_stream(socket).await?;
        Ok(())
    }

    pub async fn send_json(&self, json: Value, status_code: u32) -> Result<()> {
        self.send(json.to_string().into_bytes(), status_code).await
    }

    pub async fn response(&self, response: BaseResponse) -> Result<()> {
        self.send(response.as_bytes()?, response.get_status_code()?)
            .await
    }

    pub async fn recv(&self) -> Result<Response> {
        if self.closed().await {
            return Err(Exception::ConnectionClosed.into());
        }

        let socket = &self.socket;

        let flag = OSC::from_stream(socket).await?.status_code;
        let content = OED::new(&self.aes_key)
            .from_stream(socket)
            .await?
            .get_data();
        let status_code = OSC::from_stream(socket).await?.status_code;
        let response = Response::new(None, content, None, status_code, flag);

        if flag == 1 {
            socket.close().await?;
        }
        Ok(response)
    }

    pub async fn close(&self) -> Result<()> {
        if !self.closed().await {
            *self.closed.write().await = true;
            self.socket.close().await
        } else {
            Ok(())
        }
    }

    pub async fn closed(&self) -> bool {
        *self.closed.read().await
    }

    pub fn header(&self) -> &str {
        &self.header
    }

    pub fn get_ip(&self) -> &str {
        self.request.get_ip()
    }
}
