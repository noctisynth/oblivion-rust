//! # Oblivion API Interface
//!
//! Oblivion provides methods for making direct GET, POST, PUT, etc. requests.
use serde_json::Value;

use crate::{exceptions::OblivionException, models::client::Response};

use super::sessions::Session;

/// Naked Oblivion Request Mode
///
/// ```rust
/// use oblivion::api::request;
/// use oblivion::models::client::Response;
/// use oblivion::exceptions::OblivionException;
///
/// #[tokio::test]
/// async fn run() -> Result<Response, OblivionException> {
///     request("get", "127.0.0.1:813/get", None, None, true).await
/// }
/// ```
pub async fn request(
    method: &str,
    olps: &str,
    data: Option<Value>,
    file: Option<Vec<u8>>,
    tfo: bool,
) -> Result<Response, OblivionException> {
    let session = Session::new();
    session
        .request(method.to_string(), olps.to_string(), data, file, tfo)
        .await
}

pub async fn get(olps: &str, tfo: bool) -> Result<Response, OblivionException> {
    request("get", olps, None, None, tfo).await
}

pub async fn post(olps: &str, data: Value, tfo: bool) -> Result<Response, OblivionException> {
    request("post", olps, Some(data), None, tfo).await
}

pub async fn put(
    olps: &str,
    data: Option<Value>,
    file: Vec<u8>,
    tfo: bool,
) -> Result<Response, OblivionException> {
    request("put", olps, data, Some(file), tfo).await
}

pub async fn forward(
    olps: &str,
    data: Option<Value>,
    file: Vec<u8>,
    tfo: bool,
) -> Result<Response, OblivionException> {
    request("forward", olps, data, Some(file), tfo).await
}
