use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DocEmbedding {
    pub id: uuid::Uuid,
    pub doc_html: String,
    pub story_id: i64,
    pub index: i32,
    pub qdrant_point_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl DocEmbedding {
    pub fn from_details(
        id: Option<uuid::Uuid>,
        doc_html: String,
        story_id: i64,
        index: i32,
        qdrant_point_id: Option<uuid::Uuid>,
        created_at: Option<chrono::NaiveDateTime>,
        updated_at: Option<chrono::NaiveDateTime>,
    ) -> Self {
        Self {
            id: id.unwrap_or(uuid::Uuid::new_v4()),
            doc_html,
            story_id,
            index,
            qdrant_point_id: qdrant_point_id.unwrap_or(uuid::Uuid::new_v4()),
            created_at: created_at.unwrap_or(chrono::Utc::now().naive_utc()),
            updated_at: updated_at.unwrap_or(chrono::Utc::now().naive_utc()),
        }
    }
}

pub struct DocEmbeddingQdrantPayload {
    pub story_id: i64,
    pub index: i32,
}

impl From<DocEmbedding> for DocEmbeddingQdrantPayload {
    fn from(doc_embedding: DocEmbedding) -> Self {
        Self {
            story_id: doc_embedding.story_id,
            index: doc_embedding.index,
        }
    }
}

impl From<DocEmbeddingQdrantPayload> for HashMap<String, qdrant_client::prelude::Value> {
    fn from(val: DocEmbeddingQdrantPayload) -> Self {
        let mut map = HashMap::new();
        map.insert("story_id".to_string(), val.story_id.to_string().into());
        map.insert("index".to_string(), val.index.to_string().into());
        map
    }
}

impl From<HashMap<String, qdrant_client::prelude::Value>> for DocEmbeddingQdrantPayload {
    fn from(value: HashMap<String, qdrant_client::prelude::Value>) -> Self {
        Self {
            story_id: value
                .get("story_id")
                .unwrap_or(&qdrant_client::qdrant::Value { kind: Option::None })
                .to_string()
                .parse::<i64>()
                .unwrap_or(0),
            index: value
                .get("index")
                .unwrap_or(&qdrant_client::qdrant::Value { kind: Option::None })
                .to_string()
                .parse::<i32>()
                .unwrap_or(0),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DocGroupEmbedding {
    pub id: uuid::Uuid,
    pub story_id: i64,
    pub doc_group_size: i32,
    pub index: i32,
    pub qdrant_point_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl DocGroupEmbedding {
    pub fn from_details(
        id: Option<uuid::Uuid>,
        story_id: i64,
        doc_group_size: i32,
        index: i32,
        qdrant_point_id: Option<uuid::Uuid>,
        created_at: Option<chrono::NaiveDateTime>,
        updated_at: Option<chrono::NaiveDateTime>,
    ) -> Self {
        Self {
            id: id.unwrap_or(uuid::Uuid::new_v4()),
            story_id,
            doc_group_size,
            index,
            qdrant_point_id: qdrant_point_id.unwrap_or(uuid::Uuid::new_v4()),
            created_at: created_at.unwrap_or(chrono::Utc::now().naive_utc()),
            updated_at: updated_at.unwrap_or(chrono::Utc::now().naive_utc()),
        }
    }
}

pub struct DocGroupEmbeddingQdrantPayload {
    pub story_id: i64,
    pub doc_group_size: i32,
    pub index: i32,
}

impl From<DocGroupEmbedding> for DocGroupEmbeddingQdrantPayload {
    fn from(doc_group_embedding: DocGroupEmbedding) -> Self {
        Self {
            story_id: doc_group_embedding.story_id,
            doc_group_size: doc_group_embedding.doc_group_size,
            index: doc_group_embedding.index,
        }
    }
}

impl From<DocGroupEmbeddingQdrantPayload> for HashMap<String, qdrant_client::prelude::Value> {
    fn from(
        doc_group_embedding: DocGroupEmbeddingQdrantPayload,
    ) -> HashMap<String, qdrant_client::prelude::Value> {
        let mut map = HashMap::new();
        map.insert(
            "story_id".to_string(),
            doc_group_embedding.story_id.to_string().into(),
        );
        map.insert(
            "index".to_string(),
            (doc_group_embedding.index as i64).into(),
        );
        map
    }
}
