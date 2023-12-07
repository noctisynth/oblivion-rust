<div align="center">
    <img src="./static/favicon.png" alt="未知访客" width="200" height="200"></img>
</div>

<div align="center">

# Oblivion

Rust 的 Oblivion 协议实现

</div>

## 客户端

### Oblivion `client`

```rust
use oblivion::models::client;
```

### Oblivion `api`

```rust
use oblivion::api::get;
use oblivion::api::post;
use oblivion::api::put;
use oblivion::api::forward; // 弃用

let req = get("127.0.0.1:813/path", true).await.unwrap(); //  返回一个Response结构体
println!("{}", req.text()); // 输出GET内容
```

### 结构体

```rust
pub struct Response {
    header: String,
    content: Vec<u8>,
    olps: String,
    status_code: i32,
}

impl Response {
    // ...

    pub fn ok(&self) -> bool {
        self.status_code < 400
    }

    pub fn text(&mut self) -> String {
        String::from_utf8(self.content.to_vec()).expect("Unable to decode.")
    }

    pub fn json(&mut self) -> Result<Value, serde_json::Error> {
        from_str::<Value>(&to_string(&self.content)?)
    }
}
```

## 服务端

### Oblivion `router`、`render` 与 `server`

```rust
use oblivion::models::server;
use oblivion::models::render;
use oblivion::models::router;

// 创建空的初始化路由
let mut router = Router::new(Some(HashMap::new()));

// 注册路由
// 以下两种方案是等效的
router.route("/test1", test2);
route!(&mut router, "/test2" => test2);

// 创建服务器
let mut server = server::Server::new("127.0.0.1", 813, router);

// 异步启动服务器
server.run().await;
```
