//! # Oblivion API Interface
//!
//! Oblivion provides methods for making direct GET, POST, PUT, etc. requests.
use anyhow::Result;
use serde_json::Value;

use crate::models::client::Response;

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
) -> Result<Response> {
    let session = Session::new();
    session
        .request(method.to_string(), olps.to_string(), data, file)
        .await
}

/// GET method
pub async fn get(olps: &str) -> Result<Response> {
    request("get", olps, None, None).await
}

/// POST method
pub async fn post(olps: &str, data: Value) -> Result<Response> {
    request("post", olps, Some(data), None).await
}

/// PUT method
pub async fn put(olps: &str, data: Option<Value>, file: Vec<u8>) -> Result<Response> {
    request("put", olps, data, Some(file)).await
}

#[deprecated(since = "1.0.0", note = "FORWARD method is no longer supported.")]
pub async fn forward(olps: &str, data: Option<Value>, file: Vec<u8>) -> Result<Response> {
    request("forward", olps, data, Some(file)).await
}
