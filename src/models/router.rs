//! # Oblivion Router
use super::handler::not_found;
use super::session::Session;
use crate::types::server;
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

pub type Handler = fn(Session) -> server::Result;

#[derive(Clone)]
pub struct Route {
    handler: Handler,
}

impl Route {
    pub fn new(handler: Handler) -> Self {
        Self { handler }
    }

    #[inline]
    pub fn get_handler(&self) -> Handler {
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

    #[inline]
    pub fn check(&self, entrance: &str) -> Result<bool> {
        if self.route_type == RouteType::RegexPath {
            let regex = Regex::new(&self.route)?;
            Ok(regex.is_match(entrance))
        } else if self.route_type == RouteType::StartswithPath {
            Ok(entrance.starts_with(&self.route))
        } else {
            Ok(self.route == entrance.trim_end_matches("/"))
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
        self.routes.insert(path, Route { handler });
        self
    }

    pub fn register(&mut self, path: RoutePath, route: Route) {
        let route = route;
        self.routes.insert(path, route);
    }

    pub fn get_handler(&self, path: &str) -> Result<Handler> {
        for (route_path, route) in self.routes.iter() {
            if route_path.check(path)? {
                return Ok(route.get_handler());
            }
        }
        Ok(not_found)
    }
}
