//! # Oblivion Router
use super::handler::not_found;
use super::render::Response;
use crate::utils::parser::OblivionRequest;
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Route {
    handler: fn(OblivionRequest) -> Response,
}

impl Route {
    pub fn new(handler: fn(OblivionRequest) -> Response) -> Self {
        Self { handler }
    }

    pub fn clone(&mut self) -> Self {
        Self {
            handler: self.handler.clone(),
        }
    }

    pub fn get_handler(&mut self) -> fn(OblivionRequest) -> Response {
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
            route_type,
        }
    }

    pub fn check(&mut self, olps: &str) -> Result<bool> {
        if self.route_type == RouteType::RegexPath {
            let regex = Regex::new(&self.route)?;
            Ok(regex.is_match(olps))
        } else if self.route_type == RouteType::StartswithPath {
            Ok(olps.starts_with(&self.route))
        } else {
            Ok(self.route == olps.trim_end_matches("/"))
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
        handler: fn(OblivionRequest) -> Response,
    ) -> &mut Self {
        self.routes.insert(path.clone(), Route { handler: handler });
        self
    }

    pub fn regist(&mut self, path: RoutePath, route: Route) {
        let route = route;
        self.routes.insert(path.clone(), route);
    }

    pub fn get_handler(&self, path: &str) -> Result<Route> {
        for (route_path, route) in &self.routes {
            let mut route_path = route_path.clone();
            if route_path.check(path)? {
                return Ok(route.clone());
            };
        }
        Ok(Route::new(not_found))
    }
}
