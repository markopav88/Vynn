// src/models/user.rs

// Define Application User Model to be used in the application and in the database
// src/models/user.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub ai_credits: Option<i32>,
}

// Payload for creating a new user.
#[derive(Debug, Deserialize)]
pub struct CreateUserPayload {
    pub name: String,
    pub email: String,
    pub password: String,
}

// Payload for a user logging in
#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

/*
/ Payload for a user logging in
/ Why need this when we have 'CreateUserPayload' which is currently the same enum?
/ Well we might expand upon all a user is and have traits not needed to simply login
/ but that will be needed for us to update our account information
*/
#[derive(Debug, Deserialize)]
pub struct UpdateUserPayload {
    pub name: String,
    pub email: String,
    pub password: String,
}
