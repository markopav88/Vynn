use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct DocumentPermission {
    pub document_id: i32,
    pub user_id: i32,
    pub role: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPermissions {
    pub user_id: i32,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePermissionPayload {
    pub user_id: i32,
    pub role: String
}

#[derive(Debug, Deserialize)]
pub struct UpdatePermissionPayload {
    pub user_id: i32,
    pub role: String
} 