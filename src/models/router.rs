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
    // path: String,
    handler: fn(&mut OblivionRequest) -> BaseResponse,
}

impl Route {
    // pub fn new(path: String, handler: fn(&mut OblivionRequest) -> BaseResponse) -> Self {
    pub fn new(handler: fn(&mut OblivionRequest) -> BaseResponse) -> Self {
        Self {
            // path: path,
            handler: handler,
        }
    }

    pub fn clone(&mut self) -> Self {
        Self {
            // path: self.path.clone(),
            handler: self.handler.clone(),
        }
    }

    // pub fn get_path(&mut self) -> String {
    //     self.path.clone()
    // }

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

    pub fn route(&mut self, path: String, handler: fn(&mut OblivionRequest) -> BaseResponse) {
        self.routes.insert(
            path.clone(),
            Route {
                // path: path,
                handler: handler,
            },
        );
    }

    // pub fn regist(&mut self, route: Route) {
    //     let mut route = route;
    //     self.routes.insert(route.get_path(), route);
    // }

    pub async fn get_handler(&self, path: String) -> Route {
        let maybe_a_handler = self.routes.get(&path);
        if maybe_a_handler.is_none() {
            Route::new(not_found)
        } else {
            maybe_a_handler.unwrap().clone()
        }
    }
}

// #[macro_export]
// macro_rules! route {
//     ($path:expr => $handler:ident) => {{
//         let path = $path;
//         let handler = $handler;
//         $crate::models::router::Route {
//             // path: path.to_string(),
//             handler: handler,
//         }
//     }};
// }
