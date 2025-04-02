/*
/ src/controllers/user_controller.rs
/ Request Handlers
/
/ File containing various API Backend endpoints for manipulating a user
/
/ API Summary:
/ api_create_user       POST    /users          - Create a New User In The Database
/ api_get_user          GET     /users          - Get Current User By Cookies
/ api_update_user       PUT     /users/update   - Update The Current User By Cookies
/ api_login             POST    /login          - Attempt Login And Set Cookies
/ api_logout            GET     /logout         - Logout By Wiping Cookies
/ api_check_auth        GET     /check-auth     - Check User Authentication
/
*/

use axum::routing::{get, post, put};
use axum::{
    extract::{Extension, Json},
    Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower_cookies::cookie::time::Duration;
use tower_cookies::{Cookie, Cookies};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2
};

use crate::models::user::{CreateUserPayload, LoginPayload, UpdateUserPayload, User};
use crate::{Error, Result};
use backend::get_user_id_from_cookie;

/// GET handler for retrieving a user by ID in cookies.
/// Accessible via: GET /api/users/:id
/// Test: test_users.rs/test_get_user()
/// Frontend: login.ts/get_current_user()
pub async fn api_get_user(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<User>> {
    println!("->> {:<12} - get_user", "HANDLER");

    // Need to check cookie here to get user id
    let user_id = get_user_id_from_cookie(&cookies);

    // if we cant parse a user's id
    if user_id == None {
        return Err(Error::UserIdUpdateError);
    }

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
/// Test: test_users.rs/test_create_user()
/// Frontend: signup.ts/attempt_signup()
pub async fn api_create_user(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<Json<User>> {
    println!("->> {:<12} - create_user", "HANDLER");

    // Check for duplicate email
    let existing_user = sqlx::query!("SELECT id FROM users WHERE email = $1", payload.email)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            println!("Error checking for existing user: {:?}", e);
            Error::DatabaseError
        })?;

    // If a user with this email already exists, return an error
    if existing_user.is_some() {
        return Err(Error::EmailAlreadyExistsError);
    }

    // Hash the password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| Error::UserCreationError)?
        .to_string();

    // Insert the user with hashed password
    let result = sqlx::query_as!(
        User,
        "INSERT INTO users (name, email, password) VALUES ($1, $2, $3) RETURNING id, name, email",
        payload.name,
        payload.email,
        password_hash
    )
    .fetch_one(&pool)
    .await;

    // Check if insertion was successful
    match result {
        Ok(user) => Ok(Json(user)),
        Err(e) => {
            println!("Error creating user: {:?}", e);
            Err(Error::UserCreationError)
        }
    }
}

/// PUT handler for updating a user.
/// Accessible via: PUT /api/users/update
/// Test: test_users.rs/test_update_user()
/// Frontend: login.ts/update_user()
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
    .await;

    // if the update doesnt affect any rows it failed
    if result.unwrap().rows_affected() == 0 {
        return Err(Error::UserNotFoundError);
    }

    // otherwise it passes
    Ok(Json(json!({
        "result": {
            "success": true
        }
    })))
}

