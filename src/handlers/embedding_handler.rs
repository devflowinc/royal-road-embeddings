use actix_web::{web, HttpResponse};

use crate::{
    errors::ServiceError,
    operators::{embedding_operator, parse_operator, qdrant_operator}, data::models::Document,
};

use super::auth_handler::AuthRequired;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct IndexDocumentRequest {
    pub doc_html: String,
    pub story_id: u32,
    pub doc_num: String,
}

pub async fn index_document(
    document: web::Json<IndexDocumentRequest>,
    _: AuthRequired,
) -> Result<HttpResponse, ServiceError> {
    let doc_content = parse_operator::parse_html(document.doc_html.clone());
    let doc_chunks = parse_operator::chunk_document(doc_content.clone());

    let embedding = embedding_operator::get_average_embedding(doc_chunks).await?;

    qdrant_operator::upsert_document(Document::from_details(
        document.story_id,
        document.doc_num.clone(),
        embedding.clone(),
        doc_content.clone(),
    ))
    .await?;

    Ok(HttpResponse::Ok().json(embedding))
}
