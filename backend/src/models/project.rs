// src/models/project.rs
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub user_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub is_starred: Option<bool>,
    pub is_trashed: Option<bool>,
}

// Payload for creating a new project
#[derive(Debug, Deserialize)]
pub struct CreateProjectPayload {
    pub _name: String,
}

// Payload for updating an existing project
#[derive(Debug, Deserialize)]
pub struct UpdateProjectPayload {
    pub _name: String,
}
