use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct ProjectPermission {
    pub project_id: i32,
    pub user_id: i32,
    pub role: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct UserProjectPermissions {
    pub user_id: i32,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectPermissionPayload {
    pub user_id: i32,
    pub role: String
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectPermissionPayload {
    pub user_id: i32,
    pub role: String
} 