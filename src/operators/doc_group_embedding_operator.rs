use qdrant_client::qdrant::PointStruct;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;

use crate::{
    data::models::{DocGroupEmbedding, DocGroupEmbeddingQdrantPayload},
    errors::ServiceError,
};

use super::{
    embedding_operator::average_embeddings,
    qdrant_operator::{get_doc_embeddings_qdrant_query, get_qdrant_connection},
};

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
        group_index as i64 * doc_group_size as i64,
        group_index as i64 * doc_group_size as i64 + doc_group_size as i64 - 1,
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

pub async fn re_index_appropriate_doc_groups(
    story_id: i64,
    single_chapter_index: i32,
    pool: Pool<Postgres>,
) -> Result<(), ServiceError> {
    // get all of the doc group sizes for the given story_id
    let mut group_to_reaverage = HashMap::new();
    let unique_doc_group_sizes =
        get_unique_doc_group_sizes(vec![story_id.clone()], pool.clone()).await?;
    let unique_doc_group_sizes1 = unique_doc_group_sizes.clone();

    // from the single_chapter_index and unique_doc_group_sizes, determine the indices of the groups which need to be re-averaged
    for doc_group_size in unique_doc_group_sizes {
        let group_index = single_chapter_index / doc_group_size;

        group_to_reaverage.insert(doc_group_size, group_index);
    }
    let group_to_reaverage1 = group_to_reaverage.clone();
    let group_to_reaverage2 = group_to_reaverage.clone();

    // get all of the single chapter vectors for each of the the group sizes and indices determined above
    let mut vectors_to_average = HashMap::new();
    for (doc_group_size, group_index) in group_to_reaverage {
        let vectors = get_single_vectors_to_re_average(
            story_id.clone(),
            doc_group_size.clone(),
            group_index.clone(),
            pool.clone(),
        )
        .await?;

        vectors_to_average.insert(doc_group_size, vectors);
    }
    let mut group_average_vectors = HashMap::new();

    // average the vectors for each of the group sizes and indices determined above
    for (doc_group_size, vectors) in vectors_to_average {
        let average_vector = average_embeddings(vectors)?;

        group_average_vectors.insert(doc_group_size, average_vector);
    }
    let group_average_vectors1 = group_average_vectors.clone();

    // get the existing qdrant_point_ids for each of the group sizes and indices determined above or none if they don't exist
    let mut group_qdrant_point_ids = HashMap::new();
    for (doc_group_size, group_index) in group_to_reaverage1 {
        let doc_group_qdrant_point_ids = get_indexed_doc_group_qdrant_ids_pg_query(
            vec![story_id.clone()],
            doc_group_size.clone(),
            vec![group_index.clone()],
            pool.clone(),
        )
        .await?;

        let uuid_to_insert = match doc_group_qdrant_point_ids.first() {
            Some(doc_group_qdrant_point_id) => Some(doc_group_qdrant_point_id.qdrant_point_id),
            None => None,
        };

        group_qdrant_point_ids.insert(doc_group_size, uuid_to_insert);
    }
    let group_qdrant_point_ids1 = group_qdrant_point_ids.clone();

    // create the qdrant points to upsert
    let mut qdrant_points_to_upsert: Vec<PointStruct> = Vec::new();
    for (doc_group_size, average_vector) in group_average_vectors {
        let qdrant_point_id = match group_qdrant_point_ids.get(&doc_group_size) {
            Some(qdrant_point_id) => match qdrant_point_id {
                Some(qdrant_point_id) => qdrant_point_id.clone(),
                None => uuid::Uuid::new_v4(),
            },
            None => uuid::Uuid::new_v4(),
        };

        let qdrant_point = PointStruct {
            id: Some(qdrant_point_id.to_string().into()),
            vectors: Some(average_vector.into()),
            payload: DocGroupEmbeddingQdrantPayload {
                story_id: story_id.clone(),
                doc_group_size: doc_group_size.clone(),
                index: group_to_reaverage2[&doc_group_size].clone(),
            }
            .into(),
        };

        qdrant_points_to_upsert.push(qdrant_point);
    }

    // upsert the qdrant points for each of the group sizes determined above
    let qdrant_client = get_qdrant_connection().await?;
    for unique_size in unique_doc_group_sizes1 {
        let points_of_group_size = qdrant_points_to_upsert
            .iter()
            .filter(|point| {
                let group_size = point
                    .payload
                    .get("doc_group_size")
                    .expect("doc_group_size in payload must not be none")
                    .as_integer()
                    .expect("doc_group_size in payload must be an integer");
                group_size == unique_size as i64
            })
            .cloned()
            .collect::<Vec<PointStruct>>();

        qdrant_client
            .upsert_points(
                format!("doc_group_{}", unique_size),
                points_of_group_size,
                None,
            )
            .await
            .map_err(ServiceError::UpsertDocGroupEmbeddingQdrantError)?;
    }

    // create the doc group embeddings to upsert
    let doc_groups_to_upsert = group_average_vectors1
        .into_iter()
        .map(|(doc_group_size, _)| {
            DocGroupEmbedding::from_details(
                None,
                story_id.clone(),
                doc_group_size.clone(),
                group_to_reaverage2[&doc_group_size],
                group_qdrant_point_ids1[&doc_group_size],
                None,
                None,
            )
        })
        .collect::<Vec<DocGroupEmbedding>>();

    // upsert the doc group embeddings created above
    upsert_doc_group_embedding_pg_query(doc_groups_to_upsert.into_iter(), pool).await?;

    Ok(())
}
