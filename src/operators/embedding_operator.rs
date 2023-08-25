use ndarray::{Array, Array1};
use serde::{Deserialize, Serialize};

use crate::errors::ServiceError;

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomServerData {
    pub input: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomServerResponse {
    pub embeddings: Vec<f32>,
}

pub async fn create_embedding(messages: String) -> Result<Vec<f32>, ServiceError> {
    let embedding_server_call =
        std::env::var("EMBEDDING_SERVER_CALL").expect("EMBEDDING_SERVER_CALL must be set");

    let client = reqwest::Client::new();
    let resp = client
        .post(embedding_server_call)
        .json(&CustomServerData { input: messages })
        .send()
        .await
        .map_err(ServiceError::EmbeddingServerCallError)?
        .json::<CustomServerResponse>()
        .await
        .map_err(ServiceError::EmbeddingServerParseError)?;

    Ok(resp.embeddings)
}
pub async fn get_average_embedding(document_chunks: Vec<String>) -> Result<Vec<f32>, ServiceError> {
    let mut embeddings: Vec<Vec<f32>> = vec![];
    for chunk in document_chunks {
        embeddings.push(create_embedding(chunk).await?);
    }
    let average_embedding: Array1<f32> = embeddings
        .iter()
        .map(|a| Array::from(a.clone()))
        .reduce(|a, b| a + b)
        .ok_or(ServiceError::EmbeddingAveragingError)?;

    Ok(average_embedding.to_vec())
}
