//! # Oblivion Server
use std::net::SocketAddr;
use std::sync::Arc;

use crate::utils::gear::Socket;
#[cfg(not(feature = "bench"))]
use crate::VERSION;

use anyhow::{Error, Result};
use chrono::Local;
use colored::Colorize;
#[cfg(feature = "bench")]
use std::process;
use tokio::net::{TcpListener, TcpStream};
#[cfg(feature = "perf")]
use tokio::time::Instant;

use super::packet::{OED, OSC};
use super::router::Router;
use super::session::Session;

#[inline]
async fn _handle(router: &Router, stream: TcpStream, peer: SocketAddr) -> Result<()> {
    #[cfg(feature = "perf")]
    let now = std::time::Instant::now();
    stream.set_ttl(20)?;
    stream.set_nodelay(true)?;
    stream.set_linger(Some(std::time::Duration::from_secs(0)))?;
    socket2::SockRef::from(&stream).set_keepalive(true)?;
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
        #[cfg(feature = "bench")]
        {
            eprintln!("Handshake failed in benchmark test unexpectedly.");
            process::exit(1);
        }
        #[cfg(not(feature = "bench"))]
        return Ok(());
    }

    #[cfg(feature = "perf")]
    println!(
        "握手时长: {}μs",
        now.elapsed().as_micros().to_string().bright_magenta()
    );

    #[cfg(not(any(feature = "perf", feature = "bench")))]
    let header = session.header().to_string();
    #[cfg(not(any(feature = "perf", feature = "bench")))]
    let ip_addr = session.get_ip().to_string();
    let aes_key = session.aes_key.clone();

    #[cfg(not(any(feature = "perf", feature = "bench")))]
    println!(
        "{} -> [{}] \"{}\" {}",
        ip_addr.cyan(),
        Local::now().format("%d/%m/%Y %H:%M:%S"),
        header.green(),
        "OK".cyan()
    );

    #[cfg(feature = "perf")]
    let now = Instant::now();

    let socket = Arc::clone(&session.socket);

    let callback = router.get_handler(&session.request.entrance)?(session).await?;

    #[cfg(not(any(feature = "perf", feature = "bench")))]
    let status_code = callback.get_status_code()?;

    #[cfg(feature = "perf")]
    println!(
        "业务函数时长: {}μs",
        now.elapsed().as_micros().to_string().bright_magenta()
    );

    #[cfg(feature = "perf")]
    let now = Instant::now();

    OSC::from_u32(1).to_stream(&socket).await?;
    OED::new(&aes_key)
        .from_bytes(callback.as_bytes()?)?
        .to_stream(&socket)
        .await?;
    OSC::from_u32(callback.get_status_code()?)
        .to_stream(&socket)
        .await?;
    socket.close().await?;

    #[cfg(feature = "perf")]
    println!(
        "结束函数时长: {}μs",
        now.elapsed().as_micros().to_string().bright_magenta()
    );

    #[cfg(not(any(feature = "perf", feature = "bench")))]
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

pub async fn handle(router: Arc<Router>, stream: TcpStream, peer: SocketAddr) {
    #[cfg(feature = "perf")]
    let now = Instant::now();
    #[cfg(feature = "perf")]
    println!("=================");
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
            eprintln!("{}", error.to_string().bright_red());
            #[cfg(feature = "bench")]
            {
                eprintln!("An error occurred in handling runtime unexpectedly.");
                process::exit(1);
            }
        }
    }
    #[cfg(feature = "perf")]
    println!(
        "总执行时长: {}μs\n=================",
        now.elapsed().as_micros().to_string().bright_magenta()
    );
}

/// Oblivion Server
///
/// Oblivion uses the `tokio` library to handle TCP connections. The `Server` struct
/// is responsible for creating and managing the TCP listener and handling incoming
/// connections. The `handle` function is called for each incoming connection,
/// which creates a new `Session` and handles the incoming data. The `Router`
/// is used to determine which handler function to call based on the incoming
/// request.
///
/// # Example
///
/// ```rust
/// # use oblivion::models::server::Server;
/// # use oblivion::models::router::Router;
/// # use anyhow::Result;
/// # async fn runner() -> Result<()> {
/// let router = Router::new(); // Create an empty router
/// // Create an oblivion server and bind it to 127.0.0.1:8080
/// let server = Server::new("127.0.0.1", 0, router);
/// server.run().await;
/// # Ok(())
/// # }
///
/// # #[tokio::main]
/// # async fn main() {
/// #     let future = tokio::spawn(async {
/// #         if let Err(error) = runner().await {
/// #             panic!("An error occurred: {}", error);
/// #         }
/// #     });
/// #     future.abort();
/// # }
/// ```
pub struct Server {
    host: String,
    port: i32,
    router: Arc<Router>,
}

impl Server {
    pub fn new(host: &str, port: i32, router: Router) -> Self {
        Self {
            host: host.to_string(),
            port,
            router: Arc::new(router),
        }
    }

    pub async fn run(&self) -> Result<()> {
        #[cfg(not(feature = "bench"))]
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
        #[cfg(all(not(feature = "unsafe"), not(feature = "bench")))]
        println!(
            "Oblivion version {}, using '{}'",
            VERSION.bright_yellow(),
            "ring".bright_green()
        );

        #[cfg(not(feature = "bench"))]
        println!(
            "Starting server at {}",
            format!("Oblivion://{}:{}/", self.host, self.port).bright_cyan()
        );
        #[cfg(not(feature = "bench"))]
        println!("Quit the server by CTRL-BREAK.\n");

        while let Ok((stream, peer)) = tcp.accept().await {
            tokio::spawn(handle(Arc::clone(&self.router), stream, peer));
        }

        Ok(())
    }
}
