use std::collections::HashMap;

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
        FROM doc_group_embeddings
        WHERE story_id = ANY($1) AND doc_group_size = $2
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

pub async fn re_index_appropriate_doc_groups(
    story_id: i64,
    single_chapter_index: i32,
    pool: Pool<Postgres>,
) -> Result<(), ServiceError> {
    let mut group_to_reaverage = HashMap::new();
    let unique_doc_group_sizes =
        get_unique_doc_group_sizes(vec![story_id.clone()], pool.clone()).await?;

    for doc_group_size in unique_doc_group_sizes {
        let group_index = single_chapter_index / doc_group_size;

        group_to_reaverage.insert(doc_group_size, group_index);
    }

    Ok(())
}
