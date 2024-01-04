//! # Oblivion Sessions
use serde_json::Value;

use crate::{
    exceptions::OblivionException,
    models::client::{Request, Response},
};

/// ## Oblivion Abstract Session
///
/// Used to connect to the model and create a request session.
pub struct Session;

impl Session {
    pub fn new() -> Self {
        Self
    }

    pub async fn request(
        &self,
        method: String,
        olps: String,
        data: Option<Value>,
        file: Option<Vec<u8>>,
        tfo: bool,
    ) -> Result<Response, OblivionException> {
        let mut req = Request::new(method, olps, data, file, tfo)?;
        req.prepare().await?;
        Ok(self.send(&mut req).await?)
    }

    pub async fn send(&self, request: &mut Request) -> Result<Response, OblivionException> {
        if request.is_prepared() != true {
            let _ = request.prepare();
        }

        request.send().await?;
        request.recv().await
    }
}
