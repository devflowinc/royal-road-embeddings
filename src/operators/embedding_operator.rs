use serde::{Deserialize, Serialize};

use ndarray::Array2;

use crate::errors::ServiceError;

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomServerData {
    pub input: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomServerResponse {
    pub embeddings: Vec<f32>,
}

pub async fn create_embedding(message: String) -> Result<Vec<f32>, ServiceError> {
    get_average_embedding(vec![message]).await
}

pub async fn get_average_embedding(document_chunks: Vec<String>) -> Result<Vec<f32>, ServiceError> {
    let embedding_server_call =
        std::env::var("EMBEDDING_SERVER_CALL").expect("EMBEDDING_SERVER_CALL must be set");

    let client = reqwest::Client::new();
    let resp = client
        .post(embedding_server_call)
        .json(&CustomServerData {
            input: document_chunks,
        })
        .send()
        .await
        .map_err(ServiceError::EmbeddingServerCallError)?
        .json::<CustomServerResponse>()
        .await
        .map_err(ServiceError::EmbeddingServerParseError)?;

    Ok(resp.embeddings)
}

pub fn average_embeddings(embeddings: Vec<Vec<f32>>) -> Result<Vec<f32>, ServiceError> {
    let shape = (embeddings.len(), embeddings[0].len());
    let flat: Vec<f32> = embeddings.iter().flatten().cloned().collect();
    let arr: Array2<f32> =
        Array2::from_shape_vec(shape, flat).map_err(ServiceError::VectorToArrayError)?;

    Ok((arr.sum_axis(ndarray::Axis(0)) / (embeddings.len() as f32)).to_vec())
}

pub fn group_average_embeddings_better(
    embeddings: Vec<Vec<f32>>,
    group_size: i32,
) -> Result<Array2<f32>, ServiceError> {
    let shape = (
        ceil_div(embeddings.len(), group_size as usize),
        group_size as usize,
        embeddings[0].len(),
    );
    let flat: Vec<f32> = embeddings.iter().flatten().cloned().collect();
    let arr: ndarray::Array3<f32> =
        ndarray::Array3::from_shape_vec(shape, flat).map_err(ServiceError::VectorToArrayError)?;

    Ok(arr.sum_axis(ndarray::Axis(1)) / arr.len_of(ndarray::Axis(1)) as f32)
}

pub fn ceil_div(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

pub fn group_average_embeddings(
    embeddings: Vec<Vec<f32>>,
    group_size: i32,
) -> Result<Vec<Vec<f32>>, ServiceError> {
    let chunks = embeddings.chunks(group_size as usize);
    chunks
        .into_iter()
        .map(|chunk| average_embeddings(chunk.to_vec()))
        .collect::<Result<Vec<Vec<f32>>, ServiceError>>()
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    pub fn test_average_embeddings() {
        let embeddings = vec![
            vec![3.0, 2.5, 1.0],
            vec![1.0, 2.5, 1.0],
            vec![2.0, 2.5, 1.0],
            vec![2.0, 2.5, 1.0],
        ];

        let result = average_embeddings(embeddings).unwrap();
        assert!(result == vec![2.0, 2.5, 1.0]);
    }

    #[test]
    pub fn test_group_average_embeddings() {
        let embeddings = vec![
            vec![3.0, 2.5, 1.0],
            vec![1.0, 2.5, 1.0],
            vec![2.0, 2.5, 1.0],
            vec![2.0, 2.5, 1.0],
        ];
        let result = group_average_embeddings(embeddings, 2).unwrap();

        assert!(result == vec![vec![2.0, 2.5, 1.0], vec![2.0, 2.5, 1.0]]);
    }
}
