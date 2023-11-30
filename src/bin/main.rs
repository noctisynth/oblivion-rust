use std::collections::HashMap;

use oblivion::models::render::BaseResponse;
use oblivion::models::router::Router;
use oblivion::models::server::Server;
use oblivion::route;
use oblivion::utils::parser::OblivionRequest;

fn test2(_: &mut OblivionRequest) -> BaseResponse {
    BaseResponse::TextResponse("毁灭人类!!!!".to_string(), 200)
}

#[tokio::main]
async fn main() {
    let mut router = Router::new(Some(HashMap::new()));
    // routes.regist(route!("/test2" => test2));

    router.route("/test2", test2);
    route!(&mut router, "/path" => test2);

    let mut server = Server::new("127.0.0.1", 813, router);
    server.run().await;
}
