use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Document {
    pub id: i32,
    pub name: String,
    pub content: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub user_id: Option<i32>,
    pub is_starred: Option<bool>,
    pub is_trashed: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentPayload {
    pub name: String,
    pub content: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentPayload {
    pub name: String,
    pub content: Option<String>,
    pub updated_at: NaiveDateTime
}