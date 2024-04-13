//! # Oblivion Server
use std::net::SocketAddr;

use crate::exceptions::OblivionException;
use crate::models::packet::{OED, OKE, OSC};
use crate::utils::gear::Socket;
use crate::utils::generator::generate_key_pair;
use crate::utils::parser::OblivionRequest;

use chrono::Local;
use p256::ecdh::EphemeralSecret;
use p256::PublicKey;

use anyhow::{Error, Result};
use colored::Colorize;
use serde_json::from_slice;
use tokio::net::{TcpListener, TcpStream};

use super::router::{Route, Router};

/// Server Connection Solver
///
/// Handshake between server and client.
pub struct ServerConnection {
    private_key: EphemeralSecret,
    public_key: PublicKey,
    aes_key: Option<Vec<u8>>,
}

impl ServerConnection {
    pub fn new() -> Result<Self> {
        let (private_key, public_key) = generate_key_pair()?;

        Ok(Self {
            private_key,
            public_key,
            aes_key: None,
        })
    }

    pub async fn handshake(
        &mut self,
        stream: &mut Socket,
        peer: SocketAddr,
    ) -> Result<OblivionRequest> {
        let len_header = stream.recv_len().await?;
        let header = stream.recv_str(len_header).await?;
        let mut request = OblivionRequest::new(&header)?;
        request.set_remote_peer(&peer);

        let mut oke = OKE::new(Some(&self.private_key), Some(self.public_key))?;
        oke.to_stream_with_salt(stream).await?;
        oke.from_stream(stream).await?;

        request.aes_key = Some(oke.get_aes_key());
        self.aes_key = Some(oke.get_aes_key());

        if request.method == "POST" {
            let mut oed = OED::new(self.aes_key.clone());
            oed.from_stream(stream, 5).await?;
            request.set_post(from_slice(&oed.get_data())?);
        } else if request.method == "GET" {
        } else if request.method == "PUT" {
            let mut oed = OED::new(self.aes_key.clone());
            oed.from_stream(stream, 5).await?;
            request.set_post(from_slice(&oed.get_data())?);

            let mut oed = OED::new(self.aes_key.clone());
            oed.from_stream(stream, 5).await?;
            request.set_put(oed.get_data());
        } else {
            return Err(Error::from(OblivionException::UnsupportedMethod {
                method: request.method,
            }));
        };
        Ok(request)
    }
}

/// Responser
///
/// Send response back to client requester.
pub async fn response(
    route: &mut Route,
    stream: &mut Socket,
    request: OblivionRequest,
    aes_key: Vec<u8>,
) -> Result<i32> {
    let handler = route.get_handler();
    let mut callback = handler(request).await?;

    let mut oed = OED::new(Some(aes_key));
    oed.from_bytes(callback.as_bytes()?)?;
    oed.to_stream(stream, 5).await?;

    let mut osc = OSC::from_int(callback.get_status_code()?);
    osc.to_stream(stream).await?;
    Ok(callback.get_status_code()?)
}

async fn _handle(
    router: &mut Router,
    stream: &mut Socket,
    peer: SocketAddr,
) -> Result<(OblivionRequest, i32)> {
    stream.set_ttl(20);
    let mut connection = ServerConnection::new()?;
    let mut request = match connection.handshake(stream, peer).await {
        Ok(request) => request,
        Err(error) => {
            eprintln!(
                "{} -> [{}] \"{}\" {}",
                peer.ip().to_string().cyan(),
                Local::now().format("%d/%m/%Y %H:%M:%S"),
                "CONNECT".yellow(),
                "500".red()
            );
            return Err(Error::from(error));
        }
    };

    let mut route = router.get_handler(&request.olps)?;
    let status_code = match response(
        &mut route,
        stream,
        request.clone(),
        connection.aes_key.unwrap(),
    )
    .await
    {
        Ok(status_code) => status_code,
        Err(error) => {
            eprintln!(
                "{} -> [{}] \"{}\" {}",
                request.get_ip().cyan(),
                Local::now().format("%d/%m/%Y %H:%M:%S"),
                &request.header.yellow(),
                "501".red()
            );
            return Err(Error::from(error));
        }
    };

    Ok((request, status_code))
}

pub async fn handle(router: Router, stream: TcpStream, peer: SocketAddr) {
    let mut stream = Socket::new(stream);
    let mut router = router;
    match _handle(&mut router, &mut stream, peer).await {
        Ok((mut request, status_code)) => {
            println!(
                "{} -> [{}] \"{}\" {}",
                request.get_ip().cyan(),
                Local::now().format("%d/%m/%Y %H:%M:%S"),
                &request.header.green(),
                if status_code >= 500 {
                    status_code.to_string().red()
                } else if status_code < 500 && status_code >= 400 {
                    status_code.to_string().yellow()
                } else {
                    status_code.to_string().cyan()
                }
            )
        }
        Err(error) => eprintln!("{}", error.to_string().bright_red()),
    }
}

/// Server Core Struct
pub struct Server {
    host: String,
    port: i32,
    router: Router,
}

impl Server {
    pub fn new(host: &str, port: i32, router: Router) -> Self {
        Self {
            host: host.to_string(),
            port,
            router,
        }
    }

    pub async fn run(&mut self) {
        println!("Performing system checks...\n");

        let tcp = match TcpListener::bind(format!("{}:{}", self.host, self.port)).await {
            Ok(tcp) => tcp,
            Err(_) => {
                eprintln!(
                    "{}",
                    OblivionException::AddressAlreadyInUse {
                        ipaddr: self.host.clone(),
                        port: self.port
                    }
                );
                return ();
            }
        };

        println!(
            "Starting server at {}",
            format!("Oblivion://{}:{}/", self.host, self.port).bright_cyan()
        );
        println!("Quit the server by CTRL-BREAK.");

        while let Ok((stream, peer)) = tcp.accept().await {
            let router = self.router.clone();
            tokio::spawn(handle(router, stream, peer));
        }
    }
}
