//! # Oblivion Server
use std::net::SocketAddr;
use std::sync::Arc;

use crate::utils::gear::Socket;

use anyhow::{Error, Result};
use chrono::Local;
use colored::Colorize;
use tokio::net::{TcpListener, TcpStream};

use super::packet::{OED, OSC};
use super::router::Router;
use super::session::Session;

async fn _handle(router: &mut Router, mut socket: Socket, peer: SocketAddr) -> Result<String> {
    socket.set_ttl(20)?;

    let mut session = Session::new(socket)?;

    if let Err(error) = session.handshake(1).await {
        eprintln!(
            "{} -> [{}] \"{}\" {}",
            peer.ip().to_string().cyan(),
            Local::now().format("%d/%m/%Y %H:%M:%S"),
            "CONNECT".yellow(),
            "500".red()
        );
        return Err(Error::from(error));
    }

    let header = session.header.as_ref().unwrap().clone();
    let ip_addr = session.request.as_mut().unwrap().get_ip();
    let aes_key = session.aes_key.clone().unwrap();

    let arc_socket = Arc::clone(&session.socket);
    let mut socket = arc_socket.lock().await;

    let mut route = router.get_handler(&session.request.as_ref().unwrap().olps)?;
    let mut callback = route.get_handler()(session).await?;

    let status_code = callback.get_status_code()?;

    OSC::from_u32(1).to_stream(&mut socket).await?;
    OED::new(Some(aes_key))
        .from_bytes(callback.as_bytes()?)?
        .to_stream(&mut socket, 5)
        .await?;
    OSC::from_u32(callback.get_status_code()?)
        .to_stream(&mut socket)
        .await?;

    let display = format!(
        "{} -> [{}] \"{}\" {}",
        ip_addr.cyan(),
        Local::now().format("%d/%m/%Y %H:%M:%S"),
        header.green(),
        if status_code >= 500 {
            status_code.to_string().red()
        } else if status_code < 500 && status_code >= 400 {
            status_code.to_string().yellow()
        } else {
            status_code.to_string().cyan()
        }
    );

    Ok(display)
}

pub async fn handle(router: Router, stream: TcpStream, peer: SocketAddr) {
    let socket = Socket::new(stream);
    let mut router = router;
    match _handle(&mut router, socket, peer).await {
        Ok(display) => {
            println!("{}", display)
        }
        Err(error) => {
            eprintln!("{}", error.to_string().bright_red())
        }
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

    pub async fn run(&mut self) -> Result<()> {
        println!("Performing system checks...\n");

        let address = format!("{}:{}", self.host, self.port);

        let tcp = match TcpListener::bind(&address).await {
            Ok(tcp) => tcp,
            Err(error) => {
                eprintln!(
                    "{}",
                    format!(
                        "Destination address [{}] is already occupied!",
                        address.bright_magenta()
                    )
                    .red()
                );
                return Err(Error::from(error));
            }
        };

        tokio::spawn(async move {
            match tokio::signal::ctrl_c().await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e.to_string().red());
                    std::process::exit(1);
                }
            }
            std::process::exit(0);
        });

        println!(
            "Starting server at {}",
            format!("Oblivion://{}:{}/", self.host, self.port).bright_cyan()
        );
        println!("Quit the server by CTRL-BREAK.\n");

        while let Ok((stream, peer)) = tcp.accept().await {
            let router = self.router.clone();
            tokio::spawn(handle(router, stream, peer));
        }

        Ok(())
    }
}
