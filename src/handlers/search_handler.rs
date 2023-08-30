use actix_web::{web, HttpResponse};
use sqlx::{Pool, Postgres};

use crate::{
    errors::ServiceError,
    operators::{embedding_operator, qdrant_operator, search_operator},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchRequest {
    pub doc_group_size: Option<i32>,
    pub page: u64,
    pub query: String,
}
pub async fn semantic_search(
    group_document_request: web::Json<SearchRequest>,
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
