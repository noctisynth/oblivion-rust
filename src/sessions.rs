//! # Oblivion Sessions
use anyhow::Result;
use serde_json::Value;

use crate::models::client::{Client, Response};

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
        _data: Option<Value>,
        _file: Option<Vec<u8>>,
    ) -> Result<Response> {
        let mut req = Client::new(method, olps)?;
        req.prepare().await?;
        Ok(req.recv().await?)
    }
}
