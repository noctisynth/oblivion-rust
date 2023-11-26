use crate::models::packet::{OED, OKE, OSC};

use crate::exceptions::{self, OblivionException};

use crate::utils::generator::{generate_key_pair, generate_random_salt};
use crate::utils::parser::{length, Oblivion, OblivionPath};

use p256::ecdh::EphemeralSecret;
use p256::PublicKey;
use serde_json::{from_str, to_string, Value};
use tokio::net::TcpStream;

pub struct Response {
    header: String,
    content: Vec<u8>,
    olps: String,
    status_code: i32,
}

impl Response {
    pub fn new(header: String, content: Vec<u8>, olps: String, status_code: i32) -> Self {
        Self {
            header: header,
            content: content,
            olps: olps,
            status_code: status_code,
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

    pub fn text(&mut self) -> String {
        String::from_utf8(self.content.to_vec()).expect("Unable to decode.")
    }

    pub fn json(&mut self) -> Result<Value, serde_json::Error> {
        from_str::<Value>(&to_string(&self.content)?)
    }
}

pub struct Request {
    method: String,
    olps: String,
    path: OblivionPath,
    data: String,
    file: Vec<u8>,
    tfo: bool,
    plain_text: String,
    prepared: bool,
    private_key: Option<EphemeralSecret>,
    public_key: Option<PublicKey>,
    tcp: Option<TcpStream>,
}

impl Request {
    pub fn new(
        method: String,
        olps: String,
        data: String,
        file: Vec<u8>,
        tfo: bool,
    ) -> Result<Self, OblivionException> {
        let method = method.to_uppercase();
        let path = OblivionPath::new(&olps)?;
        let olps = path.get_olps();
        let oblivion = Oblivion::new(&method, &olps)?;
        let plain_text = oblivion.plain_text();
        Ok(Self {
            method: method,
            olps: olps,
            path: path,
            data: data,
            file: file,
            tfo: tfo,
            plain_text: plain_text,
            prepared: false,
            private_key: None,
            public_key: None,
            tcp: None,
        })
    }

    pub async fn prepare(&mut self) -> Result<(), OblivionException> {
        let (private_key, public_key) = generate_key_pair();
        (self.private_key, self.public_key) = (Some(private_key), Some(public_key));

        let tcp =
            match TcpStream::connect(format!("{}:{}", self.path.get_host(), self.path.get_port()))
                .await
            {
                Ok(tcp) => {
                    tcp.set_ttl(20).unwrap();
                    tcp
                }
                Err(_) => {
                    return Err(OblivionException::ConnectionRefusedError(Some(
                        "向服务端的链接请求被拒绝, 可能是由于服务端遭到攻击.".to_string(),
                    )))
                }
            };

        // TODO 在这里启用TCP Fast Open

        Ok(())
    }
}
