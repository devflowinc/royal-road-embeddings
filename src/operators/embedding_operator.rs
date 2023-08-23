use serde::{Deserialize, Serialize};

use crate::errors::ServiceError;

pub async fn create_embedding(message: &str) -> Result<Vec<f32>, ServiceError> {
    create_server_embedding(message).await
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomServerData {
    pub input: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomServerResponse {
    pub embeddings: Vec<f32>,
}

pub async fn create_server_embedding(message: &str) -> Result<Vec<f32>, ServiceError> {
    let embedding_server_call =
        std::env::var("EMBEDDING_SERVER_CALL").expect("EMBEDDING_SERVER_CALL must be set");

    let client = reqwest::Client::new();
    let resp = client
        .post(embedding_server_call)
        .json(&CustomServerData {
            input: message.to_string(),
        })
        .send()
        .await
        .map_err(ServiceError::EmbeddingServerCallError)?
        .json::<CustomServerResponse>()
        .await
        .map_err(ServiceError::EmbeddingServerParseError)?;

    Ok(resp.embeddings)
}
