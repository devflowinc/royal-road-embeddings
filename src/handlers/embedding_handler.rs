use super::{auth_handler::AuthRequired, doc_group_handler::IndexDocumentGroupRequest};
use crate::{
    data::models::DocEmbedding,
    errors::ServiceError,
    operators::{
        doc_embedding_operator::{
            create_doc_group_embedding, delete_doc_embedding_pg_query,
            upsert_doc_embedding_pg_query,
        },
        doc_group_embedding_operator::get_unique_doc_group_sizes,
        embedding_operator, parse_operator,
        qdrant_operator::delete_reinsert_doc_embedding_qdrant_query,
    },
};
use actix_rt::Arbiter;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

#[derive(Debug, Deserialize, Serialize)]
pub struct IndexDocumentRequest {
    pub doc_html: String,
    pub story_id: i64,
    pub index: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IndexDocumentResponse {
    embedding: Vec<f32>,
}

pub async fn embed_document(
    document: web::Json<IndexDocumentRequest>,
    pool: web::Data<Pool<Postgres>>,
    _: AuthRequired,
) -> Result<HttpResponse, ServiceError> {
    let pool_inner = pool.get_ref().clone();

    let doc_chunks = parse_operator::chunk_document(document.doc_html.clone());

    if doc_chunks.is_empty() {
        return Err(ServiceError::EmptyDocumentError);
    }

    let embedding = embedding_operator::get_average_embedding(doc_chunks).await?;

    let doc_embedding_to_upsert = DocEmbedding::from_details(
        None,
        document.doc_html.clone(),
        document.story_id,
        document.index,
        None,
        None,
        None,
    );

    let qdrant_point_id_to_delete =
        upsert_doc_embedding_pg_query(doc_embedding_to_upsert.clone(), pool_inner.clone()).await?;

    if qdrant_point_id_to_delete.is_some() {
        let unique_group_sizes =
            get_unique_doc_group_sizes(vec![document.story_id], pool_inner.clone()).await?;

        let pool_inner1 = pool.get_ref().clone();

        Arbiter::new().spawn(async move {
            for group_size in unique_group_sizes {
                let doc_group_to_index = IndexDocumentGroupRequest::Story {
                    story_id: document.story_id,
                    doc_group_size: group_size,
                };

                let _ = create_doc_group_embedding(doc_group_to_index, pool_inner1.clone()).await;
            }
        });
    }

    let qdrant_result = delete_reinsert_doc_embedding_qdrant_query(
        qdrant_point_id_to_delete,
        doc_embedding_to_upsert.clone(),
        embedding.clone(),
    )
    .await;

    if let Err(e) = qdrant_result {
        if qdrant_point_id_to_delete.is_some() {
            delete_doc_embedding_pg_query(doc_embedding_to_upsert.clone(), pool_inner.clone())
                .await?;
        }
        return Err(e);
    }

    Ok(HttpResponse::Ok().json(IndexDocumentResponse { embedding }))
}
