use qdrant_client::{
    prelude::{QdrantClient, QdrantClientConfig},
    qdrant::{
        self, condition::ConditionOneOf, points_selector::PointsSelectorOneOf, CreateCollection,
        Distance, HasIdCondition, PointId, PointStruct, PointsSelector, VectorParams,
        VectorsConfig,
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

pub async fn delete_reinsert_doc_embedding_qdrant_query(
    point_id_to_delete: Option<uuid::Uuid>,
    doc_embedding: DocEmbedding,
    vector: Vec<f32>,
) -> Result<(), ServiceError> {
    let client = get_qdrant_connection().await?;

    if let Some(point_id_to_delete) = point_id_to_delete {
        let point_ids_to_delete: Vec<PointId> = vec![point_id_to_delete.to_string().into()];

        let mut filter = qdrant::Filter::default();

        filter.should.push(qdrant::Condition {
            condition_one_of: Some(ConditionOneOf::HasId(HasIdCondition {
                has_id: point_ids_to_delete,
            })),
        });

        let points_selector = PointsSelector {
            points_selector_one_of: Some(PointsSelectorOneOf::Filter(qdrant::Filter {
                should: vec![],
                must: vec![],
                must_not: vec![],
            })),
        };

        client
            .delete_points("doc_embeddings", &points_selector, None)
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
