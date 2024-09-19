//! # Oblivion Client
use std::sync::Arc;

use anyhow::{Error, Result};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

use crate::exceptions::Exception;
#[cfg(feature = "pyo3")]
use crate::exceptions::PyOblivionException;

use crate::utils::gear::Socket;
use crate::utils::parser::OblivionPath;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(not(feature = "pyo3"))]
use serde_json::{from_slice, Value};
#[cfg(feature = "pyo3")]
use serde_json::{json, Value};

use super::session::Session;

#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Response {
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    pub header: Option<String>,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    pub content: Vec<u8>,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    pub entrance: Option<String>,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    pub flag: u32,
}

#[cfg(not(feature = "pyo3"))]
impl Response {
    pub fn new(
        header: Option<String>,
        content: Vec<u8>,
        entrance: Option<String>,
        flag: u32,
    ) -> Self {
        Self {
            header,
            content,
            entrance,
            flag,
        }
    }

    pub fn text(&self) -> Result<String> {
        Ok(String::from_utf8(self.content.to_vec())?)
    }

    pub fn json(&self) -> Result<Value> {
        Ok(from_slice(&self.content)?)
    }
}

impl PartialEq for Response {
    fn eq(&self, other: &Self) -> bool {
        if self.entrance.is_none() {
            self.header == other.header
                && self.content == other.content
                && self.entrance == other.entrance
                && self.flag == other.flag
        } else if other.entrance.is_none() {
            false
        } else {
            self.header == other.header
                && self.content == other.content
                && self.entrance.as_ref().unwrap().trim_end_matches("/")
                    == other.entrance.as_ref().unwrap().trim_end_matches("/")
                && self.flag == other.flag
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Response {
    #[new]
    pub fn new(header: String, content: Vec<u8>, entrance: String, status_code: u32) -> Self {
        Self {
            header,
            content,
            entrance,
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
                self.entrance
            ))),
        }
    }
}

pub struct Client {
    pub entrance: String,
    pub path: OblivionPath,
    pub session: Arc<Session>,
}

impl Client {
    pub async fn connect(entrance: &str) -> Result<Self> {
        let path = OblivionPath::new(entrance)?;
        let header = format!("CONNECT {} Oblivion/2.0", path.get_entrance());

        let tcp = match TcpStream::connect(format!("{}:{}", path.get_host(), path.get_port())).await
        {
            Ok(tcp) => {
                tcp.set_ttl(20)?;
                tcp.set_nodelay(true)?;
                socket2::SockRef::from(&tcp).set_keepalive(true)?;
                tcp
            }
            Err(_) => return Err(Error::from(Exception::ConnectionRefusedError)),
        };

        let mut session = Session::new_with_header(header, Socket::new(tcp))?;

        session.handshake(0).await?;

        Ok(Self {
            entrance: entrance.to_string(),
            path,
            session: Arc::new(session),
        })
    }

    pub async fn send(&self, data: Vec<u8>) -> Result<()> {
        self.session.send(data).await
    }

    pub async fn send_json(&self, json: Value) -> Result<()> {
        self.session.send_json(json).await
    }

    pub async fn recv(&self) -> Result<Response> {
        self.session.recv().await
    }

    pub async fn close(&self) -> Result<()> {
        self.session.close().await
    }
}
