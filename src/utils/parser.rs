//! # Oblivion Parser
//!
//! Used to parse and reconstruct data and store it.
use anyhow::{Error, Result};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::exceptions::Exception;

/// Packet size analysis function
///
/// `length` accepts a `Vec<u8>` byte stream, gets its data size in no more than four digits,
/// and throws an exception if the data is out of the expected range.
///
/// The final value it returns is a `Vec<u8>`, which consists of a four-digit number converted to a string.
///
/// ```rust
/// use oblivion::utils::parser::length;
///
/// let vec = b"fw4rg45245ygergeqwrgqwerg342rg342gjisdu".to_vec();
///
/// assert_eq!((39 as u32).to_be_bytes(), length(&vec).unwrap());
/// ```
///
/// The `vec` in the above example is a `Vec<u8>` of length 39, and `length(&vec)` gets `b "0039".to_vec()`.
pub fn length(bytes: &Vec<u8>) -> Result<[u8; 4], Exception> {
    let size = bytes.len() as u32;

    if size > 4096 {
        return Err(Exception::DataTooLarge {
            size: size as usize,
        });
    }

    Ok(size.to_be_bytes())
}

/// Oblivion Location Path String Parser
///
/// ```rust
/// use oblivion::utils::parser::OblivionPath;
///
/// let olps = OblivionPath::new("oblivion://127.0.0.1:813/test").unwrap();
///
/// assert_eq!("oblivion".to_string(), olps.get_protocol());
/// assert_eq!("127.0.0.1".to_string(), olps.get_host());
/// assert_eq!("813".to_string(), olps.get_port());
/// assert_eq!("/test".to_string(), olps.get_olps());
/// ```
pub struct OblivionPath {
    protocol: String,
    host: String,
    port: String,
    olps: String,
}

impl OblivionPath {
    pub fn new(obl_str: &str) -> Result<Self> {
        let re = Regex::new(
            r"^(?P<protocol>oblivion)?(?:://)?(?P<host>[^:/]+)(:(?P<port>\d+))?(?P<olps>.+)?$",
        )?;

        if let Some(captures) = re.captures(obl_str) {
            let mut extracted_values: HashMap<&str, Option<&str>> = HashMap::new();

            for capture_name in re.capture_names() {
                if let Some(capture_name) = capture_name {
                    let value = captures.name(capture_name).map(|m| m.as_str());
                    extracted_values.insert(capture_name, value);
                }
            }

            let protocol = match extracted_values.get("protocol").unwrap() {
                Some(result) => result.to_string(),
                None => "oblivion".to_string(),
            };
            let host = match extracted_values.get("host").unwrap() {
                Some(result) => result.to_string(),
                None => "oblivion".to_string(),
            };
            let port = match extracted_values.get("port").unwrap() {
                Some(result) => result.to_string(),
                None => "80".to_string(),
            };
            let olps = match extracted_values.get("olps").unwrap() {
                Some(result) => result.to_string(),
                None => "/".to_string(),
            };
            Ok(Self {
                protocol,
                host,
                port,
                olps,
            })
        } else {
            Err(Error::from(Exception::InvalidOblivion {
                olps: obl_str.to_string(),
            }))
        }
    }

    pub fn get_protocol(&self) -> String {
        self.protocol.clone()
    }

    pub fn get_olps(&self) -> String {
        self.olps.clone()
    }

    pub fn get_host(&self) -> String {
        self.host.clone()
    }

    pub fn get_port(&self) -> String {
        self.port.clone()
    }
}

/// Oblivion Request Header Generator
///
/// ```rust
/// use oblivion::utils::parser::Oblivion;
///
/// assert_eq!(Oblivion::new("GET", "/test").plain_text().as_str(), "GET /test Oblivion/1.1");
/// ```
pub struct Oblivion {
    method: String,
    olps: String,
    version: String,
}

impl Oblivion {
    pub fn new(method: &str, olps: &str) -> Self {
        Self {
            method: method.to_string(),
            olps: olps.to_string(),
            version: "1.1".to_string(),
        }
    }

    pub fn plain_text(&self) -> String {
        format!(
            "{} {} Oblivion/{}",
            self.method.to_uppercase(),
            self.olps,
            self.version
        )
    }
}

/// Oblivion Request Header Parser
#[derive(Clone, Debug, PartialEq)]
pub struct OblivionRequest {
    pub(crate) header: String,
    pub(crate) method: String,
    pub(crate) olps: String,
    protocol: String,
    version: String,
    data: Option<String>,
    plain_text: String,
    post: Option<Value>,
    put: Option<Vec<u8>>,
    remote_addr: Option<String>,
    remote_port: Option<i32>,
    pub(crate) aes_key: Option<Vec<u8>>,
}

impl OblivionRequest {
    pub fn new(header: &str) -> Result<Self, Exception> {
        let plain_text = header;
        let re =
            Regex::new(r"(?P<method>\w+) (?P<olps>\S+) (?P<protocol>\w+)/(?P<version>\d+\.\d+)")
                .unwrap();

        if let Some(captures) = re.captures(header) {
            let mut extracted_values: HashMap<&str, Option<&str>> = HashMap::new();

            for capture_name in re.capture_names() {
                if let Some(capture_name) = capture_name {
                    let value = captures.name(capture_name).map(|m| m.as_str());
                    extracted_values.insert(capture_name, value);
                }
            }

            let method = extracted_values
                .get("method")
                .unwrap_or(&None)
                .unwrap_or_default()
                .to_string();
            let olps = extracted_values
                .get("olps")
                .unwrap_or(&None)
                .unwrap_or_default()
                .to_string();
            let protocol = extracted_values
                .get("protocol")
                .unwrap_or(&Some("80"))
                .unwrap_or_default()
                .to_string();
            let version = extracted_values
                .get("version")
                .unwrap_or(&Some("/"))
                .unwrap_or_default()
                .to_string();
            Ok(Self {
                method,
                olps,
                protocol,
                version,
                data: None,
                plain_text: plain_text.to_string(),
                post: None,
                put: None,
                remote_addr: None,
                remote_port: None,
                aes_key: None,
                header: header.to_string(),
            })
        } else {
            Err(Exception::BadProtocol {
                header: header.to_string(),
            })
        }
    }

    pub fn set_remote_peer(&mut self, peer: &SocketAddr) {
        self.remote_addr = Some(peer.ip().to_string());
        self.remote_port = Some(peer.port().into())
    }

    pub fn set_post(&mut self, value: Value) {
        self.post = Some(value);
    }

    pub fn set_put(&mut self, bytes: Vec<u8>) {
        self.put = Some(bytes);
    }

    pub fn get_method(&mut self) -> String {
        self.method.clone()
    }

    pub fn get_post(&mut self) -> Value {
        self.post.clone().unwrap()
    }

    pub fn get_put(&mut self) -> Vec<u8> {
        self.put.clone().unwrap()
    }

    pub fn get_olps(&mut self) -> String {
        self.olps.clone()
    }

    pub fn get_protocol(&mut self) -> String {
        self.protocol.clone()
    }

    pub fn get_version(&mut self) -> String {
        self.version.clone()
    }

    pub fn get_ip(&mut self) -> String {
        self.remote_addr.clone().unwrap()
    }
}
