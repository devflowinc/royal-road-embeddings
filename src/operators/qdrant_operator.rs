use super::doc_group_embedding_operator::DocGroupQdrantPointIdContainer;
use crate::{
    data::models::{DocEmbedding, DocEmbeddingQdrantPayload, DocGroupEmbeddingQdrantPayload},
    errors::ServiceError,
};
use qdrant_client::{
    prelude::{QdrantClient, QdrantClientConfig},
    qdrant::{
        self, point_id::PointIdOptions, r#match::MatchValue,
        with_payload_selector::SelectorOptions, CreateCollection, Distance, FieldCondition,
        HasIdCondition, Match, PointId, PointStruct, RecommendPoints, SearchPoints, VectorParams,
        VectorsConfig, WithPayloadSelector,
    },
};

pub async fn get_qdrant_connection() -> Result<QdrantClient, ServiceError> {
    let qdrant_url = std::env::var("QDRANT_URL").expect("QDRANT_URL must be set");
    let qdrant_api_key = std::env::var("QDRANT_API_KEY").expect("QDRANT_API_KEY must be set");
    let mut config = QdrantClientConfig::from_url(qdrant_url.as_str());
    config.api_key = Some(qdrant_api_key);
    QdrantClient::new(Some(config)).map_err(ServiceError::QdrantConnectionError)
}

pub async fn create_doc_group_collection_qdrant_query(
    doc_group_size: i32,
) -> Result<(), ServiceError> {
    let qdrant_client = get_qdrant_connection().await.unwrap();

    let embedding_size = std::env::var("EMBEDDING_SIZE").unwrap_or("1536".to_owned());
    let embedding_size = embedding_size.parse::<u64>().unwrap_or(1536);

    let _ = qdrant_client
        .create_collection(&CreateCollection {
            collection_name: format!("doc_group_{}", doc_group_size),
            vectors_config: Some(VectorsConfig {
                config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                    VectorParams {
                        size: embedding_size,
                        distance: Distance::Cosine.into(),
                        hnsw_config: None,
                        quantization_config: None,
                        on_disk: None,
                    },
                )),
            }),
            ..Default::default()
        })
        .await
        .map_err(|err| {
            log::info!("Failed to create collection: {:?}", err);
        });

    Ok(())
}

pub async fn get_doc_embeddings_qdrant_query(
    qdrant_points: Vec<uuid::Uuid>,
) -> Result<Vec<Vec<f32>>, ServiceError> {
    let limit = Some(qdrant_points.len() as u32);
    let qdrant_client = get_qdrant_connection().await?;

    let mut resulting_vectors: Vec<Vec<f32>> = vec![];
    let mut offset = None;

    loop {
        let scroll_points = qdrant::ScrollPoints {
            collection_name: "doc_embeddings".into(),
            filter: Some(qdrant::Filter {
                should: vec![HasIdCondition {
                    has_id: qdrant_points
                        .clone()
                        .into_iter()
                        .map(|id| id.to_string().into())
                        .collect(),
                }
                .into()],
                ..Default::default()
            }),
            offset,
            limit,
            with_vectors: Some(true.into()),
            with_payload: Some(false.into()),
            ..Default::default()
        };

        let scroll_response = qdrant_client
            .scroll(&scroll_points)
            .await
            .map_err(ServiceError::ScrollDocEmbeddingQdrantError)?;

        let cur_vectors = scroll_response
            .result
            .into_iter()
            .flat_map(|res| match res.vectors?.vectors_options? {
                qdrant::vectors::VectorsOptions::Vector(vector) => Some(vector.data),
                _ => None,
            })
            .collect::<Vec<Vec<f32>>>();
        resulting_vectors.extend(cur_vectors);

        offset = scroll_response.next_page_offset;
        if offset.is_none() {
            break;
        }
    }

    Ok(resulting_vectors)
}

/// Returns the PointStructs added
pub async fn insert_doc_group_embedding_qdrant_query(
    existing_doc_groups: Vec<DocGroupQdrantPointIdContainer>,
    vectors: Vec<Vec<f32>>,
    story_id: i64,
    doc_group_size: i32,
) -> Result<Vec<PointStruct>, ServiceError> {
    let points: Vec<PointStruct> = vectors
        .into_iter()
        .enumerate()
        .map(|(idx, vector)| {
            let similar_existing_doc_group_point_id = existing_doc_groups
                .iter()
                .find(|doc_group| doc_group.index == idx as i32)
                .map(|doc_group| doc_group.qdrant_point_id);

            PointStruct {
                id: Some(
                    (similar_existing_doc_group_point_id.unwrap_or(uuid::Uuid::new_v4()))
                        .to_string()
                        .into(),
                ),
                vectors: Some(vector.into()),
                payload: DocGroupEmbeddingQdrantPayload {
                    story_id,
                    doc_group_size,
                    index: idx as i32,
                }
                .into(),
            }
        })
        .collect();

    let qdrant_client = get_qdrant_connection().await?;

    qdrant_client
        .upsert_points(
            format!("doc_group_{}", doc_group_size),
            None,
            points.clone(),
            None,
        )
        .await
        .map_err(ServiceError::UpsertDocGroupEmbeddingQdrantError)?;

    Ok(points)
}

