use oblivion::api::get;
use oblivion::models::render::BaseResponse;
use oblivion::models::router::Router;
use oblivion::models::server::Server;
use oblivion::route;
use oblivion::utils::parser::OblivionRequest;
use std::collections::HashMap;
use std::env::args;
use std::time::Instant;

fn test2(_: &mut OblivionRequest) -> BaseResponse {
    BaseResponse::TextResponse(
        "每一个人都应该拥有守护信息与获得真实信息的神圣权利, 任何与之对抗的都是我们的敌人"
            .to_string(),
        200,
    )
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = args().collect();
    let is_server = if args.len() == 1 { true } else { false };
    if !is_server {
        loop {
            let now = Instant::now();
            get("127.0.0.1:813/path", true).await.unwrap();
            println!("执行时间: {}", now.elapsed().as_millis());
        }
    } else {
        let mut router = Router::new(Some(HashMap::new()));

        router.route("/test2", test2);
        route!(&mut router, "/path" => test2);

        let mut server = Server::new("127.0.0.1", 813, router);
        server.run().await;
    }
}
