extern crate base64;

use rsa::{
    pkcs8::{DecodePublicKey, EncodePublicKey},
    RsaPrivateKey, RsaPublicKey,
};

use crate::{
    exceptions::{AddressAlreadyInUse, ErrorNotPrepared, InvalidOblivion},
    utils::{
        decryptor::{decrypt_aes_key, decrypt_message},
        encryptor::{encrypt_aes_key, encrypt_message},
        gear::Socket,
        generator::{generate_aes_key, generate_key_pair},
        parser::{length, Oblivion, OblivionPath},
    },
};
use std::net::{SocketAddr, TcpListener, TcpStream};

pub struct ServerConnection {
    tcp: Option<TcpListener>,
    client: Option<Socket>,
    client_address: Option<SocketAddr>,
    private_key: Option<RsaPrivateKey>,
    public_key: Option<RsaPublicKey>,
    client_aes_key: Option<Vec<u8>>,
    request_data: Option<Vec<u8>>,
    nonce: Option<Vec<u8>>,
    tag: Option<Vec<u8>>,
}

impl ServerConnection {
    pub fn new(tcp: TcpListener) -> Self {
        Self {
            tcp: Some(tcp),
            client: None,
            client_address: None,
            private_key: None,
            public_key: None,
            client_aes_key: None,
            request_data: None,
            nonce: None,
            tag: None,
        }
    }

    pub fn prepare(&mut self) {}

    pub fn listen(&mut self) -> SocketAddr {
        let (private_key, public_key) = generate_key_pair();
        (self.private_key, self.public_key) = (Some(private_key), Some(public_key));
        let (client, client_address) = self.tcp.as_mut().unwrap().accept().expect("Failed to bind");
        self.client = Some(Socket::new(client));
        self.client_address = Some(client_address);
        client_address
    }

    pub fn get_socket(&mut self) -> &mut Socket {
        self.client.as_mut().unwrap()
    }

    pub fn handshake(&mut self) -> Vec<u8> {
        let client = self.client.as_mut().unwrap();
        let publib_key_pem = self
            .public_key
            .clone()
            .unwrap()
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .expect("Failed to encode as PEM")
            .as_bytes()
            .to_vec();
        client.send(&length(&publib_key_pem));
        client.send(&publib_key_pem);
        let len_encrypted_aes_key = client.recv_len();
        let encrypted_aes_key = client.recv(len_encrypted_aes_key);

        let private_key = self.private_key.clone().unwrap();
        self.client_aes_key = Some(decrypt_aes_key(&encrypted_aes_key, private_key));
        self.client_aes_key.as_ref().unwrap().to_vec()
    }

    pub fn recv(&mut self) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let client = self.client.as_mut().unwrap();
        let len_ciphertext = client.recv_len();
        let ciphertext = client.recv(len_ciphertext);
        self.request_data = Some(ciphertext.clone());

        let len_nonce = client.recv_len();
        let nonce = client.recv(len_nonce);
        self.nonce = Some(nonce.clone());

        let len_tag = client.recv_len();
        let tag = client.recv(len_tag);
        self.tag = Some(tag.clone());

        (ciphertext, tag, nonce)
    }

    pub fn response(&mut self) {}

    pub fn solve(&mut self) -> Result<String, ErrorNotPrepared> {
        if self.client.is_none() {
            return Err(ErrorNotPrepared);
        }

        self.handshake();
        self.recv();
        Ok(decrypt_message(
            self.request_data.clone().unwrap(),
            &self.tag.clone().unwrap(),
            &self.client_aes_key.clone().unwrap(),
            &self.nonce.clone().unwrap(),
        ))
    }
}

pub struct Request {
    method: String,
    path: OblivionPath,
    olps: String,
    oblivion: Oblivion,
    tcp: Option<Socket>,
    aes_key: Option<Vec<u8>>,
    plain_text: String,
    data: Option<String>,
    prepared: bool,
}

