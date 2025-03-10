// src/models/user.rs

// Define Application User Model to be used in the application and in the database
// src/models/user.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

// Create User Impl to be done