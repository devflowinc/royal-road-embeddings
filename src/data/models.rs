use std::collections::HashMap;

use qdrant_client::prelude::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DocEmbedding {
    pub id: uuid::Uuid,
    pub doc_html: String,
    pub story_id: i64,
    pub doc_num: i64,
    pub qdrant_point_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl DocEmbedding {
    pub fn from_details(
        id: Option<uuid::Uuid>,
        doc_html: String,
        story_id: i64,
        doc_num: i64,
        qdrant_point_id: Option<uuid::Uuid>,
        created_at: Option<chrono::NaiveDateTime>,
        updated_at: Option<chrono::NaiveDateTime>,
    ) -> Self {
        Self {
            id: id.unwrap_or(uuid::Uuid::new_v4()),
            doc_html,
            story_id,
            doc_num,
            qdrant_point_id: qdrant_point_id.unwrap_or(uuid::Uuid::new_v4()),
            created_at: created_at.unwrap_or(chrono::Utc::now().naive_utc()),
            updated_at: updated_at.unwrap_or(chrono::Utc::now().naive_utc()),
        }
    }
}

pub struct DocEmbeddingQdrantPayload {
    pub story_id: i64,
    pub doc_num: i64,
}

impl From<DocEmbedding> for DocEmbeddingQdrantPayload {
    fn from(doc_embedding: DocEmbedding) -> Self {
        Self {
            story_id: doc_embedding.story_id,
            doc_num: doc_embedding.doc_num,
        }
    }
}

impl Into<HashMap<String, qdrant_client::prelude::Value>> for DocEmbeddingQdrantPayload {
    fn into(self) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        map.insert("story_id".to_string(), self.story_id.to_string().into());
        map.insert("doc_num".to_string(), self.doc_num.to_string().into());
        map
    }
}