impl Request {
    pub fn new(method: &str, olps: &str) -> Result<Self, InvalidOblivion> {
        let path = OblivionPath::new(olps).expect("Invalid olps");
        if path.get_protocol() != "oblivion".to_string() {
            return Err(InvalidOblivion);
        }

        let olps = path.get_olps();
        let oblivion = Oblivion::new(method, &olps).unwrap();
        let plain_text = oblivion.plain_text();
        Ok(Self {
            method: method.to_string(),
            path,
            olps: olps.to_string(),
            oblivion,
            tcp: None,
            aes_key: None,
            plain_text,
            data: None,
            prepared: false,
        })
    }

    pub fn prepare(&mut self) -> Result<(), AddressAlreadyInUse> {
        let tcp = TcpStream::connect(format!("{}:{}", self.path.get_host(), self.path.get_port()))
            .unwrap();
        self.tcp = Some(Socket::new(tcp));

        if self.tcp.is_none() {
            return Err(AddressAlreadyInUse);
        }
        let tcp = self.tcp.as_mut().unwrap();

        let len_server_public_key = tcp.recv_len(); // 捕获RSA_KEY长度
        let server_public_key_pem = tcp.recv_str(len_server_public_key);
        let server_public_key = RsaPublicKey::from_public_key_pem(&server_public_key_pem) // 转义为RSA公钥实例
            .expect("Failed to load RSA_KEY");

        self.aes_key = Some(generate_aes_key()); // 生成随机的AES密钥
        let encrypted_aes_key = encrypt_aes_key(&self.aes_key.clone().unwrap(), server_public_key); // 使用RSA公钥加密AES密钥

        let len_encrypted_aes_key = length(&encrypted_aes_key.clone());
        let _ = tcp.send(&len_encrypted_aes_key); // 发送AES_KEY长度
        let _ = tcp.send(&encrypted_aes_key); // 发送AES_KEY

        self.prepared = true;
        Ok(())
    }

    pub fn is_prepared(&self) -> bool {
        self.prepared
    }

    pub fn send(&mut self) {
        if self.is_prepared() != true {
            let _ = self.prepare();
        }

        let plain_text = self.plain_text.clone();
        let aes_key = self.aes_key.clone().unwrap();
        let (ciphertext, tag, nonce) = encrypt_message(plain_text, &aes_key);

        // 发送完整请求
        let tcp = self.tcp.as_mut().unwrap();
        let _ = tcp.send(&length(&ciphertext));
        let _ = tcp.send(&ciphertext);

        // 发送nonce
        let _ = tcp.send(&length(&nonce));
        let _ = tcp.send(&nonce);

        // 发送tag
        let _ = tcp.send(&length(&tag));
        let _ = tcp.send(&tag);
    }

    pub fn recv(&mut self) -> Result<String, ErrorNotPrepared> {
        if !self.is_prepared() {
            return Err(ErrorNotPrepared);
        }

        let (private_key, public_key) = generate_key_pair();
        let tcp = self.tcp.as_mut().unwrap();
        let publib_key_pem = public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .expect("Failed to encode as PEM")
            .as_bytes()
            .to_vec();
        let _ = tcp.send(&length(&publib_key_pem));
        let _ = tcp.send(&publib_key_pem);

        let len_encrypted_aes_key: usize = tcp.recv_len();
        let encrypted_aes_key: Vec<u8> = tcp.recv(len_encrypted_aes_key); // 捕获AES_KEY
        let decrypted_aes_key = decrypt_aes_key(&encrypted_aes_key, private_key); // 使用RSA私钥解密AES密钥

        let len_recv: usize = tcp.recv_len();
        let encrypted_data: Vec<u8> = tcp.recv(len_recv);

        let len_nonce: usize = tcp.recv_len();
        let nonce: Vec<u8> = tcp.recv(len_nonce);

        let len_tag: usize = tcp.recv_len();
        let tag: Vec<u8> = tcp.recv(len_tag);

        self.data = Some(decrypt_message(
            encrypted_data,
            &tag,
            &decrypted_aes_key,
            &nonce,
        ));

        Ok(self.data.clone().unwrap())
    }

