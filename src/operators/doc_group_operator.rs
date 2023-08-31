use sqlx::{Pool, Postgres};

use crate::{errors::ServiceError, operators::doc_embedding_operator::QdrantPointIdContainer};

pub async fn get_doc_group_qdrant_ids_pg_query(
    story_ids: Vec<i64>,
    doc_group_size: i32,
    pool: Pool<Postgres>,
) -> Result<Vec<uuid::Uuid>, ServiceError> {
    let qdrant_point_ids = sqlx::query_as!(
        QdrantPointIdContainer,
        r#"
        SELECT qdrant_point_id
        FROM doc_embeddings
        WHERE story_id = ANY($1) AND index = $2
        "#,
        story_ids.as_slice(),
        doc_group_size as i64,
    )
    .fetch_all(&pool)
    .await
    .map_err(ServiceError::SelectDocGroupQdrantIdsPgError)?
    .into_iter()
    .map(|qdrant_point_id_container| qdrant_point_id_container.qdrant_point_id)
    .collect::<Vec<uuid::Uuid>>();

    Ok(qdrant_point_ids)
}
