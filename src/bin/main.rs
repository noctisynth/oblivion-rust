use oblivion::models::server::Server;
use oblivion::models::server::Router;

#[tokio::main]
async fn main() {
    // let mut r = get("oblivion://127.0.0.1:813/test", true).await.unwrap();
    // println!("{}", r.text())
    let routes = Router::new(None);
    let mut server = Server::new("127.0.0.1", 813, routes);
    server.run().await;
}