    pub fn get_method(&mut self) -> String {
        self.method.clone()
    }

    pub fn get_olps(&mut self) -> String {
        self.olps.clone()
    }

    pub fn get_oblivion(&mut self) -> &Oblivion {
        &self.oblivion
    }
}

pub struct Hook {
    olps: String,
    data: String,
    method: String,
    aes_key: Option<Vec<u8>>,
    encrypted_aes_key: Option<Vec<u8>>,
}

impl Hook {
    pub fn new(olps: &str, data: &str, method: &str) -> Self {
        Self {
            olps: olps.to_string(),
            data: data.to_string(),
            method: method.to_uppercase(),
            aes_key: None,
            encrypted_aes_key: None,
        }
    }

    pub fn is_valid(&mut self, header: &str) -> bool {
        let splited_header: Vec<&str> = header.split(" ").collect();
        let olps = splited_header[1];
        self.olps.trim_end_matches("/").to_string() == olps.trim_end_matches("/").to_string()
    }

    pub fn prepare(&mut self, tcp: &mut Socket) {
        let len_client_public_key = tcp.recv_len();
        let client_public_key_pem = tcp.recv_str(len_client_public_key);
        let client_public_key = RsaPublicKey::from_public_key_pem(&client_public_key_pem) // 转义为RSA公钥实例
            .expect("Failed to load RSA_KEY");

        self.aes_key = Some(generate_aes_key());
        self.encrypted_aes_key = Some(encrypt_aes_key(
            &self.aes_key.clone().unwrap(),
            client_public_key,
        ));

        tcp.send(&length(&self.encrypted_aes_key.clone().unwrap()));
        tcp.send(&self.encrypted_aes_key.clone().unwrap());
    }

    pub fn response(&mut self, tcp: &mut Socket) {
        let (response, tag, nonce) =
            encrypt_message(self.data.clone(), &self.aes_key.clone().unwrap());

        tcp.send(&length(&response));
        tcp.send(&response);

        tcp.send(&length(&nonce));
        tcp.send(&nonce);

        tcp.send(&length(&tag));
        tcp.send(&tag);

        tcp.close();

        if self.method == "FORWARD" {
            self.forward()
        }
    }

    pub fn forward(&mut self) {}
}

pub struct Server {
    host: String,
    port: i32,
    hooks: Vec<Hook>,
    not_found: Hook,
}

impl Server {
    pub fn new(
        host: &str,
        port: i32,
        hooks: Vec<Hook>,
        not_found: &str,
    ) -> Self {
        let not_found = Hook {
            olps: "/404".to_string(),
            data: not_found.to_string(),
            method: "GET".to_string(),
            aes_key: None,
            encrypted_aes_key: None,
        };
        Self {
            host: host.to_string(),
            port,
            hooks,
            not_found,
        }
    }

    pub fn prepare(&mut self) -> TcpListener {
        TcpListener::bind(format!("{}:{}", self.host, self.port)).unwrap()
    }

    pub fn run(&mut self) {
        let mut connection = ServerConnection::new(self.prepare());
        connection.prepare();

        loop {
            let client_address = connection.listen();
            let header = connection.solve().unwrap();

            let mut exists = false;
            for hook in &mut self.hooks {
                if hook.is_valid(&header) {
                    let mut client_tcp = connection.get_socket();
                    hook.prepare(&mut client_tcp);
                    hook.response(&mut client_tcp);

                    println!(
                        "Oblivion/1.0 From {} {} 200",
                        client_address.ip(),
                        hook.olps
                    );
                    exists = true;
                    break;
                }
            }

            if exists {
                continue;
            } else {
                self.not_found.prepare(connection.get_socket());
                self.not_found.response(connection.get_socket());
                println!("Oblivion/1.0 From {} {} 404", client_address.ip(), self.not_found.olps);
            }
        }
    }
}
