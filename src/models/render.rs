//! # Oblivion Render
use anyhow::Result;
use futures::future::BoxFuture;
use serde_json::Value;

use crate::exceptions::Exception;

#[derive(Clone)]
pub enum BaseResponse {
    FileResponse(String, u32),
    TextResponse(String, u32),
    JsonResponse(Value, u32),
}

pub type Response = BoxFuture<'static, Result<BaseResponse>>;

pub struct FileResponse {}

pub struct TextResponse {
    status_code: u32,
    text: String,
}

impl TextResponse {
    pub fn new(text: &str, status_code: u32) -> Self {
        Self {
            status_code,
            text: text.to_string(),
        }
    }

    pub fn as_bytes(&mut self) -> Vec<u8> {
        self.text.as_bytes().to_vec()
    }

    pub fn get_status_code(&mut self) -> u32 {
        self.status_code
    }
}

pub struct JsonResponse {
    data: Value,
    status_code: u32,
}

impl JsonResponse {
    pub fn new(data: Value, status_code: u32) -> Self {
        Self { data, status_code }
    }

    pub fn as_bytes(&mut self) -> Vec<u8> {
        self.data.to_string().as_bytes().to_vec()
    }

    pub fn get_status_code(&mut self) -> u32 {
        self.status_code
    }
}

impl BaseResponse {
    pub fn as_bytes(&mut self) -> Result<Vec<u8>, Exception> {
        match self {
            Self::FileResponse(_, _) => Err(Exception::UnsupportedMethod {
                method: "FileResponse".to_string(),
            }),
            Self::TextResponse(text, status_code) => {
                let mut tres = TextResponse::new(&text, *status_code);
                Ok(tres.as_bytes())
            }
            Self::JsonResponse(data, status_code) => {
                let mut jres = JsonResponse::new(data.clone(), *status_code);
                Ok(jres.as_bytes())
            }
        }
    }

    pub fn get_status_code(&mut self) -> Result<u32, Exception> {
        match self {
            Self::FileResponse(_, _) => Err(Exception::UnsupportedMethod {
                method: "FileResponse".to_string(),
            }),
            Self::TextResponse(text, status_code) => {
                let mut tres = TextResponse::new(&text, *status_code);
                Ok(tres.get_status_code())
            }
            Self::JsonResponse(data, status_code) => {
                let mut jres = JsonResponse::new(data.clone(), *status_code);
                Ok(jres.get_status_code())
            }
        }
    }
}
