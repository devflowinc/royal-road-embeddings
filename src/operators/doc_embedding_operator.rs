use sqlx::{Pool, Postgres};

use crate::{data::models::DocEmbedding, errors::ServiceError};

pub async fn upsert_doc_embedding_pg_query(
    doc_embedding: DocEmbedding,
    pool: Pool<Postgres>,
) -> Result<(), ServiceError> {
    sqlx::query!(
        r#"
        INSERT INTO doc_embeddings (id, doc_html, story_id, doc_num, qdrant_point_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id) DO UPDATE
        SET
            doc_html = EXCLUDED.doc_html,
            story_id = EXCLUDED.story_id,
            doc_num = EXCLUDED.doc_num,
            qdrant_point_id = EXCLUDED.qdrant_point_id,
            created_at = EXCLUDED.created_at,
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

    Ok(())
}
