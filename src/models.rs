extern crate base64;

use rsa::{
    pkcs8::{DecodePublicKey, EncodePublicKey},
    RsaPublicKey,
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
use std::net::{TcpListener, TcpStream};

pub struct ServerConnection {
    tcp: TcpListener,
    client: TcpListener,
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
        println!("data: {}", self.data.clone().unwrap());
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
