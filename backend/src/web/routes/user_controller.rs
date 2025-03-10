// src/controllers/user_controller.rs
// Request Handlers
use axum::{
    extract::{Extension, Path, Json},
    Router,
};
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::{Value, json};
use crate::models::user::User;
use crate::{Error, Result};
use axum::routing::{get, post};

/// The User model representing a row in the "users" table.
/// Payload for creating a new user.
#[derive(Debug,Deserialize)]
pub struct CreateUserPayload {
    pub name: String,
    pub email: String,
    pub password: String
}

#[derive(Debug,Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

/// GET handler for retrieving a user by ID.
/// Accessible via: GET /api/users/:id
pub async fn get_user(
    Path(user_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<User>> {
    println!("->> {:<12} - get_user", "HANDLER");

    let result = sqlx::query_as!(
        User,
        r#"SELECT id, name, email FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(Error::UserNotFound),
    }
}
/// POST handler for creating a new user.
/// Accessible via: POST /api/users
pub async fn create_user(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<Json<User>> {
    println!("->> {:<12} - create_user", "HANDLER");

    // First insert the user
    let result = sqlx::query!(
        "INSERT INTO users (name, email, password) VALUES ($1, $2, $3) RETURNING id",
        payload.name,
        payload.email,
        payload.password
    )
    .fetch_one(&pool)
    .await;

    // Check if insertion was successful
    match result {
        Ok(record) => {
            // Then fetch the user by email
            let user = sqlx::query_as!(
                User,
                r#"SELECT id, name, email FROM users WHERE id = $1"#,
                record.id
            )
            .fetch_one(&pool)
            .await;
            
            match user {
                Ok(user) => Ok(Json(user)),
                Err(e) => {
                    println!("Error fetching user: {:?}", e);
                    Err(Error::UserNotFound)
                }
            }
        },
        Err(e) => {
            println!("Error creating user: {:?}", e);
            Err(Error::UserCreationError)
        }
    }
}

pub async fn api_login(payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login", "HANDLER");

    // IMPL AUTH LOGIN

    // Test example
    if payload.email != "Hello@gmail.com" || payload.password != "test" {
        return Err(Error::LoginFail);
    }

    // IMPL SET COOKIES

    // Create Success
    let success = Json(json!({
        "result": {
            "success": true
        }
    }));

    Ok(success)
}

/// Combine user-related routes into one Router instance.
pub fn user_routes() -> Router {
    Router::new()
        .route("/api/login", post(api_login))
        .route("/api/users/:id", get(get_user))
        .route("/api/users", post(create_user))
}