use serde_json::Value;

use crate::exceptions::OblivionException;

pub enum BaseResponse {
    FileResponse(String, i32),
    TextResponse(String, i32),
    JsonResponse(Value, i32),
}

pub struct FileResponse {}

pub struct TextResponse {
    status_code: i32,
    text: String,
}

impl TextResponse {
    pub fn new(text: &str, status_code: i32) -> Result<Self, OblivionException> {
        Ok(Self {
            status_code: status_code,
            text: text.to_string(),
        })
    }

    pub fn as_bytes(&mut self) -> Vec<u8> {
        self.text.as_bytes().to_vec()
    }

    pub fn get_status_code(&mut self) -> i32 {
        self.status_code.clone()
    }
}

pub struct JsonResponse {
    data: Value,
    status_code: i32,
}

impl JsonResponse {
    pub fn new(data: Value, status_code: i32) -> Result<Self, OblivionException> {
        Ok(Self {
            data: data,
            status_code: status_code,
        })
    }

    pub fn as_bytes(&mut self) -> Vec<u8> {
        self.data.to_string().as_bytes().to_vec()
    }

    pub fn get_status_code(&mut self) -> i32 {
        self.status_code.clone()
    }
}

impl BaseResponse {
    pub fn as_bytes(&mut self) -> Result<Vec<u8>, OblivionException> {
        match self {
            Self::FileResponse(_, _) => Err(OblivionException::UnsupportedMethod(None)),
            Self::TextResponse(text, status_code) => {
                let mut tres = TextResponse::new(&text, *status_code)?;
                Ok(tres.as_bytes())
            }
            Self::JsonResponse(data, status_code) => {
                let mut jres = JsonResponse::new(data.clone(), *status_code)?;
                Ok(jres.as_bytes())
            }
        }
    }

    pub fn get_status_code(&mut self) -> Result<i32, OblivionException> {
        match self {
            Self::FileResponse(_, _) => Err(OblivionException::UnsupportedMethod(None)),
            Self::TextResponse(text, status_code) => {
                let mut tres = TextResponse::new(&text, *status_code)?;
                Ok(tres.get_status_code())
            }
            Self::JsonResponse(data, status_code) => {
                let mut jres = JsonResponse::new(data.clone(), *status_code)?;
                Ok(jres.get_status_code())
            }
        }
    }
}
