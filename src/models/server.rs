//! # Oblivion Server
use std::net::SocketAddr;
use std::sync::Arc;

use crate::utils::gear::Socket;
use crate::VERSION;

use anyhow::{Error, Result};
use chrono::Local;
use colored::Colorize;
use tokio::net::{TcpListener, TcpStream};

use super::packet::{OED, OSC};
use super::router::Router;
use super::session::Session;

async fn _handle(router: &Router, stream: TcpStream, peer: SocketAddr) -> Result<()> {
    stream.set_ttl(20)?;
    let mut session = Session::new(Socket::new(stream))?;

    if let Err(error) = session.handshake(1).await {
        eprintln!(
            "{} -> [{}] \"{}\" {}",
            peer.ip().to_string().cyan(),
            Local::now().format("%d/%m/%Y %H:%M:%S"),
            "CONNECT - Oblivion/2.0".yellow(),
            "500".red()
        );
        eprintln!("{}", error.to_string().bright_red());
        return Ok(());
    }

    let header = session.header();
    let ip_addr = session.get_ip();
    let aes_key = session.aes_key.clone().unwrap();

    println!(
        "{} -> [{}] \"{}\" {}",
        ip_addr.cyan(),
        Local::now().format("%d/%m/%Y %H:%M:%S"),
        header.green(),
        "OK".cyan()
    );

    let socket = Arc::clone(&session.socket);

    let mut route = router.get_handler(&session.request.as_ref().unwrap().olps)?;
    let callback = route.get_handler()(session).await?;

    let status_code = callback.get_status_code()?;

    OSC::from_u32(1).to_stream(&socket).await?;
    OED::new(aes_key)
        .from_bytes(callback.as_bytes()?)?
        .to_stream(&socket)
        .await?;
    OSC::from_u32(callback.get_status_code()?)
        .to_stream(&socket)
        .await?;
    socket.close().await?;

    println!(
        "{} <- [{}] \"{}\" {}",
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

    Ok(())
}

pub async fn handle(router: Router, stream: TcpStream, peer: SocketAddr) {
    match _handle(&router, stream, peer).await {
        Ok(()) => {}
        Err(error) => {
            eprintln!(
                "{} <-> [{}] \"{}\" {}",
                peer.ip().to_string().cyan(),
                Local::now().format("%d/%m/%Y %H:%M:%S"),
                "CONNECT - Oblivion/2.0".yellow(),
                "501".red()
            );
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

    pub async fn run(&self) -> Result<()> {
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

        #[cfg(feature = "unsafe")]
        println!(
            "Oblivion version {}, using '{}'",
            VERSION.bright_yellow(),
            "p256".bright_red()
        );
        #[cfg(not(feature = "unsafe"))]
        println!(
            "Oblivion version {}, using '{}'",
            VERSION.bright_yellow(),
            "ring".bright_green()
        );

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
