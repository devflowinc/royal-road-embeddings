use sqlx::{Pool, Postgres};

use crate::{data::models::DocEmbedding, errors::ServiceError};

pub struct QdrantPointIdContainer {
    pub qdrant_point_id: uuid::Uuid,
}

pub async fn upsert_doc_embedding_pg_query(
    doc_embedding: DocEmbedding,
    pool: Pool<Postgres>,
) -> Result<Option<uuid::Uuid>, ServiceError> {
    // select qdrant_point_id from doc_embeddings where story_id = $1 and doc_num = $2
    let qdrant_point_id: Option<QdrantPointIdContainer> = sqlx::query_as!(
        QdrantPointIdContainer,
        r#"
        SELECT qdrant_point_id
        FROM doc_embeddings
        WHERE story_id = $1 AND doc_num = $2
        "#,
        doc_embedding.story_id,
        doc_embedding.doc_num,
    )
    .fetch_optional(&pool)
    .await
    .map_err(ServiceError::UpsertDocEmbeddingPgError)?;

    sqlx::query!(
        r#"
        INSERT INTO doc_embeddings (id, doc_html, story_id, doc_num, qdrant_point_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (story_id, doc_num) DO UPDATE
        SET
            doc_html = EXCLUDED.doc_html,
            story_id = EXCLUDED.story_id,
            doc_num = EXCLUDED.doc_num,
            qdrant_point_id = EXCLUDED.qdrant_point_id,
            updated_at = EXCLUDED.updated_at
        "#,
        doc_embedding.id,
        doc_embedding.doc_html,
        doc_embedding.story_id,
        doc_embedding.doc_num,
        doc_embedding.qdrant_point_id,
        doc_embedding.created_at,
        doc_embedding.updated_at,
    )
    .execute(&pool)
    .await
    .map_err(ServiceError::UpsertDocEmbeddingPgError)?;

    Ok(qdrant_point_id.map(|qdrant_point_id_container| qdrant_point_id_container.qdrant_point_id))
}
