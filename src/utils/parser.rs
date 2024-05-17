//! # Oblivion Parser
//!
//! Used to parse and reconstruct data and store it.
use anyhow::{Error, Result};
use regex::Regex;
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
pub fn length(bytes: &[u8]) -> Result<[u8; 4], Exception> {
    let size = bytes.len() as u32;

    if size > 2048 {
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
/// let entrance = OblivionPath::new("oblivion://127.0.0.1:813/test").unwrap();
///
/// assert_eq!("oblivion".to_string(), entrance.get_protocol());
/// assert_eq!("127.0.0.1".to_string(), entrance.get_host());
/// assert_eq!("813".to_string(), entrance.get_port());
/// assert_eq!("/test".to_string(), entrance.get_entrance());
/// ```
pub struct OblivionPath {
    protocol: String,
    host: String,
    port: String,
    entrance: String,
}

impl OblivionPath {
    pub fn new(path: &str) -> Result<Self> {
        let re = Regex::new(
            r"^(?P<protocol>oblivion)?(?:://)?(?P<host>[^:/]+)(:(?P<port>\d+))?(?P<entrance>.+)?$",
        )?;

        if let Some(captures) = re.captures(path) {
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
            let entrance = match extracted_values.get("entrance").unwrap() {
                Some(result) => result.to_string(),
                None => "/".to_string(),
            };
            Ok(Self {
                protocol,
                host,
                port,
                entrance,
            })
        } else {
            Err(Error::from(Exception::InvalidOblivion {
                entrance: path.to_string(),
            }))
        }
    }

    pub fn get_protocol(&self) -> &str {
        &self.protocol
    }

    pub fn get_entrance(&self) -> &str {
        &self.entrance
    }

    pub fn get_host(&self) -> &str {
        &self.host
    }

    pub fn get_port(&self) -> &str {
        &self.port
    }
}

/// Oblivion Request Header Parser
#[derive(Debug, Default)]
pub struct OblivionRequest {
    pub(crate) method: String,
    pub(crate) entrance: String,
    protocol: String,
    version: String,
    remote_addr: String,
    remote_port: u16,
    pub(crate) aes_key: Option<[u8; 16]>,
}

impl OblivionRequest {
    pub fn new(header: &str) -> Result<Self, Exception> {
        let (mut method, mut entrance, mut protocol, mut version) =
            (String::new(), String::new(), String::new(), String::new());
        header
            .split_whitespace()
            .enumerate()
            .try_for_each(|(index, part)| {
                Ok({
                    match index {
                        0 => method = part.to_string(),
                        1 => entrance = part.to_string(),
                        2 => {
                            let parts: Vec<&str> = part.split("/").collect();
                            if parts.len() == 2 {
                                protocol = parts[0].to_string();
                                version = parts[1].to_string();
                            } else {
                                return Err(Exception::InvalidHeader(header.to_string()));
                            }
                        }
                        _ => return Err(Exception::InvalidHeader(header.to_string())),
                    };
                })
            })?;
        Ok(Self {
            method,
            entrance,
            protocol,
            version,
            remote_addr: String::new(),
            remote_port: 0,
            aes_key: None,
        })
    }

    pub fn set_remote_peer(&mut self, peer: &SocketAddr) {
        self.remote_addr = peer.ip().to_string();
        self.remote_port = peer.port();
    }

    pub fn get_method(&mut self) -> &str {
        &self.method
    }

    pub fn get_olps(&mut self) -> &str {
        &self.entrance
    }

    pub fn get_protocol(&mut self) -> &str {
        &self.protocol
    }

    pub fn get_version(&self) -> &str {
        &self.version
    }

    pub fn get_ip(&self) -> &str {
        &self.remote_addr
    }
}
