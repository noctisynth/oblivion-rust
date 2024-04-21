//! # Oblivion Client
use crate::models::packet::{OED, OSC};

use crate::exceptions::OblivionException;
#[cfg(feature = "python")]
use crate::exceptions::PyOblivionException;

use crate::utils::gear::Socket;
use crate::utils::generator::generate_key_pair;
use crate::utils::parser::{Oblivion, OblivionPath};

use anyhow::{Error, Result};
use p256::ecdh::EphemeralSecret;
use p256::PublicKey;
use tokio::net::TcpStream;

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(not(feature = "python"))]
use serde_json::{from_str, Value};
#[cfg(feature = "python")]
use serde_json::{json, Value};

use super::session::Session;

#[cfg_attr(feature = "python", pyclass)]
#[derive(Debug, Default)]
pub struct Response {
    #[cfg_attr(feature = "python", pyo3(get))]
    pub header: String,
    #[cfg_attr(feature = "python", pyo3(get))]
    pub content: Vec<u8>,
    #[cfg_attr(feature = "python", pyo3(get))]
    pub olps: String,
    #[cfg_attr(feature = "python", pyo3(get))]
    pub status_code: u32,
}

#[cfg(not(feature = "python"))]
impl Response {
    pub fn new(header: String, content: Vec<u8>, olps: String, status_code: u32) -> Self {
        Self {
            header,
            content,
            olps,
            status_code,
        }
    }

    pub fn equals(&mut self, response: Response) -> bool {
        response.header == self.header
            && response.content == self.content
            && response.olps.trim_end_matches("/") == self.olps.trim_end_matches("/")
            && response.status_code == self.status_code
    }

    pub fn ok(&self) -> bool {
        self.status_code < 400
    }

    pub fn text(&mut self) -> Result<String> {
        Ok(String::from_utf8(self.content.to_vec())?)
    }

    pub fn json(&mut self) -> Result<Value> {
        Ok(from_str::<Value>(&self.text()?)?)
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl Response {
    #[new]
    pub fn new(header: String, content: Vec<u8>, olps: String, status_code: u32) -> Self {
        Self {
            header,
            content,
            olps,
            status_code,
        }
    }

    fn ok(&self) -> bool {
        self.status_code < 400
    }

    fn text(&mut self) -> PyResult<String> {
        match String::from_utf8(self.content.to_vec()) {
            Ok(text) => Ok(text),
            Err(_) => Err(PyErr::new::<PyOblivionException, _>(format!(
                "Invalid Oblivion: {}",
                self.olps
            ))),
        }
    }
}

pub struct Client {
    olps: String,
    path: OblivionPath,
    plain_text: String,
    private_key: Option<EphemeralSecret>,
    public_key: Option<PublicKey>,
    session: Option<Session>,
}

impl Client {
    pub fn new(method: &str, olps: String) -> Result<Self> {
        let method = method.to_uppercase();
        let path = OblivionPath::new(&olps)?;
        let olps = path.get_olps();
        let oblivion = Oblivion::new(&method, &olps);
        let plain_text = oblivion.plain_text();
        Ok(Self {
            olps,
            path,
            plain_text: plain_text.clone(),
            private_key: None,
            public_key: None,
            session: None,
        })
    }

    pub async fn connect(&mut self) -> Result<()> {
        let (private_key, public_key) = generate_key_pair()?;
        (self.private_key, self.public_key) = (Some(private_key), Some(public_key));

        let tcp =
            match TcpStream::connect(format!("{}:{}", self.path.get_host(), self.path.get_port()))
                .await
            {
                Ok(tcp) => {
                    tcp.set_ttl(20)?;
                    tcp
                }
                Err(_) => return Err(Error::from(OblivionException::ConnectionRefusedError)),
            };

        self.session = Some(Session::new_with_header(
            &self.plain_text,
            Socket::new(tcp),
        )?);

        let session = self.session.as_mut().unwrap();
        session.handshake(0).await?;

        Ok(())
    }

    pub async fn send(&mut self, bytes: Vec<u8>) -> Result<()> {
        let session = self.session.as_mut().unwrap();
        let tcp = &mut session.socket.lock().await;
        let mut oed = OED::new(session.aes_key.clone());
        oed.from_bytes(bytes)?;
        oed.to_stream(tcp, 5).await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Response> {
        let session = self.session.as_mut().unwrap();
        let socket = &mut session.socket.lock().await;

        let flag = OSC::from_stream(socket).await?.status_code;
        let content = OED::new(session.aes_key.clone())
            .from_stream(socket, 5)
            .await?
            .get_data();
        let status_code = OSC::from_stream(socket).await?.status_code;

        let response = Response::new(
            self.plain_text.clone(),
            content,
            self.olps.clone(),
            status_code,
        );

        if flag == 1 {
            socket.close().await?;
        }
        Ok(response)
    }

    pub async fn close(&mut self) -> Result<()> {
        let session = self.session.as_mut().unwrap();
        let tcp = &mut session.socket.lock().await;
        tcp.close().await?;
        Ok(())
    }
}
