use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::errors::ServiceError;

#[derive(Debug, Deserialize, Serialize)]
struct ParseCallReturn {
    chunks: Vec<String>,
}

pub fn chunk_document(document: String) -> Result<Vec<String>, ServiceError> {
    // make a uuid for the document
    let temp_uuid = uuid::Uuid::new_v4();
    let temp_file_name = format!("{}.txt", temp_uuid);
    let temp_file_path = format!("./tmp/{}", temp_file_name);

    std::fs::write(&temp_file_path, &document).map_err(|e| {
        log::info!("Error: {:?}", e);
        ServiceError::CreateTmpFileError(e)
    })?;

    let output = Command::new("python")
        .args(&["parser_1.py", &temp_file_path])
        .output()
        .map_err(|e| {
            println!("Error: {:?}", e);
            ServiceError::ParseDocumentCallError(e)
        })?;

    std::fs::remove_file(&temp_file_path).map_err(|e| ServiceError::DeleteTmpFileError(e))?;

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
