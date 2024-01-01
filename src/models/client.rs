use crate::models::packet::{OED, OKE, OSC};

use crate::exceptions::OblivionException;

use crate::utils::gear::Socket;
use crate::utils::generator::generate_key_pair;
use crate::utils::parser::{length, Oblivion, OblivionPath};

use p256::ecdh::EphemeralSecret;
use p256::PublicKey;
use serde_json::{from_str, json, Value};
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

    pub fn text(&mut self) -> Result<String, OblivionException> {
        match String::from_utf8(self.content.to_vec()) {
            Ok(text) => Ok(text),
            Err(_) => Err(OblivionException::InvalidOblivion {
                olps: self.olps.to_string(),
            }),
        }
    }

    pub fn json(&mut self) -> Result<Value, OblivionException> {
        Ok(from_str::<Value>(&self.text()?).unwrap())
    }
}

pub struct Request {
    method: String,
    olps: String,
    path: OblivionPath,
    data: Option<Value>,
    file: Option<Vec<u8>>,
    tfo: bool,
    plain_text: String,
    prepared: bool,
    private_key: Option<EphemeralSecret>,
    public_key: Option<PublicKey>,
    aes_key: Option<Vec<u8>>,
    tcp: Option<Socket>,
}

impl Request {
    pub fn new(
        method: String,
        olps: String,
        data: Option<Value>,
        file: Option<Vec<u8>>,
        tfo: bool,
    ) -> Result<Self, OblivionException> {
        let method = method.to_uppercase();
        let path = OblivionPath::new(&olps)?;
        let olps = path.get_olps();
        let oblivion = Oblivion::new(&method, &olps)?;
        let plain_text = oblivion.plain_text();
        Ok(Self {
            method,
            olps,
            path,
            data,
            file,
            tfo,
            plain_text,
            prepared: false,
            private_key: None,
            public_key: None,
            aes_key: None,
            tcp: None,
        })
    }

    pub async fn prepare(&mut self) -> Result<(), OblivionException> {
        let (private_key, public_key) = generate_key_pair()?;
        (self.private_key, self.public_key) = (Some(private_key), Some(public_key));

        let tcp =
            match TcpStream::connect(format!("{}:{}", self.path.get_host(), self.path.get_port()))
                .await
            {
                Ok(tcp) => {
                    tcp.set_ttl(20).unwrap();
                    tcp
                }
                Err(_) => return Err(OblivionException::ConnectionRefusedError),
            };
        self.tcp = Some(Socket::new(tcp));

        if self.tfo {};
        // TODO 在这里启用TCP Fast Open

        self.send_header().await?;

        let mut oke = OKE::new(Some(&self.private_key.as_ref().unwrap()), self.public_key)?;
        oke.from_stream_with_salt(self.tcp.as_mut().unwrap())
            .await?;
        self.aes_key = Some(oke.get_aes_key());
        oke.to_stream(self.tcp.as_mut().unwrap()).await?;

        self.prepared = true;
        Ok(())
    }

    pub async fn send_header(&mut self) -> Result<(), OblivionException> {
        let tcp = self.tcp.as_mut().unwrap();
        let header = self.plain_text.as_bytes().to_vec();
        tcp.send(&length(&header)?).await?;
        tcp.send(&header).await?;
        Ok(())
    }

    pub async fn send(&mut self) -> Result<(), OblivionException> {
        if self.method == "GET" {
            return Ok(());
        };

        let tcp = self.tcp.as_mut().unwrap();
        let mut oed = if self.method == "POST" {
            if self.data.is_none() {
                let mut oed = OED::new(self.aes_key.clone());
                oed.from_dict(json!({}))?;
                oed
            } else {
                let mut oed = OED::new(self.aes_key.clone());
                oed.from_dict(self.data.clone().unwrap())?;
                oed
            }
        } else if self.method == "PUT" {
            let mut oed = if self.data.is_none() {
                let mut oed = OED::new(self.aes_key.clone());
                oed.from_dict(json!({}))?;
                oed
            } else {
                let mut oed = OED::new(self.aes_key.clone());
                oed.from_dict(self.data.clone().unwrap())?;
                oed
            };

            oed.to_stream(tcp, 5).await?;

            let mut oed = OED::new(self.aes_key.clone());
            oed.from_bytes(self.file.clone().unwrap())?;
            oed
        } else {
            return Err(OblivionException::UnsupportedMethod {
                method: self.method.to_string(),
            });
        };

        oed.to_stream(tcp, 5).await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Response, OblivionException> {
        let tcp = self.tcp.as_mut().unwrap();

        if !self.prepared {
            Err(OblivionException::ErrorNotPrepared)
        } else {
            let mut oed = OED::new(self.aes_key.clone());
            oed.from_stream(tcp, 5).await?;

            let mut osc = OSC::from_stream(tcp).await?;

            let response = Response::new(
                self.plain_text.clone(),
                oed.get_data(),
                self.olps.clone(),
                osc.get_status_code(),
            );
            Ok(response)
        }
    }

    pub fn is_prepared(&mut self) -> bool {
        self.prepared
    }
}
