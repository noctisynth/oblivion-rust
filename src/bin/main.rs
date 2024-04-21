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
fn handler(_sess: Session) -> Response {
    Ok(BaseResponse::TextResponse(
        "每一个人都应该拥有守护信息与获得真实信息的神圣权利, 任何与之对抗的都是我们的敌人"
            .to_string(),
        200,
    ))
}

#[async_route]
fn welcome(mut sess: Session) -> Response {
    Ok(BaseResponse::TextResponse(
        format!(
            "欢迎进入信息绝对安全区, 来自[{}]的朋友",
            sess.request.as_mut().unwrap().get_ip()
        ),
        200,
    ))
}

#[async_route]
fn json(_sess: Session) -> Response {
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
    let mut args: Vec<String> = args().collect();
    if args.len() <= 1 {
        args.push("serve".to_string());
    }
    match args[1].as_str() {
        "bench" => loop {
            let now = Instant::now();
            let mut res = get("127.0.0.1:7076/welcome").await?;
            println!("{}", res.text()?);
            println!("执行时间: {}", now.elapsed().as_millis());
        },
        "socket" => todo!(),
        "serve" => {
            let mut router = Router::new();

            router.route(RoutePath::new("/handler", RouteType::Path), handler);

            path_route!(&mut router, "/welcome" => welcome);
            path_route!(&mut router, "/json" => json);

            let mut server = Server::new("0.0.0.0", 7076, router);
            server.run().await?;
        }
        _ => {
            print!("未知的指令: {}", args[1]);
        }
    }

    Ok(())
}
