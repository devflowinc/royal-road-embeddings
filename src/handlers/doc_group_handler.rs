use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Pool, Postgres};

use crate::{
    errors::ServiceError,
    operators::{
        doc_group_operator::get_doc_group_qdrant_ids_pg_query,
        qdrant_operator::{
            create_doc_group_collection_qdrant_query, reccomend_group_doc_embeddings_qdrant_query,
        },
    },
};

use super::auth_handler::AuthRequired;

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupDocumentRequest {
    pub doc_group_size: i32,
    pub story_id: i64,
}

pub async fn create_document_group(
    group_document_request: GroupDocumentRequest,
    pool: web::Data<Pool<Postgres>>,
    _: AuthRequired,
) -> Result<HttpResponse, ServiceError> {
    let _ = create_doc_group_collection_qdrant_query(group_document_request.doc_group_size).await;

    Ok(HttpResponse::Ok().into())
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
    .await?;

    let recommended_story_ids = reccomend_group_doc_embeddings_qdrant_query(
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
