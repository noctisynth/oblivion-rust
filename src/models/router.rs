use std::{collections::HashMap, sync::{Arc, Mutex}};

use super::server::{Handler, NotFound};

pub struct Router {
    routes: HashMap<String, Arc<Mutex<Box<dyn Handler>>>>,
}

impl Router {
    pub fn new(routes: Option<HashMap<String, Arc<Mutex<Box<dyn Handler>>>>>) -> Self {
        let routes = if routes.is_none() {
            HashMap::new()
        } else {
            routes.unwrap()
        };
        Self { routes: routes }
    }

    pub fn add_route(&mut self, route: String, handler: impl Handler + 'static) {
        self.routes
            .insert(route, Arc::new(Mutex::new(Box::new(handler))));
    }

    pub async fn get_handler(&self, route: String) -> Arc<Mutex<Box<dyn Handler>>> {
        if let Some(handler) = self.routes.get(&route) {
            let handler = handler;
            handler.to_owned()
        } else {
            Arc::new(Mutex::new(Box::new(NotFound {})))
        }
    }
}