use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDateTime};

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct Document {
    pub id: i32,
    pub name: String,
    pub content: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub user_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentPayload {
    pub name: String,
    pub content: Option<String>,
    pub user_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentPayload {
    pub name: String,
    pub content: Option<String>,
    pub updated_at: NaiveDateTime
}