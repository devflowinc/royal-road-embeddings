use super::qdrant_operator::get_doc_embeddings_qdrant_query;
use crate::{data::models::DocGroupEmbedding, errors::ServiceError};
use sqlx::{Pool, Postgres};

pub async fn get_single_vectors_to_re_average(
    story_id: i64,
    doc_group_size: i32,
    group_index: i32,
    pool: Pool<Postgres>,
) -> Result<Vec<Vec<f32>>, ServiceError> {
    let qdrant_point_uuids = sqlx::query!(
        r#"
        SELECT qdrant_point_id
        FROM doc_embeddings
        WHERE story_id = $1 AND index >= $2 AND index < $3
        "#,
        story_id,
        group_index * doc_group_size,
        group_index * doc_group_size + doc_group_size - 1,
    )
    .fetch_all(&pool)
    .await
    .map_err(ServiceError::SelectDocEmbeddingsQdrantIdsPgError)?
    .into_iter()
    .map(|embedding_container| embedding_container.qdrant_point_id)
    .collect::<Vec<uuid::Uuid>>();

    let qdrant_vectors = get_doc_embeddings_qdrant_query(qdrant_point_uuids).await?;

    Ok(qdrant_vectors)
}

#[derive(Clone)]
pub struct DocGroupQdrantPointIdContainer {
    pub qdrant_point_id: uuid::Uuid,
    pub doc_group_size: i32,
    pub index: i32,
}

pub async fn get_doc_group_qdrant_ids_pg_query(
    story_ids: Vec<i64>,
    doc_group_size: i32,
    pool: Pool<Postgres>,
) -> Result<Vec<DocGroupQdrantPointIdContainer>, ServiceError> {
    let doc_group_qdrant_point_ids = sqlx::query_as!(
        DocGroupQdrantPointIdContainer,
        r#"
        SELECT qdrant_point_id, doc_group_size, index
        FROM doc_group_embeddings
        WHERE story_id = ANY($1) AND doc_group_size = $2
        "#,
        story_ids.as_slice(),
        doc_group_size as i64,
    )
    .fetch_all(&pool)
    .await
    .map_err(ServiceError::SelectDocGroupQdrantIdsPgError)?;

    Ok(doc_group_qdrant_point_ids)
}

pub async fn get_indexed_doc_group_qdrant_ids_pg_query(
    story_ids: Vec<i64>,
    doc_group_size: i32,
    indices: Vec<i32>,
    pool: Pool<Postgres>,
) -> Result<Vec<DocGroupQdrantPointIdContainer>, ServiceError> {
    let doc_group_qdrant_point_ids = sqlx::query_as!(
        DocGroupQdrantPointIdContainer,
        r#"
        SELECT qdrant_point_id, doc_group_size, index
        FROM doc_group_embeddings
        WHERE story_id = ANY($1) AND doc_group_size = $2 AND index = ANY($3)
        "#,
        story_ids.as_slice(),
        doc_group_size as i64,
        indices.as_slice(),
    )
    .fetch_all(&pool)
    .await
    .map_err(ServiceError::SelectDocGroupQdrantIdsPgError)?;

    Ok(doc_group_qdrant_point_ids)
}

pub async fn get_unique_doc_group_sizes(
    story_ids: Vec<i64>,
    pool: Pool<Postgres>,
) -> Result<Vec<i32>, ServiceError> {
    let unique_doc_group_sizes = sqlx::query!(
        r#"
        SELECT DISTINCT doc_group_size
        FROM doc_group_embeddings
        WHERE story_id = ANY($1)
        "#,
        story_ids.as_slice(),
    )
    .fetch_all(&pool)
    .await
    .map_err(ServiceError::SelectUniqueDocGroupSizesPgError)?
    .into_iter()
    .map(|doc_group_size_container| doc_group_size_container.doc_group_size)
    .collect::<Vec<i32>>();

    Ok(unique_doc_group_sizes)
}

pub async fn upsert_doc_group_embedding_pg_query(
    doc_groups: impl Iterator<Item = DocGroupEmbedding>,
    pool: Pool<Postgres>,
) -> Result<(), ServiceError> {
    // TODO make this into a bulk query
    for g in doc_groups {
        sqlx::query!(
            r#"
            INSERT INTO doc_group_embeddings (id, story_id, doc_group_size, index, qdrant_point_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (story_id, doc_group_size, index) DO UPDATE
            SET
                qdrant_point_id = EXCLUDED.qdrant_point_id,
                updated_at = EXCLUDED.updated_at
            "#,
            g.id,
            g.story_id,
            g.doc_group_size,
            g.index as i32,
            g.qdrant_point_id,
            g.created_at,
            g.updated_at,
        ).execute(&pool).await.map_err(ServiceError::InsertDocGroupEmbeddingPgError)?;
    }

    Ok(())
}
