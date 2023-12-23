//! # Oblivion 窗口
use serde_json::Value;

use crate::{
    exceptions::OblivionException,
    models::client::{Request, Response},
};

/// ## Oblivion 窗口抽象类
///
/// 用于连接模型创建请求窗口。
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

        // 发送请求
        request.send().await?;
        request.recv().await
    }
}
