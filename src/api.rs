//! # Oblivion API 接口
//!
//! Oblivion 提供了直接进行 GET、POST、PUT 等请求的方法。
use serde_json::Value;

use crate::{exceptions::OblivionException, models::client::Response};

use super::sessions::Session;

/// 裸 Oblivion 请求模式
///
/// ```rust
/// use oblivion::api::request;
///
/// async fn run() {
///     request("get", "127.0.0.1:813/get", None, None, true).await.unwrap();
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

pub async fn post(
    olps: &str,
    data: Option<Value>,
    tfo: bool,
) -> Result<Response, OblivionException> {
    request("post", olps, data, None, tfo).await
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
