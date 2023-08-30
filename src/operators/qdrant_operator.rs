use qdrant_client::{
    prelude::{QdrantClient, QdrantClientConfig},
    qdrant::{
        self, with_payload_selector::SelectorOptions, CreateCollection, Distance, HasIdCondition,
        PointId, PointStruct, RecommendPoints, VectorParams, VectorsConfig, WithPayloadSelector,
    },
};

use crate::{
    data::models::{DocEmbedding, DocEmbeddingQdrantPayload},
    errors::ServiceError,
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

    let embedding_size = std::env::var("EMBEDDING_SIZE").unwrap_or("1024".to_owned());
    let embedding_size = embedding_size.parse::<u64>().unwrap_or(1024);

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
    let qdrant_client = get_qdrant_connection().await?;
    let scroll_points = qdrant::ScrollPoints {
        filter: Some(qdrant::Filter {
            should: vec![HasIdCondition {
                has_id: qdrant_points
                    .into_iter()
                    .map(|id| id.to_string().into())
                    .collect(),
            }
            .into()],
            ..Default::default()
        }),
        limit: None,
        with_vectors: Some(true.into()),
        ..Default::default()
    };

    let scroll_response = qdrant_client
        .scroll(&scroll_points)
        .await
        .map_err(ServiceError::ScrollDocEmbeddingQdrantError)?;

    Ok(scroll_response
        .result
        .into_iter()
        .flat_map(|res| match res.vectors?.vectors_options? {
            qdrant::vectors::VectorsOptions::Vector(vector) => Some(vector.data),
            _ => None,
        })
        .collect::<Vec<Vec<f32>>>())
}

pub async fn upsert_doc_group_embedding_qdrant_query(
    _vector: Vec<f32>,
    _story_id: i64,
    _doc_group_size: i32,
) -> Result<(), ServiceError> {
    unimplemented!("upsert_doc_group_embedding_qdrant_query ")
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
            .delete_points("doc_embeddings", &filter.into(), None)
            .await
            .map_err(ServiceError::DeleteDocEmbeddingQdrantError)?;
    }

    let point = PointStruct {
        id: Some(doc_embedding.qdrant_point_id.to_string().into()),
        vectors: Some(vector.into()),
        payload: DocEmbeddingQdrantPayload::from(doc_embedding).into(),
    };

    client
        .upsert_points_blocking("doc_embeddings", vec![point], None)
        .await
        .map_err(ServiceError::UpsertDocEmbeddingQdrantError)?;

    Ok(())
}

pub async fn reccomend_group_doc_embeddings_qdrant_query(
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
        })
        .await
        .map_err(ServiceError::RecommendQdrantDocEmbeddingGroupError)?;

    let mut story_ids = vec![];

    for point in recommend_result.result {
        let story_id: i64 = point
            .payload
            .get("story_id")
            .expect("story_id not found")
            .as_integer()
            .expect("story_id is not an integer");
        story_ids.push(story_id);
    }

    Ok(story_ids)
}
