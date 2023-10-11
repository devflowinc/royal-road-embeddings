use std::process::Command;

use itertools::Itertools;
use serde::{Serialize, Deserialize};

use crate::errors::ServiceError;

#[derive(Debug, Deserialize, Serialize)]
struct ParseCallReturn <> {
    chunks: Vec<String>,
}

pub fn chunk_document(document: String) -> Result<Vec<String>, ServiceError> {
    let output = Command::new("./parser")
        .arg(document)
        .output()
        .map_err(ServiceError::ParseDocumentCallError)?;

    println!("Output: {output:?}");
    let result : ParseCallReturn = serde_json::from_slice(&output.stdout)
        .map_err(|err| {
            log::error!("Error while chunking document: {err:?}");
            println!("Error while chunking document: {err:?}");
            ServiceError::NotImplemented
        })?;

    Ok(result.chunks.iter().map(|chunk| chunk.to_string()).collect_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_chunk_document() {
        let result = chunk_document("I am a man. That has a Very very big plan.".to_string());
        println!("Result {result:?}");
        assert_eq!(result.expect("Should exist").len(), 2);
    }
}
