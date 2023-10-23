extern crate base64;

use rsa::{
    pkcs8::{der::Writer, DecodePublicKey, EncodePublicKey},
    RsaPublicKey,
};

use crate::{
    exceptions::{AddressAlreadyInUse, ErrorNotPrepared},
    utils::{
        decryptor::{decrypt_aes_key, decrypt_message},
        encryptor::{encrypt_aes_key, encrypt_message},
        generator::{generate_aes_key, generate_key_pair},
        parser::{length, Oblivion, OblivionPath},
    },
};
use std::{
    io::Read,
    net::{TcpListener, TcpStream},
};

pub struct ServerConnection {
    tcp: TcpListener,
    client: TcpListener,
}

pub struct Request {
    method: String,
    path: OblivionPath,
    olps: String,
    oblivion: Oblivion,
    tcp: Option<TcpStream>,
    aes_key: Option<Vec<u8>>,
    plain_text: String,
    data: Option<String>,
    prepared: bool,
}

impl Request {
    pub fn new(method: &str, olps: &str) -> Self {
        let path = OblivionPath::new(olps).expect("Invalid olps");
        let olps = path.get_olps();
        let oblivion = Oblivion::new(method, &olps).unwrap();
        let plain_text = oblivion.plain_text();
        Self {
            method: method.to_string(),
            path,
            olps: olps.to_string(),
            oblivion,
            tcp: None,
            aes_key: None,
            plain_text,
            data: None,
            prepared: false,
        }
    }

    pub fn prepare(&mut self) -> Result<(), AddressAlreadyInUse> {
        self.tcp = Some(
            TcpStream::connect(format!("{}:{}", self.path.get_host(), self.path.get_port()))
                .unwrap(),
        );
        if self.tcp.is_none() {
            return Err(AddressAlreadyInUse);
        }
        let tcp = self.tcp.as_mut().unwrap();

        let mut len_server_public_key_bytes: Vec<u8> = vec![0; 4];
        let _ = tcp.read_exact(&mut len_server_public_key_bytes); // 捕获RSA_KEY长度

        let len_server_public_key_str = std::str::from_utf8(&len_server_public_key_bytes).unwrap();
        println!("len_server_public_key_str: {}", len_server_public_key_str);
        let len_server_public_key_int: i32 = std::str::from_utf8(&len_server_public_key_bytes)
            .unwrap()
            .parse()
            .expect("Failed to receieve length");
        println!("len_server_public_key_int: {}", len_server_public_key_int);

        let len_server_public_key: usize = len_server_public_key_int
            .try_into()
            .expect("Failed to generate unsize value");

        let mut server_public_key_bytes: Vec<u8> = vec![0; len_server_public_key];
        let _ = tcp
            .read_exact(&mut server_public_key_bytes)
            .expect("Failed to recv RSA_KEY");

        let server_public_key_pem = String::from_utf8(server_public_key_bytes.clone()).unwrap().trim().to_string();
        println!("server_public_key: {}", server_public_key_pem);

        let server_public_key = RsaPublicKey::from_public_key_pem(&server_public_key_pem)
            .expect("Failed to load RSA_KEY");

        self.aes_key = Some(generate_aes_key()); // 生成随机的AES密钥
        let encrypted_aes_key = encrypt_aes_key(&self.aes_key.clone().unwrap(), server_public_key); // 使用RSA公钥加密AES密钥

        let len_encrypted_aes_key = length(&encrypted_aes_key.clone());
        let _ = tcp.write(&len_encrypted_aes_key); // 发送AES_KEY长度
        let _ = tcp.write(&encrypted_aes_key); // 发送AES_KEY

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
        let _ = tcp.write(&length(&ciphertext));
        let _ = tcp.write(&ciphertext);

        // 发送nonce
        let _ = tcp.write(&length(&nonce));
        let _ = tcp.write(&nonce);

        // 发送tag
        let _ = tcp.write(&length(&tag));
        let _ = tcp.write(&tag);
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
        let _ = tcp.write(&length(&publib_key_pem));
        let _ = tcp.write(&publib_key_pem);

        let mut len_encrypted_aes_key_bytes: Vec<u8> = vec![0; 4];
        let _ = tcp.read_exact(&mut len_encrypted_aes_key_bytes); // 捕获AES_KEY长度
        let len_encrypted_aes_key_int: i32 = std::str::from_utf8(&len_encrypted_aes_key_bytes)
            .unwrap()
            .parse()
            .expect("Failed to receieve length");
        let len_encrypted_aes_key: usize = len_encrypted_aes_key_int
            .try_into()
            .expect("Failed to generate unsize value");
        let mut encrypted_aes_key: Vec<u8> = vec![0; len_encrypted_aes_key];
        let _ = tcp.read_exact(&mut encrypted_aes_key); // 捕获AES_KEY
        let decrypted_aes_key = decrypt_aes_key(&encrypted_aes_key, private_key); // 使用RSA私钥解密AES密钥
        println!("decrypted_aes_key: {:?}", decrypted_aes_key);

        let mut len_recv_bytes: Vec<u8> = vec![0; 4];
        let _ = tcp.read_exact(&mut len_recv_bytes);
        let len_recv_int: i32 = std::str::from_utf8(&len_recv_bytes)
            .unwrap()
            .parse()
            .expect("Failed to receieve length");
        let len_recv: usize = len_recv_int
            .try_into()
            .expect("Failed to generate unsize value");
        let mut encrypted_data: Vec<u8> = vec![0; len_recv];
        let _ = tcp.read_exact(&mut encrypted_data);
        println!("encrypted_data: {:?}", encrypted_data);

        let mut len_nonce_bytes: Vec<u8> = vec![0; 4];
        let _ = tcp.read_exact(&mut len_nonce_bytes);
        let len_nonce_int: i32 = std::str::from_utf8(&len_nonce_bytes)
            .unwrap()
            .parse()
            .expect("Failed to receieve length");
        let len_nonce: usize = len_nonce_int
            .try_into()
            .expect("Failed to generate unsize value");
        let mut nonce: Vec<u8> = vec![0; len_nonce];
        let _ = tcp.read_exact(&mut nonce);
        println!("nonce: {:?}", nonce);

        let mut len_tag_bytes: Vec<u8> = vec![0; 4];
        let _ = tcp.read_exact(&mut len_tag_bytes);
        let len_tag_int: i32 = std::str::from_utf8(&len_tag_bytes)
            .unwrap()
            .parse()
            .expect("Failed to receieve length");
        let len_tag: usize = len_tag_int
            .try_into()
            .expect("Failed to generate unsize value");
        let mut tag: Vec<u8> = vec![0; len_tag];
        let _ = tcp.read_exact(&mut tag);
        println!("tag: {:?}", tag);

        self.data = Some(decrypt_message(
            encrypted_data,
            &tag,
            &decrypted_aes_key,
            &nonce,
        ));
        println!("data: {}", self.data.clone().unwrap());
        Ok(self.data.clone().unwrap())
    }
}
