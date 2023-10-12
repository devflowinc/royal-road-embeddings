use std::process::Command;
use serde::{Deserialize, Serialize};

use crate::errors::ServiceError;

#[derive(Debug, Deserialize, Serialize)]
struct ParseCallReturn {
    chunks: Vec<String>,
}

pub fn chunk_document(document: String) -> Result<Vec<String>, ServiceError> {
    let output = Command::new("python")
        .args(&["parser_1.py", &document])
        .output()
        .map_err(|_e| ServiceError::ChunkDocumentError)?;

    let chunk_stringified_json =
        String::from_utf8(output.stdout.clone()).map_err(|_e| ServiceError::ChunkDocumentError)?;

    let chunk_json: ParseCallReturn = serde_json::from_str(&chunk_stringified_json)
        .map_err(|_e| ServiceError::ParseDocumentResponseError)?;

    Ok(chunk_json.chunks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_chunk_document() {
        let result = chunk_document("I am a man. That has a Very very big plan.".to_string());
        println!("Result {result:?}");
        assert_eq!(result.expect("Should exist").len(), 1);
    }
}
