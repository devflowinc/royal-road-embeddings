use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::{
    data::models::DocEmbedding,
    errors::ServiceError,
    operators::{
        doc_embedding_operator::upsert_doc_embedding_pg_query, embedding_operator, parse_operator,
        qdrant_operator::upsert_doc_embedding_qdrant_query,
    },
};

use super::auth_handler::AuthRequired;

#[derive(Debug, Deserialize, Serialize)]
pub struct IndexDocumentRequest {
    pub doc_html: String,
    pub story_id: i64,
    pub doc_num: i64,
}

pub async fn index_document(
    document: web::Json<IndexDocumentRequest>,
    pool: web::Data<Pool<Postgres>>,
    _: AuthRequired,
) -> Result<HttpResponse, ServiceError> {
    let pool_inner = pool.get_ref().clone();

    let doc_content = parse_operator::parse_html(document.doc_html.clone());
    let doc_chunks = parse_operator::chunk_document(doc_content.clone());

    let embedding = embedding_operator::get_average_embedding(doc_chunks).await?;

    let doc_embedding_to_upsert = DocEmbedding::from_details(
        None,
        document.doc_html.clone(),
        document.story_id,
        document.doc_num.clone(),
        None,
        None,
        None,
    );

    upsert_doc_embedding_pg_query(doc_embedding_to_upsert.clone(), pool_inner).await?;

    upsert_doc_embedding_qdrant_query(doc_embedding_to_upsert.clone(), embedding.clone()).await?;

    Ok(HttpResponse::Ok().json(embedding))
}
