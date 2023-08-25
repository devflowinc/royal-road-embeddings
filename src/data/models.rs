use std::collections::HashMap;

use qdrant_client::prelude::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Document {
    pub id: uuid::Uuid,
    pub story_id: u32,
    pub doc_num: String,
    pub embedding: Vec<f32>,
    pub doc_content: String,
}

impl Document {
    pub fn from_details(
        story_id: u32,
        doc_num: String,
        embedding: Vec<f32>,
        doc_content: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            story_id,
            doc_num,
            embedding,
            doc_content,
        }
    }
}

impl Into<HashMap<String, qdrant_client::prelude::Value>> for Document {
    fn into(self) -> HashMap<String, Value> {
        std::collections::HashMap::from([
            ("".into(), Value::from(true)),
            ("".into(), Value::from(9000)),
            ("".into(), Value::from("Hi Qdrant!")),
            ("".into(), Value::from(vec![1.234, 0.815])),
        ])
    }
}
