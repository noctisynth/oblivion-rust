use std::collections::HashMap;

use oblivion::models::render::BaseResponse;
use oblivion::models::router::Router;
use oblivion::models::server::Server;
use oblivion::utils::parser::OblivionRequest;

fn test2(_: &mut OblivionRequest) -> BaseResponse {
    BaseResponse::TextResponse("毁灭人类!!!!".to_string(), 200)
}

#[tokio::main]
async fn main() {
    let mut routes = Router::new(Some(HashMap::new()));
    // routes.regist(route!("/test2" => test2));
    routes.route("/test2".to_string(), test2);

    let mut server = Server::new("127.0.0.1", 813, routes);
    server.run().await;
}
