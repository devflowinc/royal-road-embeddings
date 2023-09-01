use qdrant_client::qdrant;
use sqlx::{Pool, Postgres};

use crate::data::models::DocGroupEmbedding;
use crate::{
    data::models::DocEmbedding, errors::ServiceError,
    handlers::doc_group_handler::IndexDocumentGroupRequest,
};

use super::embedding_operator::group_average_embeddings;
use super::qdrant_operator::get_doc_embeddings_qdrant_query;
use super::qdrant_operator::insert_doc_group_embedding_qdrant_query;

pub struct QdrantPointIdContainer {
    pub qdrant_point_id: uuid::Uuid,
}

pub async fn upsert_doc_embedding_pg_query(
    doc_embedding: DocEmbedding,
    pool: Pool<Postgres>,
) -> Result<Option<uuid::Uuid>, ServiceError> {
    // select qdrant_point_id from doc_embeddings where story_id = $1 and index = $2
    let qdrant_point_id: Option<QdrantPointIdContainer> = sqlx::query_as!(
        QdrantPointIdContainer,
        r#"
        SELECT qdrant_point_id
        FROM doc_embeddings
        WHERE story_id = $1 AND index = $2
        "#,
        doc_embedding.story_id,
        doc_embedding.index,
    )
    .fetch_optional(&pool)
    .await
    .map_err(ServiceError::UpsertDocEmbeddingPgError)?;

    sqlx::query!(
        r#"
        INSERT INTO doc_embeddings (id, doc_html, story_id, index, qdrant_point_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (story_id, index) DO UPDATE
        SET
            doc_html = EXCLUDED.doc_html,
            story_id = EXCLUDED.story_id,
            index = EXCLUDED.index,
            qdrant_point_id = EXCLUDED.qdrant_point_id,
            updated_at = EXCLUDED.updated_at
        "#,
        doc_embedding.id,
        doc_embedding.doc_html,
        doc_embedding.story_id,
        doc_embedding.index,
        doc_embedding.qdrant_point_id,
        doc_embedding.created_at,
        doc_embedding.updated_at,
    )
    .execute(&pool)
    .await
    .map_err(ServiceError::UpsertDocEmbeddingPgError)?;

    Ok(qdrant_point_id.map(|qdrant_point_id_container| qdrant_point_id_container.qdrant_point_id))
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

pub async fn create_doc_group_embedding(
    groups: IndexDocumentGroupRequest,
    pool: Pool<Postgres>,
) -> Result<(), ServiceError> {
    let qdrant_points = match groups.clone() {
        IndexDocumentGroupRequest::All { .. } => sqlx::query_as!(
            QdrantPointIdContainer,
            r#"
                SELECT qdrant_point_id
                FROM doc_embeddings
                ORDER BY index ASC
                "#,
        )
        .fetch_all(&pool)
        .await
        .map_err(ServiceError::GetDocEmbeddingsPgError)?,
        IndexDocumentGroupRequest::Stories { story_ids, .. } => sqlx::query_as!(
            QdrantPointIdContainer,
            r#"
                SELECT qdrant_point_id
                FROM doc_embeddings
                WHERE story_id = ANY($1)
                ORDER BY index ASC
                "#,
            &story_ids[..]
        )
        .fetch_all(&pool)
        .await
        .map_err(ServiceError::GetDocEmbeddingsPgError)?,
        IndexDocumentGroupRequest::Story { story_id, .. } => sqlx::query_as!(
            QdrantPointIdContainer,
            r#"
	        SELECT qdrant_point_id
		FROM doc_embeddings
		WHERE story_id = $1
	    "#,
            story_id
        )
        .fetch_all(&pool)
        .await
        .map_err(ServiceError::GetDocEmbeddingsPgError)?,
    };

    // fetch all embeddings
    let embeddings =
        get_doc_embeddings_qdrant_query(qdrant_points.iter().map(|x| x.qdrant_point_id).collect())
            .await?;

    match groups {
        IndexDocumentGroupRequest::Story {
            story_id,
            doc_group_size,
        } => {
            let group_average = group_average_embeddings(embeddings, doc_group_size)?;
            // upsert doc group metadata
            // upsert doc group embedding
            let qdrant_points_added =
                insert_doc_group_embedding_qdrant_query(group_average, story_id, doc_group_size)
                    .await?;

            let doc_groups = qdrant_points_added.into_iter().flat_map(|point_struct| {
                let qdrant_point_id: uuid::Uuid = match point_struct.id?.point_id_options? {
                    qdrant::point_id::PointIdOptions::Uuid(id) => Some(id.parse().unwrap())?,
                    qdrant::point_id::PointIdOptions::Num(_) => {
                        unreachable!("This should not happen")
                    }
                };
                let index = match point_struct.payload.get("index")?.kind.clone()? {
                    qdrant::value::Kind::IntegerValue(num) => num as usize,
                    _ => unreachable!("This should not happen"),
                };
                Some(DocGroupEmbedding::from_details(
                    None,
                    story_id,
                    doc_group_size,
                    index as i32,
                    Some(qdrant_point_id),
                    None,
                    None,
                ))
            });

            upsert_doc_group_embedding_pg_query(doc_groups, pool).await?;

            Ok(())
        }
        _ => unimplemented!("Not yet implemented for multiple stories at once"),
    }
}
