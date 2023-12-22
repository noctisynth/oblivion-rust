use super::handler::not_found;
use super::render::BaseResponse;
use crate::utils::parser::OblivionRequest;
use futures::future::BoxFuture;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Route {
    handler: fn(OblivionRequest) -> BoxFuture<'static, BaseResponse>,
}

impl Route {
    pub fn new(handler: fn(OblivionRequest) -> BoxFuture<'static, BaseResponse>) -> Self {
        Self { handler: handler }
    }

    pub fn clone(&mut self) -> Self {
        Self {
            handler: self.handler.clone(),
        }
    }

    pub fn get_handler(&mut self) -> fn(OblivionRequest) -> BoxFuture<'static, BaseResponse> {
        self.handler.clone()
    }
}

#[derive(Clone)]
pub struct Router {
    routes: HashMap<String, Route>,
    // not_found_route: Route,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            // not_found_route: Route { handler: not_found },
        }
    }

    pub fn route(
        &mut self,
        path: &str,
        handler: fn(OblivionRequest) -> BoxFuture<'static, BaseResponse>,
    ) {
        self.routes
            .insert(path.to_owned(), Route { handler: handler });
    }

    pub fn regist(&mut self, path: &str, route: Route) {
        let route = route;
        self.routes.insert(path.to_owned(), route);
    }

    pub async fn get_handler(&self, path: String) -> Route {
        let maybe_a_handler = self.routes.get(&path);
        if maybe_a_handler.is_none() {
            Route::new(not_found)
        } else {
            maybe_a_handler.unwrap().clone()
        }
    }
}

#[macro_export]
macro_rules! route {
    ($router:expr, $path:expr => $handler:ident) => {{
        let mut router = $router;
        let path = $path;
        let handler = $handler;
        let route = $crate::models::router::Route::new(handler);
        router.regist(path, route);
    }};
}
