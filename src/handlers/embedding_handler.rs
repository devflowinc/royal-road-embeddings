use actix_web::{HttpResponse, web};

use crate::{errors::ServiceError, operators::{embedding_operator, parse_operator}};

use super::auth_handler::AuthRequired;


#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct IndexDocumentRequest {
    pub doc_html: String,
    pub story_id: u32,
    pub doc_num: String,
}

pub async fn index_document(document: web::Json<IndexDocumentRequest>, _: AuthRequired) -> Result<HttpResponse, ServiceError> {
    let doc_content = parse_operator::parse_html(document.doc_html.clone());

    let embedding = embedding_operator::create_embedding(&doc_content).await?;

    Ok(HttpResponse::Ok().json(embedding))
}
