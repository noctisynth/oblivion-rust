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
