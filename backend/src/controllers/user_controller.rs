// src/controllers/user_controller.rs
//Request Handlers
use axum::{
    extract::{Extension, Path, Json},
    response::IntoResponse,
    Router,
};
use sqlx::PgPool;
use serde::{Serialize, Deserialize};
use crate::models::user::User;

/// The User model representing a row in the "users" table.
/// Payload for creating a new user.
#[derive(Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
}

/// GET handler for retrieving a user by ID.
/// Accessible via: GET /api/users/:id
pub async fn get_user(
    Path(user_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, User>(
        "SELECT id, name, email FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(user) => axum::Json(user).into_response(),
        Err(_) => "User not found".into_response(),
    }
}
#[axum::debug_handler]
/// POST handler for creating a new user.
/// Accessible via: POST /api/users
pub async fn create_user(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id",
        payload.name,
        payload.email
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(row) => {
            let id = row.id;
            (axum::http::StatusCode::CREATED, format!("User created with id: {}", id)).into_response()
        }
        Err(e) => {
            eprintln!("Error creating user: {:?}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user").into_response()
        }
    }
}

/// Combine user-related routes into one Router instance.
pub fn user_routes() -> Router {
    use axum::routing::{get, post};
    Router::new()
        .route("/api/users/:id", get(get_user))
        .route("/api/users", post(create_user))
}