/// POST handler for user login.
/// Accessible via: POST /api/login
/// Test: test_users.rs/test_good_login(), test_users.rs/test_bad_login()
/// Frontend: login.ts/attempt_login()
pub async fn api_login(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login", "HANDLER");

    // Get user from database
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
            // Verify password
            let parsed_hash = PasswordHash::new(&record.password)
                .map_err(|_| Error::LoginFailError)?;
            
            let password_verified = Argon2::default()
                .verify_password(payload.password.as_bytes(), &parsed_hash)
                .is_ok();

            if password_verified {
                // Create token and set cookie as before
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

/// GET handler for user logout.
/// Accessible via: GET /api/users/logout
/// Test: test_users.rs/test_logout()
/// Frontend: login.ts/logout()
pub async fn api_logout(cookies: Cookies) -> Result<Json<Value>> {
    println!("->> {:<12} - logout", "HANDLER");
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

/// Check if user is authenticated
/// Accessible via: GET /api/user/check-auth
/// Frontend: user.ts/check_auth()
pub async fn api_check_auth(
    cookies: Cookies,
    Extension(_pool): Extension<PgPool>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - check_auth", "HANDLER");

    // Try to get user_id from cookie
    let user_id = get_user_id_from_cookie(&cookies);
    
    // Return JSON with auth status
    match user_id {
        Some(_) => Ok(Json(json!({ "authenticated": true }))),
        None => Ok(Json(json!({ "authenticated": false }))),
    }
}

/// Upload or update a user's profile image
/// Accessible via: POST /api/users/profile-image
/// 
/// This endpoint accepts a multipart form with an "image" field containing the image file.
/// The image must be a valid image format (JPEG, PNG, etc.) and less than 5MB in size.
/// The user must be authenticated (have a valid auth-token cookie).
/// 
/// Returns a JSON response with success status and message.
pub async fn api_upload_profile_image(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<Value>> {
    println!("->> {:<12} - upload_profile_image", "HANDLER");
    
    // Get user id from cookie
    let user_id = match get_user_id_from_cookie(&cookies) {
        Some(id) => id,
        None => return Err(Error::UserIdUpdateError),
    };
    
    // Process the multipart form data with better error handling
    let mut image_data = Vec::new();
    let mut content_type = String::from("image/jpeg");
    
    // Add debug logging
    println!("->> {:<12} - processing multipart form", "DEBUG");
    
    while let Some(field) = match multipart.next_field().await {
        Ok(field) => field,
        Err(e) => {
            println!("->> {:<12} - multipart error: {:?}", "ERROR", e);
            return Err(Error::ProfilePicError);
        }
    } {
        let name = field.name().unwrap_or("").to_string();
        println!("->> {:<12} - processing field: {}", "DEBUG", name);
        
        if name == "image" {
            // Get content type
            content_type = field.content_type()
                .unwrap_or("image/jpeg")
                .to_string();
            
            // Check if it's an image
            if !content_type.starts_with("image/") {
                println!("->> {:<12} - invalid content type: {}", "ERROR", content_type);
                return Err(Error::ProfilePicError);
            }
            
            // Get file data with better error handling
            match field.bytes().await {
                Ok(bytes) => {
                    image_data = bytes.to_vec();
                    println!("->> {:<12} - received image of size: {} bytes", "DEBUG", image_data.len());
                },
                Err(e) => {
                    println!("->> {:<12} - failed to read bytes: {:?}", "ERROR", e);
                    return Err(Error::ProfilePicError);
                }
            }
            
            // Check file size (5MB limit)
            if image_data.len() > 5 * 1024 * 1024 {
                println!("->> {:<12} - image too large: {} bytes", "ERROR", image_data.len());
                return Err(Error::ProfilePicSizeError);
            }
        }
    }
    
    if image_data.is_empty() {
        println!("->> {:<12} - no image data received", "ERROR");
        return Err(Error::ProfilePicError);
    }
    
    // Upsert the image into the database
    let result = sqlx::query!(
        "INSERT INTO user_profile_images (user_id, image_data, content_type) 
         VALUES ($1, $2, $3)
         ON CONFLICT (user_id) 
         DO UPDATE SET image_data = $2, content_type = $3",
        user_id,
        image_data,
        content_type
    )
    .execute(&pool)
    .await;
    
    match result {
        Ok(_) => {
            println!("->> {:<12} - profile image updated successfully", "SUCCESS");
            Ok(Json(json!({
                "result": {
                    "success": true,
                    "message": "Profile image updated successfully"
                }
            })))
        },
        Err(e) => {
            println!("->> {:<12} - database error: {:?}", "ERROR", e);
            Err(Error::DatabaseError)
        },
    }
}

/// Get a user's profile image
/// Accessible via: GET /api/users/:id/profile-image
/// 
/// This endpoint returns the binary image data with the appropriate content-type header.
/// If the user has no profile image, it returns a 404 error.
/// 
/// The image can be used directly in HTML img tags:
/// <img src="/api/users/1/profile-image" alt="User profile" />
pub async fn api_get_profile_image(
    Extension(pool): Extension<PgPool>,
    axum::extract::Path(user_id): axum::extract::Path<i32>,
) -> Result<impl axum::response::IntoResponse> {
    println!("->> {:<12} - get_profile_image for user_id: {}", "HANDLER", user_id);
    
    // Validate user_id
    if user_id <= 0 {
        println!("->> {:<12} - invalid user_id: {}", "ERROR", user_id);
        return Err(Error::UserNotFoundError);
    }
    
    // Query the database for the user's profile image with better error handling
    let result = match sqlx::query!(
        "SELECT image_data, content_type FROM user_profile_images WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&pool)
    .await {
        Ok(row) => row,
        Err(e) => {
            println!("->> {:<12} - database error: {:?}", "ERROR", e);
            return Err(Error::DatabaseError);
        }
    };
    
    match result {
        Some(row) => {
            // Ensure image_data is not empty
            if row.image_data.is_empty() {
                println!("->> {:<12} - empty image data for user_id: {}", "ERROR", user_id);
                return Err(Error::ProfilePicError);
            }
            
            println!("->> {:<12} - returning image with content type: {}", "SUCCESS", row.content_type);
            
            // Return the image with the correct content type
            Ok((
                [(axum::http::header::CONTENT_TYPE, row.content_type)],
                row.image_data
            ))
        },
        None => {
            println!("->> {:<12} - no image found for user_id: {}", "ERROR", user_id);
            Err(Error::UserNotFoundError)
        }
    }
}

// Combine user-related routes into one Router instance.
pub fn user_routes() -> Router {
    Router::new()
        .route("/login", post(api_login))
        .route("/", post(api_create_user))
        .route("/update", put(api_update_user))
        .route("/:id", get(api_get_user))
        .route("/logout", get(api_logout))
        .route("/check-auth", get(api_check_auth))
        .route("/profile-image", post(api_upload_profile_image))
        .route("/:id/profile-image", get(api_get_profile_image))
}
