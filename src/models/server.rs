use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use crate::models::packet::{OED, OKE, OSC};
use crate::models::render::BaseResponse;

use crate::exceptions::OblivionException;

use crate::utils::gear::Socket;
use crate::utils::generator::generate_key_pair;
use crate::utils::parser::OblivionRequest;

use p256::ecdh::EphemeralSecret;
use p256::PublicKey;
// use tokio::net::TcpListener;

use serde_json::from_slice;
use tokio::net::TcpListener;

// use super::router::Router;

pub struct ServerConnection {
    private_key: EphemeralSecret,
    public_key: PublicKey,
    aes_key: Option<Vec<u8>>,
}

impl ServerConnection {
    pub fn new() -> Result<Self, OblivionException> {
        let (private_key, public_key) = generate_key_pair();

        Ok(Self {
            private_key: private_key,
            public_key: public_key,
            aes_key: None,
        })
    }

    pub async fn handshake(
        &mut self,
        stream: &mut Socket,
        peer: SocketAddr,
    ) -> Result<OblivionRequest, OblivionException> {
        let len_header = stream.recv_len().await?;
        let header = stream.recv_str(len_header).await?;
        let mut request = OblivionRequest::new(&header)?;
        request.set_remote_peer(peer);

        let mut oke = OKE::new(Some(&self.private_key), Some(self.public_key))?;
        oke.to_stream_with_salt(stream).await;
        self.aes_key = Some(oke.get_aes_key());

        if request.get_method() == "POST" {
            let mut oed = OED::new(self.aes_key.clone());
            let mut oed = oed.from_stream(stream, 5).await?;
            request.set_post(from_slice(&oed.get_data()).unwrap());
        } else if request.get_method() == "GET" {
        } else if request.get_method() == "PUT" {
            let mut oed = OED::new(self.aes_key.clone());
            let mut oed = oed.from_stream(stream, 5).await?;
            request.set_post(from_slice(&oed.get_data()).unwrap());

            let mut oed = OED::new(self.aes_key.clone());
            let mut oed = oed.from_stream(stream, 5).await?;
            request.set_put(oed.get_data());
        } else {
            return Err(OblivionException::UnsupportedMethod(Some(
                request.get_method(),
            )));
        };
        Ok(request)
    }

    pub async fn solve(
        &mut self,
        stream: &mut Socket,
        peer: SocketAddr,
    ) -> Result<OblivionRequest, OblivionException> {
        self.handshake(stream, peer).await
    }
}

pub trait Handler {
    fn handle(&mut self, request: &mut OblivionRequest) -> BaseResponse;
}

pub struct NotFound;

impl Handler for NotFound {
    fn handle(&mut self, _: &mut OblivionRequest) -> BaseResponse {
        todo!()
    }
}

pub struct Router {
    routes: HashMap<String, Arc<Mutex<Box<dyn Handler>>>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    // fn add_route(&mut self, route: String, handler: impl Handler + 'static) {
    //     self.routes.insert(route, Box::new(handler));
    // }

    pub async fn get_handler(&self, route: String) -> Arc<Mutex<Box<dyn Handler>>> {
        if let Some(handler) = self.routes.get(&route) {
            let handler = handler;
            handler.to_owned()
        } else {
            // Handle unknown route
            unimplemented!()
        }
    }
}

pub async fn response(
    handler: Arc<Mutex<Box<dyn Handler>>>,
    stream: &mut Socket,
    request: &mut OblivionRequest,
    aes_key: Vec<u8>,
) -> Result<i32, OblivionException> {
    let mut callback = handler.clone().lock().unwrap().handle(request);

    let mut oed = OED::new(Some(aes_key)).from_bytes(callback.as_bytes()?)?;
    oed.to_stream(stream, 5).await?;

    let mut osc = OSC::from_int(callback.get_status_code()?)?;
    osc.to_stream(stream).await;
    Ok(callback.get_status_code()?)
}

pub struct Server {
    host: String,
    port: i32,
    routes: Router,
}

impl Server {
    pub fn new(host: &str, port: i32, routes: Router) -> Self {
        Self {
            host: host.to_string(),
            port,
            routes,
        }
    }

    async fn _handle(
        &mut self,
        stream: &mut Socket,
        peer: SocketAddr,
    ) -> Result<(OblivionRequest, i32), OblivionException> {
        stream.set_ttl(20);
        let mut connection = ServerConnection::new()?;
        let mut request = match connection.solve(stream, peer).await {
            Ok(requset) => requset,
            Err(_) => return Err(OblivionException::ServerError(None, 500)),
        };

        let handler = self.routes.get_handler(request.get_olps()).await;
        let status_code =
            response(handler, stream, &mut request, connection.aes_key.unwrap()).await?;

        Ok((request.to_owned(), status_code))
    }

    pub async fn handle(&mut self, stream: &mut Socket, peer: SocketAddr) {
        match self._handle(stream, peer).await {
            Ok((mut request, status_code)) => {
                println!(
                    "{}/{} {} From {} {} {}",
                    request.get_protocol(),
                    request.get_version(),
                    request.get_method(),
                    request.get_ip(),
                    request.get_olps(),
                    status_code
                );
            }
            Err(error) => {
                println!("{}", error);
            }
        }
    }

    pub async fn run(&mut self) {
        println!("Performing system checks...\n");

        let tcp = TcpListener::bind(format!("{}:{}", self.host, self.port))
            .await
            .unwrap();

        println!("Starting server at Oblivion://{}:{}/", self.host, self.port);
        println!("Quit the server by CTRL-BREAK.");

        while let Ok((socket, peer)) = tcp.accept().await {
            let mut stream = Socket::new(socket);
            let future = self.handle(&mut stream, peer).await;
            future
            // tokio::spawn(future);
        }
    }
}
