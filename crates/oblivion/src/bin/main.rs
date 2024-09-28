use anyhow::{anyhow, Result};
use futures::future::BoxFuture;
use oblivion::models::client::Client;
use oblivion::models::render::BaseResponse;
use oblivion::models::router::{RoutePath, RouteType, Router};
use oblivion::models::server::Server;
use oblivion::models::session::Session;
use oblivion::path_route;
use oblivion::types::Response;
use oblivion_codegen::async_route;
use serde_json::{json, Value};
use std::env::args;
use std::sync::Arc;
use tokio::time::Instant;

#[async_route]
fn handler(_session: Session) -> String {
    "每一个人都应该拥有守护信息与获得真实信息的神圣权利, \
    任何与之对抗的都是我们的敌人"
        .to_string()
}

#[async_route]
fn welcome(session: Session) -> Result<String> {
    let ip_address = session.request.get_ip();
    if ip_address != "127.0.0.1" {
        return Err(anyhow!("禁止访问"));
    }
    Ok(format!(
        "欢迎进入信息绝对安全区, 来自[{}]的朋友",
        session.request.get_ip()
    ))
}

#[async_route]
fn json(_session: Session) -> ServerResponse {
    Ok(BaseResponse::JsonResponse(
        json!({"status": true, "msg": "只身堕入极暗之永夜, 以期再世涅槃之阳光"}),
    ))
}

#[async_route]
async fn alive(session: Session) -> Result<Value> {
    session.send("test".into()).await?;
    assert_eq!(session.recv().await?.text()?, "test");
    Ok(json!({"status": true, "msg": "结束"}))
}

fn server_callback(res: Response, session: Arc<Session>) -> BoxFuture<'static, bool> {
    Box::pin(async move {
        println!("callback: {}", res.text().unwrap());
        if res.text().unwrap() == "test_end" {
            false
        } else {
            session.send("server".into()).await.unwrap();
            true
        }
    })
}

#[async_route]
async fn callback_handler(mut session: Session) -> Value {
    session.set_callback(Arc::new(server_callback));
    let session_arc = Arc::new(session);
    session_arc.listen().await.unwrap().await.unwrap();
    json!({"status": "close"})
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
            client.send("test".as_bytes().to_vec()).await?;
            client.recv().await?.json()?;
            client.close().await?;
        }
        "callback" => {
            let client = Client::connect(&format!("127.0.0.1:7076{}", args[2])).await?;
            client.send("test".as_bytes().to_vec()).await?;
            client.recv().await?.text()?;
            client.send("test".as_bytes().to_vec()).await?;
            client.recv().await?.text()?;
            client.send("test_end".as_bytes().to_vec()).await?;
            let res = client.recv().await?.json()?;
            println!("{}", res);
            client.close().await?;
        }
        "serve" => {
            let mut router = Router::new();

            router.route(RoutePath::new("/handler", RouteType::Path), handler);

            path_route!(router, "/welcome" => welcome);
            path_route!(router, "/json" => json);
            path_route!(router, "/alive" => alive);
            path_route!(router, "/callback" => callback_handler);

            let server = Server::new("0.0.0.0", 7076, router);
            server.run().await?;
        }
        _ => {
            println!("未知的指令: {}", args[1]);
        }
    }

    Ok(())
}
