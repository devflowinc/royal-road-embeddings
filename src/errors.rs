use actix_web::{http, HttpResponse, ResponseError};
use derive_more::Display;

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug, Display)]
pub enum ServiceError {
    IAMATeaPot,
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::IAMATeaPot => HttpResponse::new(http::StatusCode::IM_A_TEAPOT),
        }
    }
}
