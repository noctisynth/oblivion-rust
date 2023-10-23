use std::collections::HashMap;
use std::str::FromStr;

use super::super::exceptions::InvalidOblivion;
// use super::gear::Kwargs;
use regex::Regex;

pub(crate) fn length(string: &Vec<u8>) -> Vec<u8> {
    let str_num = string.len().to_string();
    if str_num.len() == 4 {
        return str_num.into_bytes();
    }

    let mut list_num: Vec<char> = str_num.chars().collect();
    while list_num.len() != 4 {
        list_num.insert(0, '0');
    }

    list_num.into_iter().collect::<String>().into_bytes()
}

pub struct OblivionPath {
    protocol: String,
    host: String,
    port: String,
    olps: String,
}

impl OblivionPath {
    pub fn new(obl_str: &str) -> Result<Self, InvalidOblivion> {
        let re = Regex::new(
            r"^(?P<protocol>oblivion)?(?:://)?(?P<host>[^:/]+)(:(?P<port>\d+))?(?P<url>/.+)?$",
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
                .unwrap_or(&None)
                .unwrap_or_default()
                .to_string();
            let url = extracted_values
                .get("url")
                .unwrap_or(&None)
                .unwrap_or_default()
                .to_string();
            Ok(Self {
                protocol: protocol,
                host: host,
                port: port,
                olps: url,
            })
        } else {
            Err(InvalidOblivion)
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
    // kwargs: Kwargs,
}

impl Oblivion {
    pub fn new(method: &str, olps: &str) -> Result<Self, InvalidOblivion> {
        // let method = kwargs.get("method", "GET");
        // let olps = kwargs.get("olps", "/");
        // let version = kwargs.get("version", "1.0.0");
        Ok(Self {
            method: method.to_string(),
            olps: olps.to_string(),
            version: String::from_str("1.0.0").unwrap(),
            // kwargs: kwargs,
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
