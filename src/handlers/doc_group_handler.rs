use super::auth_handler::AuthRequired;
use crate::{
    errors::ServiceError,
    operators::{
        doc_embedding_operator::create_doc_group_embedding,
        doc_group_embedding_operator::get_doc_group_qdrant_ids_pg_query,
        qdrant_operator::{
            create_doc_group_collection_qdrant_query, recommend_group_doc_embeddings_qdrant_query,
        },
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Pool, Postgres};

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupDocumentRequest {
    pub doc_group_size: i32,
}

pub async fn create_document_group(
    group_document_request: web::Json<GroupDocumentRequest>,
    _: AuthRequired,
) -> Result<HttpResponse, ServiceError> {
    create_doc_group_collection_qdrant_query(group_document_request.doc_group_size)
        .await
        .map(|_| HttpResponse::NoContent().into())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum IndexDocumentGroupRequest {
    Stories {
        doc_group_size: i32,
        story_ids: Vec<i64>,
    },
    Story {
        story_id: i64,
        doc_group_size: i32,
    },
    All {
        doc_group_size: i32,
    },
}

pub async fn index_document_group(
    req: web::Json<IndexDocumentGroupRequest>,
    pool: web::Data<Pool<Postgres>>,
    _: AuthRequired,
) -> Result<HttpResponse, ServiceError> {
    create_doc_group_embedding(req.into_inner(), pool.get_ref().clone())
        .await
        .map(|_| HttpResponse::NoContent().into())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetDocumentGroupRequest {
    pub doc_group_size: i32,
    pub story_id: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetDocumentGroupResponse {
    pub embeddings: Vec<Vec<f32>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RecommendDocumentRequest {
    pub doc_group_size: i32,
    pub story_ids: Vec<i64>,
    pub limit: Option<u64>,
    pub page: Option<u64>,
}

pub async fn recommend_document_group(
    recommend_document_request: web::Json<RecommendDocumentRequest>,
    pool: web::Data<Pool<Postgres>>,
    _: AuthRequired,
) -> Result<HttpResponse, ServiceError> {
    let positive_qdrant_ids = get_doc_group_qdrant_ids_pg_query(
        recommend_document_request.story_ids.clone(),
        recommend_document_request.doc_group_size,
        pool.get_ref().clone(),
    )
    .await?
    .into_iter()
    .map(|doc_group_qdrant_id| doc_group_qdrant_id.qdrant_point_id)
    .collect::<Vec<uuid::Uuid>>();

    let recommended_story_ids = recommend_group_doc_embeddings_qdrant_query(
        positive_qdrant_ids,
        recommend_document_request.doc_group_size,
        recommend_document_request.limit,
        recommend_document_request.page,
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "recommended_story_ids": recommended_story_ids,
    })))
}
