//! # Oblivion Render
use anyhow::Result;
use serde_json::Value;

use crate::exceptions::Exception;

#[derive(Clone)]
pub enum BaseResponse {
    FileResponse(String, u32),
    TextResponse(String, u32),
    JsonResponse(Value, u32),
}

impl BaseResponse {
    pub fn as_bytes(&self) -> Result<Vec<u8>, Exception> {
        match self {
            Self::FileResponse(_, _) => Err(Exception::UnsupportedMethod {
                method: "FileResponse".to_string(),
            }),
            Self::TextResponse(text, _) => {
                Ok(text.as_bytes().to_vec())
            }
            Self::JsonResponse(data, _) => {
                Ok(data.to_string().as_bytes().to_vec())
            }
        }
    }

    pub fn get_status_code(&self) -> Result<u32, Exception> {
        match self {
            Self::FileResponse(_, _) => Err(Exception::UnsupportedMethod {
                method: "FileResponse".to_string(),
            }),
            Self::TextResponse(_, status_code) => {
                Ok(*status_code)
            }
            Self::JsonResponse(_, status_code) => {
                Ok(*status_code)
            }
        }
    }
}
