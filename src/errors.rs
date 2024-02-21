use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::io;

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
    SelectDocGroupQdrantIdsPgError(sqlx::Error),
    RecommendQdrantDocEmbeddingGroupError(anyhow::Error),
    GetDocEmbeddingsPgError(sqlx::Error),
    ScrollDocEmbeddingQdrantError(anyhow::Error),
    NotImplemented,
    VectorToArrayError(ndarray::ShapeError),
    UpsertDocGroupEmbeddingPgError(sqlx::Error),
    QdrantSearchError(anyhow::Error),
    PgSearchError(sqlx::Error),
    UpsertDocGroupEmbeddingQdrantError(anyhow::Error),
    InsertDocGroupEmbeddingPgError(sqlx::Error),
    QdrantSimilarityTopFilteredPointError(anyhow::Error),
    MatchingRecordNotFound,
    SelectUniqueDocGroupSizesPgError(sqlx::Error),
    SelectDocEmbeddingsQdrantIdsPgError(sqlx::Error),
    CreateEmbeddingServerError(async_openai::error::OpenAIError),
    ParseDocumentCallError(io::Error),
    ParseDocumentResponseError,
    ChunkDocumentError,
    EmptyDocumentError,
    CreateTmpFileError(io::Error),
    DeleteTmpFileError(io::Error),
    DeleteDocEmbeddingError(sqlx::Error),
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::NotImplemented => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Not implemented.".to_string(),
                    error_code: "0420".to_string(),
                })
            }
            ServiceError::InvalidAPIKey => HttpResponse::Unauthorized().json(ErrorResponse {
                message: "Invalid API key provided.".to_string(),
                error_code: "0001".to_string(),
            }),
            ServiceError::EmbeddingServerCallError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!("Error calling embedding server: {:?}", e),
                    error_code: "0002".to_string(),
                })
            }
            ServiceError::EmbeddingServerParseError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!("Error parsing embedding server response: {:?}", e),
                    error_code: "0003".to_string(),
                })
            }
            ServiceError::EmbeddingAveragingError => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error averaging embeddings.".to_string(),
                    error_code: "0004".to_string(),
                })
            }
            ServiceError::QdrantConnectionError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!("Error connecting to Qdrant: {:?}", e),
                    error_code: "0005".to_string(),
                })
            }
            ServiceError::UpsertDocEmbeddingQdrantError(e) => HttpResponse::InternalServerError()
                .json(ErrorResponse {
                    message: format!("Error upserting DocEmbedding to Qdrant: {:?}", e),
                    error_code: "0006".to_string(),
                }),
            ServiceError::DeleteDocEmbeddingQdrantError(e) => HttpResponse::InternalServerError()
                .json(ErrorResponse {
                    message: format!("Error deleting DocEmbedding from Qdrant: {:?}", e),
                    error_code: "0007".to_string(),
                }),
            ServiceError::UpsertDocEmbeddingPgError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!("Error upserting DocEmbedding to Postgres: {:?}", e),
                    error_code: "0008".to_string(),
                })
            }
            ServiceError::SelectDocGroupQdrantIdsPgError(e) => HttpResponse::InternalServerError()
                .json(ErrorResponse {
                    message: format!("Error selecting DocGroup Qdrant IDs from Postgres: {:?}", e),
                    error_code: "0009".to_string(),
                }),
            ServiceError::RecommendQdrantDocEmbeddingGroupError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!("Error recommending DocEmbedding group from Qdrant: {:?}", e),
                    error_code: "0010".to_string(),
                })
            }
            ServiceError::GetDocEmbeddingsPgError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!("Error getting DocEmbeddings from Postgres: {:?}", e),
                    error_code: "0011".to_string(),
                })
            }
            ServiceError::ScrollDocEmbeddingQdrantError(e) => HttpResponse::InternalServerError()
                .json(ErrorResponse {
                    message: format!("Error scrolling DocEmbeddings from Qdrant: {:?}", e),
                    error_code: "0012".to_string(),
                }),
            ServiceError::VectorToArrayError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!("Error converting vector to array: {:?}", e),
                    error_code: "0013".to_string(),
                })
            }
            ServiceError::UpsertDocGroupEmbeddingPgError(e) => HttpResponse::InternalServerError()
                .json(ErrorResponse {
                    message: format!("Error upserting DocGroupEmbedding to Postgres: {:?}", e),
                    error_code: "0014".to_string(),
                }),
            ServiceError::QdrantSearchError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!("Error searching Qdrant: {:?}", e),
                    error_code: "0015".to_string(),
                })
            }
            ServiceError::PgSearchError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!("Error searching Postgres: {:?}", e),
                    error_code: "0016".to_string(),
                })
            }
            ServiceError::UpsertDocGroupEmbeddingQdrantError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!("Error upserting DocGroupEmbedding to Qdrant: {:?}", e),
                    error_code: "0017".to_string(),
                })
            }
            ServiceError::InsertDocGroupEmbeddingPgError(e) => HttpResponse::InternalServerError()
                .json(ErrorResponse {
                    message: format!("Error inserting DocGroupEmbedding to Postgres: {:?}", e),
                    error_code: "0018".to_string(),
                }),
            ServiceError::QdrantSimilarityTopFilteredPointError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!("Error getting top filtered points from Qdrant: {:?}", e),
                    error_code: "0018".to_string(),
                })
            }
            ServiceError::MatchingRecordNotFound => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Matching record not found.".to_string(),
                    error_code: "0019".to_string(),
                })
            }
            ServiceError::SelectUniqueDocGroupSizesPgError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!(
                        "Error selecting unique DocGroup sizes from Postgres: {:?}",
                        e
                    ),
                    error_code: "0020".to_string(),
                })
            }
            ServiceError::SelectDocEmbeddingsQdrantIdsPgError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!(
                        "Error selecting DocEmbeddings Qdrant IDs from Postgres: {:?}",
                        e
                    ),
                    error_code: "0021".to_string(),
                })
            }
            ServiceError::CreateEmbeddingServerError(e) => HttpResponse::InternalServerError()
                .json(ErrorResponse {
                    message: format!("Error creating embedding server: {:?}", e),
                    error_code: "0022".to_string(),
                }),
            ServiceError::ParseDocumentCallError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!("Error calling Parse Document Command: {:?}", e),
                    error_code: "0023".to_string(),
                })
            }
            ServiceError::ParseDocumentResponseError => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error Parsing Response from Parse Document Command".to_string(),
                    error_code: "0024".to_string(),
                })
            }
            ServiceError::ChunkDocumentError => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error Chunking Document".to_string(),
                    error_code: "0025".to_string(),
                })
            }
            ServiceError::EmptyDocumentError => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Empty Document".to_string(),
                    error_code: "0026".to_string(),
                })
            }
            ServiceError::CreateTmpFileError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!("Error Creating Temporary File: {:?}", e),
                    error_code: "0027".to_string(),
                })
            }
            ServiceError::DeleteTmpFileError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!("Error Deleting Temporary File: {:?}", e),
                    error_code: "0028".to_string(),
                })
            }
            ServiceError::DeleteDocEmbeddingError(e) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: format!("Error Deleting DocEmbedding: {:?}", e),
                    error_code: "0029".to_string(),
                })
            }
        }
    }
}
