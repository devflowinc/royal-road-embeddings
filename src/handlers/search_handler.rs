use super::auth_handler::AuthRequired;
use crate::{
    errors::ServiceError,
    operators::{embedding_operator, qdrant_operator, search_operator},
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Pool, Postgres};

#[derive(Debug, Deserialize, Serialize)]
pub struct SemanticSearchRequest {
    pub doc_group_size: Option<i32>,
    pub page: u64,
    pub query: String,
}

pub async fn semantic_search(
    group_document_request: web::Json<SemanticSearchRequest>,
    pool: web::Data<Pool<Postgres>>,
) -> Result<HttpResponse, ServiceError> {
    /*
       Step 1: Create an embedding for query from microservice
       Step 2: Pass embedding to Qdrant
       Step 3: Get the top N of M doc_group_size results from Qdrant that are grouped by story id
       Step 4: Join with postgres to get the full document
       Step 5: Return the results
    */
    let embedding =
        embedding_operator::create_embedding(group_document_request.query.clone()).await?;
    let point_ids = qdrant_operator::search_qdrant_query(
        embedding,
        group_document_request.page,
        group_document_request.doc_group_size,
    )
    .await?;

    let results = search_operator::get_docs_by_point_id(
        point_ids,
        group_document_request.doc_group_size,
        pool.get_ref().clone(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(results))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SimilarityToSingleVectorRequest {
    pub query: String,
    pub doc_group_size: Option<i32>,
    pub index: i64,
    pub story_id: i64,
}

pub async fn similarity_to_single_vector(
    similarity_to_single_vector_request: web::Json<SimilarityToSingleVectorRequest>,
    _auth_required: AuthRequired,
) -> Result<HttpResponse, ServiceError> {
    let query_embedding =
        embedding_operator::create_embedding(similarity_to_single_vector_request.query.clone())
            .await?;

    let similarity = qdrant_operator::similarity_top_filtered_point(
        query_embedding,
        similarity_to_single_vector_request.story_id,
        similarity_to_single_vector_request.index,
        similarity_to_single_vector_request.doc_group_size,
    )
    .await?;

    match similarity {
        Some(similarity) => Ok(HttpResponse::Ok().json(json!({ "similarity": similarity }))),
        None => Err(ServiceError::MatchingRecordNotFound),
    }
}
