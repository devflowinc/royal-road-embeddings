use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuestionType {
    title: String,
    choices: Vec<String>,
    is_required: bool,
}

impl Question {
    pub fn from_details(question: QuestionType, poll_id: uuid::Uuid) -> Question {
        Question {
            id: uuid::Uuid::new_v4(),
            poll_id,
            question: question.title,
            choices: question.choices,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, FromRow)]
pub struct Question {
    pub id: uuid::Uuid,
    pub poll_id: uuid::Uuid,
    pub question: String,
    pub choices: Vec<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
