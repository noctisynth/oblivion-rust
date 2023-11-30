use std::collections::HashMap;

use crate::utils::parser::OblivionRequest;

use super::render::BaseResponse;

fn not_found(request: &mut OblivionRequest) -> BaseResponse {
    BaseResponse::TextResponse(
        format!(
            "Path {} is not found, error with code 404.",
            request.get_olps()
        ),
        404,
    )
}

#[derive(Clone)]
pub struct Route {
    handler: fn(&mut OblivionRequest) -> BaseResponse,
}

impl Route {
    pub fn new(handler: fn(&mut OblivionRequest) -> BaseResponse) -> Self {
        Self { handler: handler }
    }

    pub fn clone(&mut self) -> Self {
        Self {
            handler: self.handler.clone(),
        }
    }

    pub fn get_handler(&mut self) -> fn(&mut OblivionRequest) -> BaseResponse {
        self.handler.clone()
    }
}

#[derive(Clone)]
pub struct Router {
    routes: HashMap<String, Route>,
}

impl Router {
    pub fn new(routes: Option<HashMap<String, Route>>) -> Self {
        let routes = if routes.is_none() {
            HashMap::new()
        } else {
            routes.unwrap()
        };
        Self { routes: routes }
    }

    pub fn route(&mut self, path: &str, handler: fn(&mut OblivionRequest) -> BaseResponse) {
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
