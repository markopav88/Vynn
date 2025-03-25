/*
/ src/controllers/user_controller.rs
/ Request Handlers
/
/ File containing various API Backend endpoints for manipulating a user
/
/ API Summary:
/
/
*/

use axum::routing::{get, post, put};
use axum::{
    extract::{Extension, Json, Path},
    Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower_cookies::cookie::time::Duration;
use tower_cookies::{Cookie, Cookies};

use crate::models::user::{CreateUserPayload, LoginPayload, UpdateUserPayload, User};
use crate::{Error, Result};
use backend::get_user_id_from_cookie;

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

    // Check for duplicate email
    let existing_user = sqlx::query!("SELECT id FROM users WHERE email = $1", payload.email)
        .fetch_optional(&pool)
        .await;

    // If a user with this email already exists, return an error
    match existing_user {
        Ok(Some(_)) => {
            return Err(Error::EmailAlreadyExistsError);
        }
        Ok(None) => {
            // Email is available, proceed with user creation
        }
        Err(e) => {
            println!("Error checking for existing user: {:?}", e);
            return Err(Error::DatabaseError);
        }
    }

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
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<UpdateUserPayload>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - update_user", "HANDLER");

    // Need to check cookie here to get user id
    let user_id = get_user_id_from_cookie(&cookies);

    // if we cant parse a user's id
    if user_id == None {
        return Err(Error::UserIdUpdateError);
    }

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
    .execute(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    // if the update doesnt affect any rows it failed
    if result.rows_affected() == 0 {
        return Err(Error::UserNotFoundError);
    }

    Ok(Json(json!({
        "result": {
            "success": true
        }
    })))
}

pub async fn api_user_login(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login", "HANDLER");

    // Grab email and password from database and compare it to the payload
    let result = sqlx::query!(
        "SELECT id, email, password
         FROM users
         WHERE email = $1;",
        payload.email
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(record) => {
            if payload.email == record.email && payload.password == record.password {
                // Get environment variables with fallbacks for development
                let domain = option_env!("DOMAIN").unwrap_or("localhost");
                let app_env = option_env!("APP_ENV").unwrap_or("development");
                let on_production = app_env == "production";

                // Create a token value (in a real app, this would be a JWT or similar)
                let token_value = format!("user-{}.exp.sign", record.id);

                // Build the cookie with enhanced security
                let cookie = Cookie::build("auth-token", token_value)
                    .domain(domain.to_string())
                    .path("/")
                    .secure(on_production)
                    .http_only(true)
                    .max_age(Duration::days(3))
                    .finish();

                // Add the cookie
                cookies.add(cookie);

                // Return success
                return Ok(Json(json!({
                    "result": {
                        "success": true,
                        "user_id": record.id
                    }
                })));
            } else {
                return Err(Error::LoginFailError);
            }
        }
        Err(_) => return Err(Error::LoginFailError),
    }
}

// Logout function to wipe user session and cookies
pub async fn api_logout(cookies: Cookies) -> Result<Json<Value>> {
    // Get environment variables with fallbacks for development
    let domain = option_env!("DOMAIN").unwrap_or("localhost");
    let app_env = option_env!("APP_ENV").unwrap_or("development");
    let on_production = app_env == "production";

    // Build a cookie with the same properties as the login cookie
    let cookie = Cookie::build("auth-token", "")
        .domain(domain.to_string())
        .path("/")
        .secure(on_production)
        .http_only(true)
        .max_age(Duration::days(0)) // Expire immediately
        .finish();

    // Remove the private cookie
    cookies.remove(cookie);

    return Ok(Json(json!({
        "result": {
            "success": true
        }
    })));
}

// Combine user-related routes into one Router instance.
pub fn user_routes() -> Router {
    Router::new()
        .route("/login", post(api_user_login))
        .route("/users", post(api_create_user))
        .route("/users/update", put(api_update_user))
        .route("/users/:id", get(get_user))
        .route("/users/logout", get(api_logout))
}
