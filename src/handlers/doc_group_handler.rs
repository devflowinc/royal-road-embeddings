use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::{
    errors::ServiceError, operators::qdrant_operator::create_doc_group_collection_qdrant_query,
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
