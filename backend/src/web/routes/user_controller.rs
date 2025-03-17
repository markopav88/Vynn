// src/controllers/user_controller.rs
// Request Handlers
// ! Needs signout method which wipes cookies
use crate::models::user::{CreateUserPayload, LoginPayload, UpdateUserPayload, User};
use crate::{Error, Result};
use axum::routing::{get, post};
use axum::{
    extract::{Extension, Json, Path},
    Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower_cookies::{Cookies, Cookie};

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
        Err(_) => Err(Error::UserNotFoundError),
    }
}
/// POST handler for creating a new user.
/// Accessible via: POST /api/users
pub async fn api_create_user(
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
            // Then fetch the user by id
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
                    Err(Error::UserNotFoundError)
                }
            }
        }
        Err(e) => {
            println!("Error creating user: {:?}", e);
            Err(Error::UserCreationError)
        }
    }
}

pub async fn api_update_user(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<UpdateUserPayload>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - update_user", "HANDLER");

    // ! Need to check cookie here to get user id
    
    // temp user id

    let user_id = 1;

    // perform update
    let result = sqlx::query!(
        "UPDATE users
         SET name = $1, email = $2, password = $3
         WHERE id = $4;",
        payload.name,
        payload.email,
        payload.password,
        user_id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(_) => Ok(Json(json!({
            "result": {
                "success": true
            }
        }))),
        Err(_) => Err(Error::UserUpdateError),
    }
}

pub async fn api_user_login(
    cookies: Cookies, 
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<LoginPayload>
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login", "HANDLER");

    // Grab email and password from database and compare it to the payload
    let result = sqlx::query!(
        "SELECT email, password
         FROM users
         WHERE email = $1;",
         payload.email
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(record) => {
            if payload.email == record.email && payload.password == record.password {
                // Set cookie
                cookies.add(Cookie::new("auth-token", "user-1.exp.sign"));
                
                // Return success
                return Ok(Json(json!({
                    "result": true
                })));
            } else {
                return Err(Error::LoginFailError);
            }
        }
        Err(_) => return Err(Error::LoginFailError),
    }
}

/// Combine user-related routes into one Router instance.
pub fn user_routes() -> Router {
    Router::new()
        .route("/login", post(api_user_login))
        .route("/users", post(api_create_user))
        .route("/users/:id", get(get_user))
        .route("/users/:id", post(api_update_user))
}

