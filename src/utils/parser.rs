//! # Oblivion 解析器
//!
//! 用于对数据进行解析重构并存储。
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::exceptions::OblivionException;
use regex::Regex;
use serde_json::Value;

/// 数据包大小分析函数
///
/// `length`接受一个`Vec<u8>`的字节流，得到其不多于四位数的数据大小，如果数据超出预计范围，它将抛出一个异常。
///
/// 它最终返回的值是一个`Vec<u8>`，它由一个四位数以字符串转换而来。
///
/// ```rust
/// use oblivion::utils::parser::length;
///
/// let vec = b"fw4rg45245ygergeqwrgqwerg342rg342gjisdu".to_vec();
///
/// assert_eq!(b"0039".to_vec(), length(&vec).unwrap());
/// ```
///
/// 以上示例中的`vec`是一个长度为 39 的`Vec<u8>`，`length(&vec)`得到了`b"0039".to_vec()`。
pub fn length(bytes: &Vec<u8>) -> Result<Vec<u8>, OblivionException> {
    let str_num = bytes.len().to_string();
    if str_num.len() == 4 {
        return Ok(str_num.into_bytes());
    } else if str_num.len() >= 4 {
        return Err(OblivionException::DataTooLarge { size: bytes.len() });
    }

    let mut list_num: Vec<char> = str_num.chars().collect();
    while list_num.len() != 4 {
        list_num.insert(0, '0');
    }

    Ok(list_num.into_iter().collect::<String>().into_bytes())
}

pub struct OblivionPath {
    protocol: String,
    host: String,
    port: String,
    olps: String,
}

impl OblivionPath {
    pub fn new(obl_str: &str) -> Result<Self, OblivionException> {
        let re = Regex::new(
            r"^(?P<protocol>oblivion)?(?:://)?(?P<host>[^:/]+)(:(?P<port>\d+))?(?P<url>.+)?$",
        )
        .unwrap();

        if let Some(captures) = re.captures(obl_str) {
            let mut extracted_values: HashMap<&str, Option<&str>> = HashMap::new();

            for capture_name in re.capture_names() {
                if let Some(capture_name) = capture_name {
                    let value = captures.name(capture_name).map(|m| m.as_str());
                    extracted_values.insert(capture_name, value);
                }
            }

            let protocol = extracted_values
                .get("protocol")
                .unwrap_or(&None)
                .unwrap_or_default()
                .to_string();
            let host = extracted_values
                .get("host")
                .unwrap_or(&None)
                .unwrap_or_default()
                .to_string();
            let port = extracted_values
                .get("port")
                .unwrap_or(&Some("80"))
                .unwrap_or_default()
                .to_string();
            let url = extracted_values
                .get("url")
                .unwrap_or(&Some("/"))
                .unwrap_or_default()
                .to_string();
            Ok(Self {
                protocol: protocol,
                host: host,
                port: port,
                olps: url,
            })
        } else {
            Err(OblivionException::InvalidOblivion {
                olps: obl_str.to_string(),
            })
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

pub struct Oblivion {
    method: String,
    olps: String,
    version: String,
}

impl Oblivion {
    pub fn new(method: &str, olps: &str) -> Result<Self, OblivionException> {
        Ok(Self {
            method: method.to_string(),
            olps: olps.to_string(),
            version: "1.1".to_string(),
        })
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

#[derive(Clone, Debug, PartialEq)]
pub struct OblivionRequest {
    method: String,
    olps: String,
    protocol: String,
    version: String,
    data: Option<String>,
    plain_text: String,
    post: Option<Value>,
    put: Option<Vec<u8>>,
    remote_addr: Option<String>,
    remote_port: Option<i32>,
}

impl OblivionRequest {
    pub fn new(header: &str) -> Result<Self, OblivionException> {
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
                method: method,
                olps: olps,
                protocol: protocol,
                version: version,
                data: None,
                plain_text: plain_text.to_owned(),
                post: None,
                put: None,
                remote_addr: None,
                remote_port: None,
            })
        } else {
            Err(OblivionException::BadProtocol {
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
