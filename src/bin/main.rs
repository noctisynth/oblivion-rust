use anyhow::Result;
use oblivion::api::get;
use oblivion::models::render::{BaseResponse, Response};
use oblivion::models::router::{RoutePath, RouteType, Router};
use oblivion::models::server::Server;
use oblivion::models::session::Session;
use oblivion::path_route;
use oblivion_codegen::async_route;
use serde_json::json;
use std::env::args;
use std::time::Instant;

#[async_route]
fn handler(_sess: &mut Session) -> Response {
    Ok(BaseResponse::TextResponse(
        "每一个人都应该拥有守护信息与获得真实信息的神圣权利, 任何与之对抗的都是我们的敌人"
            .to_string(),
        200,
    ))
}

#[async_route]
fn welcome(_sess: &mut Session) -> Response {
    Ok(BaseResponse::TextResponse(
        format!("欢迎进入信息绝对安全区, 来自[{}]的朋友", 1),
        200,
    ))
}

#[async_route]
fn json(_sess: &mut Session) -> Response {
    Ok(BaseResponse::JsonResponse(
        json!({"status": true, "msg": "只身堕入极暗之永夜, 以期再世涅槃之阳光"}),
        200,
    ))
}

#[async_route]
async fn alive(mut _sess: Session) -> Response {
    Ok(BaseResponse::JsonResponse(
        json!({"status": true, "msg": "结束"}),
        200,
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = args().collect();
    let is_server = if args.len() == 1 { true } else { false };
    if !is_server {
        loop {
            let now = Instant::now();
            get("127.0.0.1:7076/path").await?;
            println!("执行时间: {}", now.elapsed().as_millis());
        }
    } else {
        let mut router = Router::new();

        router.route(RoutePath::new("/handler", RouteType::Path), handler);

        path_route!(&mut router, "/welcome" => welcome);
        path_route!(&mut router, "/json" => json);

        let mut server = Server::new("0.0.0.0", 7076, router);
        server.run().await?;
    }
    Ok(())
}
