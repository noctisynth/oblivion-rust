use std::collections::HashMap;
use std::str::FromStr;

use crate::exceptions::OblivionException;
use regex::Regex;

pub fn length(bytes: &Vec<u8>) -> Result<Vec<u8>, OblivionException> {
    let str_num = bytes.len().to_string();
    if str_num.len() == 4 {
        return Ok(str_num.into_bytes());
    } else if str_num.len() >= 4 {
        return Err(OblivionException::DataTooLarge(Some(format!(
            "Data in {} exceed max data limit!",
            bytes.len()
        ))));
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
            Err(OblivionException::InvalidOblivion(Some(
                "Bad Olivion location path sequence found.".to_string(),
            )))
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
            version: String::from_str("1.1").unwrap(),
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