pub async fn delete_reinsert_doc_embedding_qdrant_query(
    point_id_to_delete: Option<uuid::Uuid>,
    doc_embedding: DocEmbedding,
    vector: Vec<f32>,
) -> Result<(), ServiceError> {
    let client = get_qdrant_connection().await?;

    if let Some(point_id_to_delete) = point_id_to_delete {
        let point_ids_to_delete: Vec<PointId> = vec![point_id_to_delete.to_string().into()];

        let filter = qdrant::Filter {
            should: vec![HasIdCondition {
                has_id: point_ids_to_delete,
            }
            .into()],
            ..Default::default()
        };

        client
            .delete_points("doc_embeddings", None, &filter.into(), None)
            .await
            .map_err(ServiceError::DeleteDocEmbeddingQdrantError)?;
    }

    let point = PointStruct {
        id: Some(doc_embedding.qdrant_point_id.to_string().into()),
        vectors: Some(vector.into()),
        payload: DocEmbeddingQdrantPayload::from(doc_embedding).into(),
    };

    client
        .upsert_points_blocking("doc_embeddings", None, vec![point], None)
        .await
        .map_err(ServiceError::UpsertDocEmbeddingQdrantError)?;

    Ok(())
}

pub async fn recommend_group_doc_embeddings_qdrant_query(
    positive_qdrant_ids: Vec<uuid::Uuid>,
    doc_group_size: i32,
    limit: Option<u64>,
    page: Option<u64>,
) -> Result<Vec<i64>, ServiceError> {
    let client = get_qdrant_connection().await?;

    let recommend_result = client
        .recommend(&RecommendPoints {
            collection_name: format!("doc_group_{}", doc_group_size),
            positive: positive_qdrant_ids
                .into_iter()
                .map(|id| id.to_string().into())
                .collect(),
            negative: vec![],
            filter: None,
            limit: limit.unwrap_or(10) * 2,
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(SelectorOptions::Enable(true)),
            }),
            params: None,
            score_threshold: None,
            offset: Some(page.unwrap_or(0) * (limit.unwrap_or(10) * 2)),
            using: None,
            with_vectors: None,
            lookup_from: None,
            read_consistency: None,
            negative_vectors: vec![],
            positive_vectors: vec![],
            strategy: None,
            timeout: None,
            shard_key_selector: None,
        })
        .await
        .map_err(ServiceError::RecommendQdrantDocEmbeddingGroupError)?;

    let mut story_ids = vec![];

    for point in recommend_result.result {
        let story_id_as_str = point
            .payload
            .get("story_id")
            .expect("story_id not found")
            .as_str()
            .expect("story_id is not a string");

        let story_id: i64 = story_id_as_str.parse().expect("story_id is not a number");

        story_ids.push(story_id);
    }

    Ok(story_ids)
}

pub struct QdrantPoints {
    pub score: f32,
    pub point_id: uuid::Uuid,
    pub payload: DocEmbeddingQdrantPayload,
}

pub async fn search_qdrant_query(
    embedding: Vec<f32>,
    page: u64,
    doc_group_size: Option<i32>,
) -> Result<Vec<QdrantPoints>, ServiceError> {
    let qdrant_client = get_qdrant_connection().await?;
    let data = qdrant_client
        .search_points(&SearchPoints {
            collection_name: if doc_group_size.is_some() {
                format!("doc_group_{}", doc_group_size.unwrap())
            } else {
                "doc_embeddings".to_owned()
            },
            vector: embedding,
            limit: 10,
            offset: Some((page - 1) * 10),
            with_payload: Some(true.into()),
            ..Default::default()
        })
        .await
        .map_err(ServiceError::QdrantSearchError)?;

    let point_ids: Vec<QdrantPoints> = data
        .result
        .iter()
        .filter_map(|point| match point.clone().id?.point_id_options? {
            PointIdOptions::Uuid(id) => Some(QdrantPoints {
                score: point.score,
                point_id: uuid::Uuid::parse_str(&id).ok()?,
                payload: point.payload.clone().into(),
            }),
            PointIdOptions::Num(_) => None,
        })
        .collect();

    Ok(point_ids)
}

pub async fn similarity_top_filtered_point(
    query_embedding: Vec<f32>,
    story_id: i64,
    index: i64,
    doc_group_size: Option<i32>,
) -> Result<Option<f32>, ServiceError> {
    let mut collection_name = "doc_embeddings".to_owned();

    if let Some(doc_group_size) = doc_group_size {
        if doc_group_size > 1 {
            collection_name = format!("doc_group_{}", doc_group_size);
        }
    }

    let qdrant_filter = qdrant::Filter {
        should: vec![
            FieldCondition {
                key: "story_id".to_owned(),
                r#match: Some(Match {
                    match_value: Some(MatchValue::Text(story_id.to_string())),
                }),
                range: None,
                geo_bounding_box: None,
                geo_radius: None,
                values_count: None,
                geo_polygon: None,
            }
            .into(),
            FieldCondition {
                key: "index".to_owned(),
                r#match: Some(Match {
                    match_value: Some(MatchValue::Integer(index)),
                }),
                range: None,
                geo_bounding_box: None,
                geo_radius: None,
                values_count: None,
                geo_polygon: None,
            }
            .into(),
        ],
        ..Default::default()
    };

    let qdrant_client = get_qdrant_connection().await?;

    let search_result: Option<f32> = qdrant_client
        .search_points(&SearchPoints {
            collection_name,
            vector: query_embedding,
            limit: 1,
            offset: None,
            with_payload: None,
            filter: Some(qdrant_filter),
            ..Default::default()
        })
        .await
        .map_err(ServiceError::QdrantSearchError)
        .map(|res| {
            res.result
                .into_iter()
                .map(|point| point.score)
                .collect::<Vec<f32>>()
                .first()
                .cloned()
        })?;

    Ok(search_result)
}
