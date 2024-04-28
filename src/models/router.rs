//! # Oblivion Router
use super::handler::not_found;
use super::session::Session;
use crate::types::server;
use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
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

    pub fn check(&self, olps: &str) -> Result<bool> {
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
        self.routes.insert(path.clone(), Route { handler });
        self
    }

    pub fn regist(&mut self, path: RoutePath, route: Route) {
        let route = route;
        self.routes.insert(path.clone(), route);
    }

    pub fn get_handler(&self, path: &str) -> Result<Handler> {
        let handler = self
            .routes
            .par_iter()
            .find_any(|values| {
                let (route_path, _) = values;
                route_path.check(path).unwrap_or(false)
            })
            .map(|route| route.1.get_handler());

        if let Some(route) = handler {
            Ok(route)
        } else {
            Ok(not_found)
        }
    }
}
