//! # Oblivion Router
use super::handler::not_found;
use super::render::Response;
use super::session::Session;
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

pub type Handler = fn(&mut Session) -> Response;

#[derive(Clone)]
pub struct Route {
    handler: Handler,
}

impl Route {
    pub fn new(handler: Handler) -> Self {
        Self { handler }
    }

    pub fn clone(&mut self) -> Self {
        Self {
            handler: self.handler.clone(),
        }
    }

    pub fn get_handler(&mut self) -> Handler {
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

    pub fn route(&mut self, path: RoutePath, handler: Handler) -> &mut Self {
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
