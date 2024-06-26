use anyhow::Result;
use oblivion::models::client::Client;
use oblivion::models::render::BaseResponse;
use oblivion::models::router::{RoutePath, RouteType, Router};
use oblivion::models::server::Server;
use oblivion::models::session::Session;
use oblivion::path_route;
use oblivion::types::server;
use oblivion_codegen::async_route;
use serde_json::json;
use std::env::args;
use tokio::time::Instant;

#[async_route]
fn handler(_sess: Session) -> server::Result {
    Ok(BaseResponse::TextResponse(
        "每一个人都应该拥有守护信息与获得真实信息的神圣权利, 任何与之对抗的都是我们的敌人"
            .to_string(),
        200,
    ))
}

#[async_route]
fn welcome(sess: Session) -> server::Result {
    Ok(BaseResponse::TextResponse(
        format!(
            "欢迎进入信息绝对安全区, 来自[{}]的朋友",
            sess.request.get_ip()
        ),
        200,
    ))
}

#[async_route]
fn json(_sess: Session) -> server::Result {
    Ok(BaseResponse::JsonResponse(
        json!({"status": true, "msg": "只身堕入极暗之永夜, 以期再世涅槃之阳光"}),
        200,
    ))
}

#[async_route]
async fn alive(sess: Session) -> server::Result {
    sess.send("test".into(), 200).await?;
    assert_eq!(sess.recv().await?.text()?, "test");
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
    if args.len() <= 2 {
        args.push("/welcome".to_string());
    }
    match args[1].as_str() {
        "bench" => loop {
            let now = Instant::now();
            let client = Client::connect(&format!("127.0.0.1:7076{}", args[2])).await?;
            client.recv().await?.text()?;
            client.close().await?;
            println!("执行时间: {}", now.elapsed().as_millis());
        },
        "socket" => {
            let client = Client::connect(&format!("127.0.0.1:7076{}", args[2])).await?;
            client.recv().await?.text()?;
            client.send("test".as_bytes().to_vec(), 200).await?;
            client.recv().await?.json()?;
            client.close().await?;
        }
        "serve" => {
            let mut router = Router::new();

            router.route(RoutePath::new("/handler", RouteType::Path), handler);

            path_route!(router, "/welcome" => welcome);
            path_route!(router, "/json" => json);
            path_route!(router, "/alive" => alive);

            let server = Server::new("0.0.0.0", 7076, router);
            server.run().await?;
        }
        _ => {
            print!("未知的指令: {}", args[1]);
        }
    }

    Ok(())
}
