// src/models/project.rs
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// Payload for creating a new project
#[derive(Debug, Deserialize)]
pub struct CreateProjectPayload {
    pub name: String,
    pub description: Option<String>,
}

// Payload for updating an existing project
#[derive(Debug, Deserialize)]
pub struct UpdateProjectPayload {
    pub name: Option<String>,
    pub description: Option<String>,
}
