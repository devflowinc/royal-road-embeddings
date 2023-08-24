use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug, Display)]
pub enum ServiceError {
    InvalidAPIKey,
    EmbeddingServerCallError(reqwest::Error),
    EmbeddingServerParseError(reqwest::Error),
    EmbeddingAveragingError,
    QdrantConnectionError(anyhow::Error),
    UpsertDocumentError(anyhow::Error),
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InvalidAPIKey => HttpResponse::Unauthorized().json(ErrorResponse {
                message: "Invalid API key provided.".to_string(),
            }),
            ServiceError::EmbeddingServerCallError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error calling embedding server.".to_string(),
                })
            }
            ServiceError::EmbeddingServerParseError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error parsing embedding server response.".to_string(),
                })
            }
            ServiceError::EmbeddingAveragingError => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error averaging embeddings.".to_string(),
                })
            }
            ServiceError::QdrantConnectionError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error connecting to Qdrant.".to_string(),
                })
            }
            ServiceError::UpsertDocumentError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error upserting document.".to_string(),
                })
            }
        }
    }
}
