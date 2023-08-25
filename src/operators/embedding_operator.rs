use serde::{Deserialize, Serialize};

use crate::errors::ServiceError;

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomServerData {
    pub input: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomServerResponse {
    pub embedding: Vec<f32>,
}

/// Creates embeddings given a list of messages and returns the average embedding vector
/// if you want the embedding of a single message, just pass a vector with one element
/// # Arguments
///     messages: Vec<String> - a vector of messages
/// # Example
/// ```
/// use embedding_operator::create_embedding;
/// let messages = vec!["Hello World".to_string(), "How are you?".to_string()];
/// let embedding = create_embedding(messages).await.unwrap();
pub async fn create_embedding(messages: Vec<String>) -> Result<Vec<f32>, ServiceError> {
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

    Ok(resp.embedding)
}
