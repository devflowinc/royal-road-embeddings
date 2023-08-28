use qdrant_client::{
    prelude::{QdrantClient, QdrantClientConfig},
    qdrant::{
        self, condition::ConditionOneOf, points_selector::PointsSelectorOneOf, HasIdCondition,
        PointId, PointStruct, PointsSelector,
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
