use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::{data::models::DocEmbedding, data::models::DocGroupEmbedding, errors::ServiceError};

use super::qdrant_operator::QdrantPoints;

#[derive(Debug, Deserialize, Serialize)]
pub enum DocEmbeddingType {
    DocEmbedding(DocEmbedding),
    DocGroupEmbedding(DocGroupEmbedding),
}
pub async fn get_docs_by_point_id(
    points: Vec<QdrantPoints>,
    doc_group_size: Option<i32>,
    pool: Pool<Postgres>,
) -> Result<Vec<DocEmbeddingType>, ServiceError> {
    let qdrant_point_ids = points
        .iter()
        .map(|point| point.point_id)
        .collect::<Vec<uuid::Uuid>>();
    match doc_group_size {
        Some(doc_group_size) => {
            let embeds = sqlx::query_as!(
                DocGroupEmbedding,
                r#"
                SELECT *
                FROM doc_group_embeddings
                WHERE qdrant_point_id = ANY($1) AND doc_group_size = $2
                "#,
                qdrant_point_ids.as_slice(),
                doc_group_size,
            )
            .fetch_all(&pool)
            .await
            .map_err(ServiceError::PgSearchError)?;

            Ok(embeds
                .into_iter()
                .map(DocEmbeddingType::DocGroupEmbedding)
                .collect::<Vec<DocEmbeddingType>>())
        }
        None => {
            let embeds = sqlx::query_as!(
                DocEmbedding,
                r#"
                SELECT *
                FROM doc_embeddings
                WHERE qdrant_point_id = ANY($1)
                "#,
                qdrant_point_ids.as_slice(),
            )
            .fetch_all(&pool)
            .await
            .map_err(ServiceError::PgSearchError)?;
            Ok(embeds
                .into_iter()
                .map(DocEmbeddingType::DocEmbedding)
                .collect::<Vec<DocEmbeddingType>>())
        }
    }
}
