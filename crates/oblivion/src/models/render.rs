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
            Self::TextResponse(text) => {
                Ok(text.as_bytes().to_vec())
            }
            Self::JsonResponse(data) => {
                Ok(data.to_string().as_bytes().to_vec())
            }
        }
    }
}
