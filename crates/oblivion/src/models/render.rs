//! # Oblivion Render
use anyhow::Result;
use serde_json::Value;

use crate::exceptions::Exception;

#[derive(Clone)]
pub enum BaseResponse {
    FileResponse(String),
    TextResponse(String),
    JsonResponse(Value),
}

impl BaseResponse {
    pub fn as_bytes(&self) -> Result<Vec<u8>, Exception> {
        match self {
            Self::FileResponse(_) => Err(Exception::UnsupportedMethod {
                method: "FileResponse".to_string(),
            }),
            Self::TextResponse(text) => Ok(text.as_bytes().to_vec()),
            Self::JsonResponse(data) => Ok(data.to_string().as_bytes().to_vec()),
        }
    }
}

impl TryInto<Vec<u8>> for BaseResponse {
    type Error = Exception;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        self.as_bytes().map(|bytes| bytes.to_vec())
    }
}

impl From<&str> for BaseResponse {
    fn from(text: &str) -> Self {
        Self::TextResponse(text.to_string())
    }
}

impl From<String> for BaseResponse {
    fn from(text: String) -> Self {
        Self::TextResponse(text)
    }
}

impl From<Value> for BaseResponse {
    fn from(data: Value) -> Self {
        Self::JsonResponse(data)
    }
}
