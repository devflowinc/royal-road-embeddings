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
            ServiceError::UpsertDocEmbeddingQdrantError(err) => {
                HttpResponse::InternalServerError()
                            .json(ErrorResponse {
                                message: "Error upserting DocEmbedding to Qdrant.".to_string(),
                                error_code: "0006".to_string(),
                            })
            },
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
            ServiceError::SelectDocGroupQdrantIdsPgError(_) => HttpResponse::InternalServerError()
                .json(ErrorResponse {
                    message: "Error selecting DocGroup Qdrant IDs from Postgres.".to_string(),
                    error_code: "0009".to_string(),
                }),
            ServiceError::RecommendQdrantDocEmbeddingGroupError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error recommending DocEmbedding group from Qdrant.".to_string(),
                    error_code: "0010".to_string(),
                })
            }
            ServiceError::GetDocEmbeddingsPgError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error getting DocEmbeddings from Postgres.".to_string(),
                    error_code: "0011".to_string(),
                })
            }
            ServiceError::ScrollDocEmbeddingQdrantError(_) => HttpResponse::InternalServerError()
                .json(ErrorResponse {
                    message: "Error getting DocEmbeddings from Qdrant.".to_string(),
                    error_code: "0012".to_string(),
                }),
            ServiceError::VectorToArrayError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error converting vector to array.".to_string(),
                    error_code: "0013".to_string(),
                })
            }
            ServiceError::UpsertDocGroupEmbeddingPgError(_) => HttpResponse::InternalServerError()
                .json(ErrorResponse {
                    message: "Error upserting DocGroupEmbedding to Postgres.".to_string(),
                    error_code: "0014".to_string(),
                }),
            ServiceError::QdrantSearchError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error searching Qdrant.".to_string(),
                    error_code: "0015".to_string(),
                })
            }
            ServiceError::PgSearchError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error searching Postgres.".to_string(),
                    error_code: "0016".to_string(),
                })
            }
            ServiceError::UpsertDocGroupEmbeddingQdrantError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error upserting DocGroupEmbedding to Qdrant.".to_string(),
                    error_code: "0017".to_string(),
                })
            }
            ServiceError::InsertDocGroupEmbeddingPgError(_) => HttpResponse::InternalServerError()
                .json(ErrorResponse {
                    message: "Error inserting DocGroupEmbedding to Postgres.".to_string(),
                    error_code: "0018".to_string(),
                }),
            ServiceError::QdrantSimilarityTopFilteredPointError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error getting similarity to single vector from Qdrant.".to_string(),
                    error_code: "0018".to_string(),
                })
            }
            ServiceError::MatchingRecordNotFound => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Matching record not found.".to_string(),
                    error_code: "0019".to_string(),
                })
            }
            ServiceError::SelectUniqueDocGroupSizesPgError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error selecting unique DocGroup sizes from Postgres.".to_string(),
                    error_code: "0020".to_string(),
                })
            }
            ServiceError::SelectDocEmbeddingsQdrantIdsPgError(_) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Error selecting DocEmbedding Qdrant IDs from Postgres.".to_string(),
                    error_code: "0021".to_string(),
                })
            }
            ServiceError::CreateEmbeddingServerError(_) => HttpResponse::InternalServerError()
                .json(ErrorResponse {
                    message: "Error selecting DocEmbedding Qdrant IDs from Postgres.".to_string(),
                    error_code: "0022".to_string(),
                }),
        }
    }
}
