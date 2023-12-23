//! # Oblivion
//!
//! Oblivion 是浊莲为确保信息安全而开发的端到端加密协议，这是 Oblivion 的 Rust 实现。
//! 它在 Python 实现的基础上大大提高了 Oblivion 的安全性、稳定性和并发性。
//!
//! 由于 Oblivion 协议中要求的加密算法为 ECDHE 算法，它以高效安全密钥派生方法，使得它
//! 可以应用于信息派发和及时通讯。
pub extern crate oblivion_codegen;
pub extern crate proc_macro;
pub mod api;
pub mod exceptions;
pub mod sessions;
pub mod utils {
    pub mod decryptor;
    pub mod encryptor;
    pub mod gear;
    pub mod generator;
    pub mod parser;
}
pub mod models {
    pub mod client;
    pub mod handler;
    pub mod packet;
    pub mod render;
    pub mod router;
    pub mod server;
}

/// 绝对路由宏
///
/// 使用路由宏可以简单的实现路由：
///
/// ```rust
/// use futures::future::{BoxFuture, FutureExt};
/// use oblivion::path_route;
/// use oblivion::utils::parser::OblivionRequest;
/// use oblivion::models::render::BaseResponse;
/// use oblivion_codegen::async_route;
/// use oblivion::models::router::Router;
///
/// #[async_route]
/// fn welcome(mut req: OblivionRequest) -> BaseResponse {
///     BaseResponse::TextResponse(
///        format!("欢迎进入信息绝对安全区, 来自[{}]的朋友", req.get_ip()),
///        200,
///     )
/// }
///
/// let mut router = Router::new();
/// path_route!(&mut router, "/welcome" => welcome);
/// ```
///
/// 上面的路由将会引导路径为`/welcome`或`/welcome/`的请求。
#[macro_export]
macro_rules! path_route {
    ($router:expr, $path:expr => $handler:ident) => {{
        let mut router = $router;
        let route = $crate::models::router::Route::new($handler);
        router.regist(
            $crate::models::router::RoutePath::new($path, $crate::models::router::RouteType::Path),
            route,
        );
    }};
}

/// 起始路由宏
///
/// 使用起始路由宏可以简单的实现起始路由：
///
/// ```rust
/// use futures::future::{BoxFuture, FutureExt};
/// use oblivion::startswith_route;
/// use oblivion::utils::parser::OblivionRequest;
/// use oblivion::models::render::BaseResponse;
/// use oblivion_codegen::async_route;
/// use oblivion::models::router::Router;
///
/// #[async_route]
/// fn welcome(mut req: OblivionRequest) -> BaseResponse {
///     BaseResponse::TextResponse(
///        format!("欢迎进入信息绝对安全区, 来自[{}]的朋友", req.get_ip()),
///        200,
///     )
/// }
///
/// let mut router = Router::new();
/// startswith_route!(&mut router, "/welcome" => welcome);
/// ```
///
/// 上面的路由将会引导所有以`/welcome`起始的 Oblivion Location Path String。
#[macro_export]
macro_rules! startswith_route {
    ($router:expr, $path:expr => $handler:ident) => {{
        let mut router = $router;
        let route = $crate::models::router::Route::new($handler);
        router.regist(
            $crate::models::router::RoutePath::new(
                $path,
                $crate::models::router::RouteType::StartswithPath,
            ),
            route,
        );
    }};
}

/// 正则路由宏
///
/// 使用正则路由宏可以简单的实现正则路由：
///
/// ```rust
/// use futures::future::{BoxFuture, FutureExt};
/// use oblivion::regex_route;
/// use oblivion::utils::parser::OblivionRequest;
/// use oblivion::models::render::BaseResponse;
/// use oblivion_codegen::async_route;
/// use oblivion::models::router::Router;
///
/// #[async_route]
/// fn welcome(mut req: OblivionRequest) -> BaseResponse {
///     BaseResponse::TextResponse(
///        format!("欢迎进入信息绝对安全区, 来自[{}]的朋友", req.get_ip()),
///        200,
///     )
/// }
///
/// let mut router = Router::new();
/// regex_route!(&mut router, r"^/welcome/.*" => welcome);
/// ```
///
/// 上面的路由将会引导所有以`/welcome/`起始的 Oblivion Location Path String。
///
/// 你还可以使用`^/.*`来劫持所有路由。
#[macro_export]
macro_rules! regex_route {
    ($router:expr, $path:expr => $handler:ident) => {{
        let mut router = $router;
        let route = $crate::models::router::Route::new($handler);
        router.regist(
            $crate::models::router::RoutePath::new(
                $path,
                $crate::models::router::RouteType::RegexPath,
            ),
            route,
        );
    }};
}
