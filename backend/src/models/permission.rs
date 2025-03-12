use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct DocumentPermission {
    pub document_id: i32,
    pub user_id: i32,
    pub role: String,
    pub created_at: DateTime<Utc>
}

#[derive(Debug, Deserialize)]
pub struct CreatePermissionPayload {
    pub document_id: i32,
    pub user_id: i32,
    pub role: String
}

#[derive(Debug, Deserialize)]
pub struct UpdatePermissionPayload {
    pub role: String
} 