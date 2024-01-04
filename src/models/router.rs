//! Oblivion Router
use super::handler::not_found;
use super::render::BaseResponse;
use crate::utils::parser::OblivionRequest;
use futures::future::BoxFuture;
use regex::Regex;
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

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum RouteType {
    Path,
    StartswithPath,
    RegexPath,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RoutePath {
    route: String,
    route_type: RouteType,
}

impl RoutePath {
    pub fn new(route: &str, route_type: RouteType) -> Self {
        Self {
            route: route.trim_end_matches("/").to_string(),
            route_type: route_type,
        }
    }

    pub fn check(&mut self, olps: String) -> bool {
        if self.route_type == RouteType::RegexPath {
            let regex = Regex::new(&self.route).unwrap();
            regex.is_match(&olps)
        } else if self.route_type == RouteType::StartswithPath {
            olps.starts_with(&self.route)
        } else {
            self.route == olps.trim_end_matches("/")
        }
    }
}

#[derive(Clone)]
pub struct Router {
    routes: HashMap<RoutePath, Route>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn route(
        &mut self,
        path: RoutePath,
        handler: fn(OblivionRequest) -> BoxFuture<'static, BaseResponse>,
    ) -> &mut Self {
        self.routes.insert(path.clone(), Route { handler: handler });
        self
    }

    pub fn regist(&mut self, path: RoutePath, route: Route) {
        let route = route;
        self.routes.insert(path.clone(), route);
    }

    pub fn get_handler(&self, path: String) -> Route {
        for (route_path, route) in &self.routes {
            let mut route_path = route_path.clone();
            if route_path.check(path.clone()) {
                return route.clone();
            };
        }
        Route::new(not_found)
    }
}
