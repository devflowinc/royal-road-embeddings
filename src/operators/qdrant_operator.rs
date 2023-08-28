use qdrant_client::{
    prelude::{QdrantClient, QdrantClientConfig},
    qdrant::PointStruct,
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

pub async fn upsert_doc_embedding_qdrant_query(
    doc_embedding: DocEmbedding,
    vector: Vec<f32>,
) -> Result<(), ServiceError> {
    let client = get_qdrant_connection().await?;
    let point = PointStruct {
        id: Some(doc_embedding.qdrant_point_id.to_string().into()),
        vectors: Some(vector.into()),
        payload: DocEmbeddingQdrantPayload::from(doc_embedding).into(),
    };

    client
        .upsert_points("collection_name", vec![point], None)
        .await
        .map_err(ServiceError::UpsertDocEmbeddingQdrantError)?;

    Ok(())
}
