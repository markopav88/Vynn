// src/models/user.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    #[warn(dead_code)]
    pub password: String,
    pub ai_credits: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_bytes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_projects: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_documents: Option<i32>
}

#[derive(Debug, Deserialize)]
pub struct CreateUserPayload {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserPayload {
    pub email: String, 
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserPayload {
    pub name: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}
