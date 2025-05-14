// src/models/user.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub ai_credits: i32,
    pub storage_bytes: i64,
    pub max_projects: i32,
    pub max_documents: i32
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
}
