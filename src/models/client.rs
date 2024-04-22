//! # Oblivion Client
use anyhow::{Error, Result};
use tokio::net::TcpStream;

use crate::exceptions::OblivionException;
#[cfg(feature = "python")]
use crate::exceptions::PyOblivionException;

use crate::utils::gear::Socket;
use crate::utils::parser::OblivionPath;

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(not(feature = "python"))]
use serde_json::{from_str, Value};
#[cfg(feature = "python")]
use serde_json::{json, Value};

use super::session::Session;

#[cfg_attr(feature = "python", pyclass)]
#[derive(Debug, Default)]
pub struct Response<'a> {
    #[cfg_attr(feature = "python", pyo3(get))]
    pub header: Option<&'a str>,
    #[cfg_attr(feature = "python", pyo3(get))]
    pub content: Vec<u8>,
    #[cfg_attr(feature = "python", pyo3(get))]
    pub entrance: Option<&'a str>,
    #[cfg_attr(feature = "python", pyo3(get))]
    pub status_code: u32,
    #[cfg_attr(feature = "python", pyo3(get))]
    pub flag: u32,
}

#[cfg(not(feature = "python"))]
impl<'a> Response<'a> {
    pub fn new(
        header: Option<&'a str>,
        content: Vec<u8>,
        entrance: Option<&'a str>,
        status_code: u32,
        flag: u32,
    ) -> Self {
        Self {
            header,
            content,
            entrance,
            status_code,
            flag,
        }
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

impl<'a> PartialEq for Response<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.entrance.is_none() {
            self.header == other.header
                && self.content == other.content
                && self.entrance == other.entrance
                && self.status_code == other.status_code
                && self.flag == other.flag
        } else {
            if other.entrance.is_none() {
                false
            } else {
                self.header == other.header
                    && self.content == other.content
                    && self.entrance.unwrap().trim_end_matches("/")
                        == other.entrance.unwrap().trim_end_matches("/")
                    && self.status_code == other.status_code
                    && self.flag == other.flag
            }
        }
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
    pub entrance: String,
    pub path: OblivionPath,
    pub header: String,
    pub session: Session,
}

impl Client {
    pub async fn connect(entrance: &str) -> Result<Self> {
        let path = OblivionPath::new(&entrance)?;
        let header = format!("CONNECT {} Oblivion/2.0", path.get_olps());

        let tcp = match TcpStream::connect(format!("{}:{}", path.get_host(), path.get_port())).await
        {
            Ok(tcp) => {
                tcp.set_ttl(20)?;
                tcp
            }
            Err(_) => return Err(Error::from(OblivionException::ConnectionRefusedError)),
        };

        let mut session = Session::new_with_header(&header, Socket::new(tcp))?;

        session.handshake(0).await?;

        Ok(Self {
            entrance: entrance.to_string(),
            path,
            header,
            session,
        })
    }

    pub async fn send(&mut self, data: Vec<u8>, status_code: u32) -> Result<()> {
        let session = &mut self.session;

        Ok(session.send(data, status_code).await?)
    }

    pub async fn recv(&mut self) -> Result<Response> {
        let session = &mut self.session;

        Ok(session.recv().await?)
    }

    pub async fn close(&mut self) -> Result<()> {
        let session = &mut self.session;
        let tcp = &mut session.socket.lock().await;
        tcp.close().await?;
        Ok(())
    }
}
