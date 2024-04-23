use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use p256::{ecdh::EphemeralSecret, PublicKey};
use tokio::sync::{Mutex, RwLock};

use crate::exceptions::Exception;
use crate::utils::gear::Socket;
use crate::utils::generator::generate_key_pair;
use crate::utils::parser::{length, OblivionRequest};

use super::client::Response;
use super::packet::{OED, OKE, OSC};

pub struct Session {
    pub header: Option<String>,
    pub(crate) private_key: EphemeralSecret,
    pub(crate) public_key: PublicKey,
    pub(crate) aes_key: Option<Vec<u8>>,
    pub request_time: DateTime<Local>,
    pub request: Option<OblivionRequest>,
    pub socket: Arc<Mutex<Socket>>,
    closed: Arc<RwLock<bool>>,
}

impl Session {
    pub fn new(socket: Socket) -> Result<Self> {
        let (private_key, public_key) = generate_key_pair()?;
        Ok(Self {
            header: None,
            private_key,
            public_key,
            aes_key: None,
            request_time: Local::now(),
            request: None,
            socket: Arc::new(Mutex::new(socket)),
            closed: Arc::new(RwLock::new(false)),
        })
    }

    pub fn new_with_header(header: &str, socket: Socket) -> Result<Self> {
        let (private_key, public_key) = generate_key_pair()?;
        Ok(Self {
            header: Some(header.to_string()),
            private_key,
            public_key,
            aes_key: None,
            request_time: Local::now(),
            request: None,
            socket: Arc::new(Mutex::new(socket)),
            closed: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn first_hand(&mut self) -> Result<()> {
        let socket = &mut self.socket.lock().await;
        let header = self.header.as_ref().unwrap().as_bytes();
        socket
            .send(&[&length(&header.to_vec())?, header].concat())
            .await?;

        let mut oke = OKE::new(Some(&self.private_key), Some(self.public_key))?;
        oke.from_stream_with_salt(socket).await?;
        self.aes_key = Some(oke.get_aes_key());
        oke.to_stream(socket).await?;
        Ok(())
    }

    pub async fn second_hand(&mut self) -> Result<()> {
        let socket = &mut self.socket.lock().await;
        let peer = socket.peer_addr()?;
        let len_header = socket.recv_usize().await?;
        let header = socket.recv_str(len_header).await?;
        let mut request = OblivionRequest::new(&header)?;
        request.set_remote_peer(&peer);

        let mut oke = OKE::new(Some(&self.private_key), Some(self.public_key))?;
        oke.to_stream_with_salt(socket).await?;
        oke.from_stream(socket).await?;

        request.aes_key = Some(oke.get_aes_key());
        self.aes_key = Some(oke.get_aes_key());

        self.request = Some(request);
        self.header = Some(header);
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

        let socket = &mut self.socket.lock().await;

        OSC::from_u32(0).to_stream(socket).await?;
        OED::new(self.aes_key.clone())
            .from_bytes(data)?
            .to_stream(socket, 5)
            .await?;
        OSC::from_u32(status_code).to_stream(socket).await?;
        Ok(())
    }

    pub async fn recv(&self) -> Result<Response> {
        if self.closed().await {
            return Err(Exception::ConnectionClosed.into());
        }

        let socket = &mut self.socket.lock().await;

        let flag = OSC::from_stream(socket).await?.status_code;
        let content = OED::new(self.aes_key.clone())
            .from_stream(socket, 5)
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
            let socket = &mut self.socket.lock().await;
            *self.closed.write().await = true;
            socket.close().await
        } else {
            Ok(())
        }
    }

    pub async fn closed(&self) -> bool {
        *self.closed.read().await
    }

    pub fn header(&mut self) -> String {
        self.header.clone().unwrap()
    }

    pub fn get_ip(&mut self) -> String {
        self.request.as_mut().unwrap().get_ip()
    }
}
