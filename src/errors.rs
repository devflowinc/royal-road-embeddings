use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
    pub error_code: String,
}

#[derive(Debug, Display)]
pub enum ServiceError {
    InvalidAPIKey,
    EmbeddingServerCallError(reqwest::Error),
    EmbeddingServerParseError(reqwest::Error),
    EmbeddingAveragingError,
    QdrantConnectionError(anyhow::Error),
    UpsertDocEmbeddingQdrantError(anyhow::Error),
    DeleteDocEmbeddingQdrantError(anyhow::Error),
    UpsertDocEmbeddingPgError(sqlx::Error),
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InvalidAPIKey => HttpResponse::Unauthorized().json(ErrorResponse {
                message: "Invalid API key provided.".to_string(),
                error_code: "0001".to_string(),
            }),
            ServiceError::EmbeddingServerCallError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error calling embedding server.".to_string(),
                    error_code: "0002".to_string(),
                })
            }
            ServiceError::EmbeddingServerParseError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error parsing embedding server response.".to_string(),
                    error_code: "0003".to_string(),
                })
            }
            ServiceError::EmbeddingAveragingError => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error averaging embeddings.".to_string(),
                    error_code: "0004".to_string(),
                })
            }
            ServiceError::QdrantConnectionError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error connecting to Qdrant.".to_string(),
                    error_code: "0005".to_string(),
                })
            }
            ServiceError::UpsertDocEmbeddingQdrantError(_) => HttpResponse::InternalServerError()
                .json(ErrorResponse {
                    message: "Error upserting DocEmbedding to Qdrant.".to_string(),
                    error_code: "0006".to_string(),
                }),
            ServiceError::DeleteDocEmbeddingQdrantError(_) => HttpResponse::InternalServerError()
                .json(ErrorResponse {
                    message: "Error deleting DocEmbedding from Qdrant.".to_string(),
                    error_code: "0007".to_string(),
                }),
            ServiceError::UpsertDocEmbeddingPgError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error upserting DocEmbedding to Postgres.".to_string(),
                    error_code: "0008".to_string(),
                })
            }
        }
    }
}
