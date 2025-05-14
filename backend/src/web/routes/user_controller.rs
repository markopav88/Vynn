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
    extract::{Extension, Json, Path},
    Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower_cookies::cookie::time::Duration;
use tower_cookies::cookie::SameSite;
use tower_cookies::{Cookie, Cookies};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2
};
use std::sync::OnceLock;

use crate::models::user::{CreateUserPayload, LoginUserPayload, UpdateUserPayload, User};
use crate::models::storage::StorageManager;
use crate::{Error, Result};
use backend::get_user_id_from_cookie;

// Define a static variable to hold the default profile image data
static DEFAULT_PROFILE_IMAGE: OnceLock<(Vec<u8>, String)> = OnceLock::new();

// Function to get the default profile image data
fn get_default_profile_image() -> (Vec<u8>, String) {
    DEFAULT_PROFILE_IMAGE.get_or_init(|| {
        // Hardcoded simple SVG data for a user avatar
        let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 100\" width=\"100\" height=\"100\"><circle cx=\"50\" cy=\"35\" r=\"25\" fill=\"#10B981\"/><circle cx=\"50\" cy=\"100\" r=\"40\" fill=\"#10B981\"/></svg>";
        
        let image_data = svg.as_bytes().to_vec();
        let content_type = "image/svg+xml".to_string();
        (image_data, content_type)
    }).clone()
}

/// GET handler for retrieving a user by ID provided in the path.
/// Accessible via: GET /api/users/:id
/// Test: test_users.rs/test_get_user()
/// Frontend: // TODO: No direct frontend call, user info usually fetched via /current
pub async fn api_get_user(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<User>> {
    println!("->> {:<12} - get_user (by ID: {})", "HANDLER", id);

    let result = sqlx::query_as!(
        User,
        r#"SELECT id, name, email, password, ai_credits, 
           NULL::BIGINT as storage_bytes, 
           NULL::INT as max_projects, 
           NULL::INT as max_documents 
           FROM users WHERE id = $1"#,
        id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(Error::UserNotFoundError { user_id: id }),
    }
}

/// POST handler for creating a new user.
/// Accessible via: POST /api/users
/// Test: test_users.rs/test_create_user()
/// Frontend: user.ts/attempt_signup()
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
    
    // Validate password complexity
    if payload.password.is_empty() {
        println!("->> {:<12} - empty password not allowed", "ERROR");
        return Err(Error::PasswordValidationError);
    }
    
    // Password complexity requirements:
    // 1. Minimum length of 8 characters
    if payload.password.len() < 8 {
        println!("->> {:<12} - password too short, minimum 8 characters required", "ERROR");
        return Err(Error::PasswordValidationError);
    }
    
    // 2. Contains at least one uppercase letter
    if !payload.password.chars().any(|c| c.is_uppercase()) {
        println!("->> {:<12} - password must contain at least one uppercase letter", "ERROR");
        return Err(Error::PasswordValidationError);
    }
    
    // 3. Contains at least one number
    if !payload.password.chars().any(|c| c.is_numeric()) {
        println!("->> {:<12} - password must contain at least one number", "ERROR");
        return Err(Error::PasswordValidationError);
    }
    
    // 4. Contains at least one special character
    if !payload.password.chars().any(|c| !c.is_alphanumeric()) {
        println!("->> {:<12} - password must contain at least one special character", "ERROR");
        return Err(Error::PasswordValidationError);
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
        r#"INSERT INTO users (name, email, password) 
           VALUES ($1, $2, $3) 
           RETURNING id, name, email, password, ai_credits, 
           NULL::BIGINT as storage_bytes, 
           NULL::INT as max_projects, 
           NULL::INT as max_documents"#,
        payload.name,
        payload.email,
        password_hash
    )
    .fetch_one(&pool)
    .await;

    // Check if insertion was successful
    match result {
        Ok(user) => {
            // Add default profile image for the new user
            let (default_image_data, default_content_type) = get_default_profile_image();
            
            // Insert the default profile image
            let _image_result = sqlx::query!(
                "INSERT INTO user_profile_images (user_id, image_data, content_type) 
                 VALUES ($1, $2, $3)
                 ON CONFLICT (user_id) DO NOTHING",
                user.id,
                default_image_data,
                default_content_type
            )
            .execute(&pool)
            .await;
            
            // Return the user info even if profile image insertion fails
            Ok(Json(user))
        },
        Err(e) => {
            println!("Error creating user: {:?}", e);
            Err(Error::UserCreationError)
        }
    }
}

/// PUT handler for updating the current user.
/// Accessible via: PUT /api/users/update
/// Test: test_users.rs/test_update_user()
/// Frontend: user.ts/update_user()
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
    
    // Validate password complexity
    if payload.password.is_none() || payload.password.as_ref().unwrap().is_empty() {
        // perform update without password
        let result = sqlx::query!(
            "UPDATE users
            SET name = $1, email = $2
            WHERE id = $3;",
            payload.name,
            payload.email,
            user_id
        )
        .execute(&pool)
        .await;

        // if the update doesnt affect any rows it failed
        if result.unwrap().rows_affected() == 0 {
            return Err(Error::UserNotFoundError { user_id: user_id.unwrap() });
        }

        // otherwise it passes
        return Ok(Json(json!({
            "result": {
                "success": true
            }
        })));
    }
    
    let password = payload.password.as_ref().unwrap();
    
    // Password complexity requirements:
    // 1. Minimum length of 8 characters
    if password.len() < 8 {
        println!("->> {:<12} - password too short, minimum 8 characters required", "ERROR");
        return Err(Error::PasswordValidationError);
    }
    
    // 2. Contains at least one uppercase letter
    if !password.chars().any(|c| c.is_uppercase()) {
        println!("->> {:<12} - password must contain at least one uppercase letter", "ERROR");
        return Err(Error::PasswordValidationError);
    }
    
    // 3. Contains at least one number
    if !password.chars().any(|c| c.is_numeric()) {
        println!("->> {:<12} - password must contain at least one number", "ERROR");
        return Err(Error::PasswordValidationError);
    }
    
    // 4. Contains at least one special character
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        println!("->> {:<12} - password must contain at least one special character", "ERROR");
        return Err(Error::PasswordValidationError);
    }
    
    // Hash the password before updating
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            println!("->> {:<12} - password hashing error: {:?}", "ERROR", e);
            Error::UserUpdateError { user_id: user_id.unwrap() }
        })?
        .to_string();

    // perform update with hashed password
    let result = sqlx::query!(
        "UPDATE users
         SET name = $1, email = $2, password = $3
         WHERE id = $4;",
        payload.name,
        payload.email,
        password_hash,
        user_id
    )
    .execute(&pool)
    .await;

    // if the update doesnt affect any rows it failed
    if result.unwrap().rows_affected() == 0 {
        return Err(Error::UserNotFoundError { user_id: user_id.unwrap() });
    }

    // otherwise it passes
    Ok(Json(json!({
        "result": {
            "success": true
        }
    })))
}

/// POST handler for user login.
/// Accessible via: POST /api/users/login
/// Test: test_users.rs/test_good_login(), test_users.rs/test_bad_login()
/// Frontend: user.ts/attempt_login()
pub async fn api_login(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<LoginUserPayload>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login", "HANDLER");
    println!("Received login request for email: {}", payload.email);

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
            println!("User found: {:?}", record); // Log the user record

            // Verify password
            let parsed_hash = PasswordHash::new(&record.password)
                .map_err(|_| {
                    println!("Failed to parse password hash for user: {}", record.email);
                    Error::LoginFailError
                })?;
            
            let password_verified = Argon2::default()
                .verify_password(payload.password.as_bytes(), &parsed_hash)
                .is_ok();

            if password_verified {
                println!("Password verified for user: {}", record.email);

                // Create token and set cookie as before
                let _domain = option_env!("DOMAIN").unwrap_or("localhost");
                let app_env = option_env!("APP_ENV").unwrap_or("development");
                let on_production = app_env == "production";

                // Create a token value (in a real app, this would be a JWT or similar)
                let token_value = format!("user-{}.exp.sign", record.id);
                let token_for_cookie = token_value.clone();

                println!("Generated token value: {}", token_value);
                println!("Production is: {}", on_production);

                // Build the cookie with enhanced security
                let cookie = Cookie::build("auth-token", token_value)
                    //.domain(domain.to_string())
                    .path("/")
                    .secure(on_production)
                    .http_only(true)
                    .same_site(if on_production { 
                        SameSite::None  // For cross-origin in production 
                    } else { 
                        SameSite::Lax   // For local development
                    })
                    .max_age(Duration::days(3))
                    .finish();

                // Add the cookie
                cookies.add(cookie);

                // Return success
                return Ok(Json(json!({
                    "result": {
                        "success": true,
                        "user_id": record.id,
                        "token": token_for_cookie
                    }
                })));

            } else {
                println!("Password verification failed for user: {}", record.email);
                return Err(Error::LoginFailError);
            }
        }
        Err(_) => {
            println!("No user found with email: {}", payload.email);
            return Err(Error::LoginFailError);
        },
    }
}

/// GET handler for user logout.
/// Accessible via: GET /api/users/logout
/// Test: test_users.rs/test_logout()
/// Frontend: user.ts/logout()
pub async fn api_logout(cookies: Cookies) -> Result<Json<Value>> {
    println!("->> {:<12} - logout", "HANDLER");
    // Get environment variables with fallbacks for development
    let _domain = option_env!("DOMAIN").unwrap_or("localhost");
    let app_env = option_env!("APP_ENV").unwrap_or("development");
    let on_production = app_env == "production";

    // Build a cookie with the same properties as the login cookie
    let cookie = Cookie::build("auth-token", "")
        //.domain(domain.to_string())
        .path("/")
        .secure(on_production)
        .http_only(true)
        .same_site(if on_production { 
            SameSite::None  // For cross-origin in production 
        } else { 
            SameSite::Lax   // For local development
        })
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

/// GET handler to check if user is authenticated via cookie.
/// Accessible via: GET /api/users/check-auth
/// Test: TODO: test_users.rs/test_check_auth()
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

/// POST handler to upload or update a user's profile image.
/// Accessible via: POST /api/users/profile-image
/// Test: TODO: test_users.rs/test_upload_profile_image()
/// Frontend: user.ts/upload_profile_image()
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
        
        if name == "profile_image" {
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

/// GET handler to retrieve a user's profile image.
/// Accessible via: GET /api/users/:id/profile-image
/// Test: TODO: test_users.rs/test_get_profile_image()
/// Frontend: user.ts/get_profile_image_url()
/// 
/// This endpoint returns the binary image data with the appropriate content-type header.
/// If the user has no profile image, it returns a default image.
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
        return Err(Error::UserNotFoundError { user_id });
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
                // Return default image instead of error
                let (default_image, default_content_type) = get_default_profile_image();
                return Ok((
                    [(axum::http::header::CONTENT_TYPE, default_content_type)],
                    default_image
                ));
            }
            
            println!("->> {:<12} - returning image with content type: {}", "SUCCESS", row.content_type);
            
            // Return the image with the correct content type
            Ok((
                [(axum::http::header::CONTENT_TYPE, row.content_type)],
                row.image_data
            ))
        },
        None => {
            println!("->> {:<12} - no image found for user_id: {}, returning default", "INFO", user_id);
            // Return default image when no profile image exists
            let (default_image, default_content_type) = get_default_profile_image();
            Ok((
                [(axum::http::header::CONTENT_TYPE, default_content_type)],
                default_image
            ))
        }
    }
}

/// GET handler to search users by email.
/// Accessible via: GET /api/users/search?q=search_term
/// Test: TODO: test_users.rs/test_search_users() - Test missing
/// Frontend: // TODO: No frontend function implemented yet
/// Returns a list of users matching the search term.
pub async fn api_search_users(
    Extension(pool): Extension<PgPool>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<User>>> {
    println!("->> {:<12} - search_users", "HANDLER");

    // Get search term from query params
    let search_term = params.get("q").unwrap_or(&"".to_string()).to_string();

    // If search term is empty, return empty list
    if search_term.is_empty() {
        return Ok(Json(vec![]));
    }

    // Search for users with email containing the search term
    let users = sqlx::query_as!(
        User,
        r#"SELECT id, name, email, password, ai_credits, 
           NULL::BIGINT as storage_bytes, 
           NULL::INT as max_projects, 
           NULL::INT as max_documents 
           FROM users WHERE email ILIKE $1 ORDER BY email LIMIT 10"#,
        format!("%{}%", search_term)
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(users))
}

/// GET handler for retrieving the current logged-in user's information.
/// Accessible via: GET /api/users/current
/// Test: test_users.rs/test_get_current_user()
/// Frontend: user.ts/get_current_user()
pub async fn api_get_current_user(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<User>> {
    println!("->> {:<12} - get_current_user", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::UserIdUpdateError)?;

    let result = sqlx::query_as!(
        User,
        r#"SELECT id, name, email, password, ai_credits, 
           NULL::BIGINT as storage_bytes, 
           NULL::INT as max_projects, 
           NULL::INT as max_documents 
           FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(Error::UserNotFoundError { user_id }),
    }
}

/// GET handler for retrieving the user's storage usage data.
/// Accessible via: GET /api/users/storage
/// Frontend: user.ts/get_storage_usage()
pub async fn api_get_storage_usage(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - get_storage_usage", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::UserIdUpdateError)?;
    
    // Calculate document storage
    // We'll count characters in content as a proxy for storage space (1 char = ~1-4 bytes)
    let document_storage = sqlx::query!(
        r#"
        SELECT SUM(LENGTH(COALESCE(content, ''))) as total_size
        FROM documents d
        JOIN document_permissions dp ON d.id = dp.document_id
        WHERE dp.user_id = $1
        "#,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;
    
    // Count number of documents
    let document_count = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM documents d
        JOIN document_permissions dp ON d.id = dp.document_id
        WHERE dp.user_id = $1
        "#,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;
    
    // Count number of projects
    let project_count = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM projects p
        JOIN project_permissions pp ON p.id = pp.project_id
        WHERE pp.user_id = $1
        "#,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;
    
    // Convert document content size to megabytes (assuming 1 char ≈ 1 byte for simplicity)
    let size_bytes = document_storage.total_size.unwrap_or(0) as f64;
    let size_mb = size_bytes / (1024.0 * 1024.0);
    
    // Calculate storage usage percentage (assuming 10GB limit)
    let max_storage_gb = 10.0;
    let size_gb = size_mb / 1024.0;
    let usage_percentage = (size_gb / max_storage_gb) * 100.0;
    
    Ok(Json(json!({
        "used_bytes": size_bytes,
        "used_mb": size_mb,
        "used_gb": size_gb,
        "max_storage_gb": max_storage_gb,
        "usage_percentage": usage_percentage,
        "document_count": document_count.count,
        "project_count": project_count.count
    })))
}

/// GET handler for retrieving a user's storage usage and limits.
/// Accessible via: GET /api/user/storage
/// Frontend: user.ts/get_user_storage()
pub async fn api_get_user_storage(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - get_user_storage", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Get project and document counts
    let project_count = sqlx::query!(
        r#"SELECT COUNT(*) as count FROM projects p 
           JOIN project_permissions pp ON p.id = pp.project_id 
           WHERE pp.user_id = $1 AND pp.role = 'owner'"#,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| Error::UserNotFoundError { user_id })?;
    
    let document_count = sqlx::query!(
        r#"SELECT COUNT(*) as count FROM documents d 
           JOIN document_permissions dp ON d.id = dp.document_id 
           WHERE dp.user_id = $1 AND dp.role = 'owner'"#,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| Error::UserNotFoundError { user_id })?;
    
    // Calculate storage bytes (sum of document content lengths) - with precise character counting
    let storage_bytes = sqlx::query!(
        r#"SELECT COALESCE(SUM(LENGTH(COALESCE(d.content, ''))), 0) as total_bytes
           FROM documents d
           JOIN document_permissions dp ON d.id = dp.document_id
           WHERE dp.user_id = $1 AND dp.role = 'owner'"#,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    // Get dynamic storage limits
    let max_projects = 3; // Project limit remains fixed
    let max_documents = 10; // Document limit remains fixed
    let max_storage_bytes = StorageManager::get_user_quota();
    
    // Get overall database statistics
    let db_size = StorageManager::get_db_size(&pool).await.unwrap_or(0);
    let db_total = StorageManager::get_total_db_allocated();
    let db_usage_percentage = StorageManager::get_db_usage_percentage(&pool).await.unwrap_or(0.0);
    
    // Calculate ultra-precise percentages
    let storage_percentage = (storage_bytes.total_bytes.unwrap_or(0) as f64 / max_storage_bytes as f64) * 100.0;
    let projects_percentage = (project_count.count.unwrap_or(0) as f64 / max_projects as f64) * 100.0;
    let documents_percentage = (document_count.count.unwrap_or(0) as f64 / max_documents as f64) * 100.0;
    
    // Return the storage usage information with detailed byte-level precision
    Ok(Json(json!({
        // Raw byte counts for maximum precision
        "storage_bytes": storage_bytes.total_bytes,
        "max_storage_bytes": max_storage_bytes,
        
        // Database overview
        "database_info": {
            "total_size_bytes": db_total,
            "total_size_gb": format!("{:.6}", db_total as f64 / (1024.0 * 1024.0 * 1024.0)),
            "used_bytes": db_size,
            "used_percentage": format!("{:.6}", db_usage_percentage)
        },
        
        // Formatted values for different units
        "storage_bytes_formatted": {
            "bytes": storage_bytes.total_bytes.unwrap_or(0),
            "kb": format!("{:.10}", storage_bytes.total_bytes.unwrap_or(0) as f64 / 1024.0),
            "mb": format!("{:.10}", storage_bytes.total_bytes.unwrap_or(0) as f64 / (1024.0 * 1024.0)),
            "gb": format!("{:.10}", storage_bytes.total_bytes.unwrap_or(0) as f64 / (1024.0 * 1024.0 * 1024.0))
        },
        
        // Counts and limits
        "max_projects": max_projects,
        "max_documents": max_documents,
        "project_count": project_count.count,
        "document_count": document_count.count,
        
        // Percentages with extreme precision for even the tiniest storage usage
        "storage_percentage": storage_percentage,
        "projects_percentage": projects_percentage,
        "documents_percentage": documents_percentage
    })))
}

// Combine user-related routes into one Router instance.
pub fn user_routes() -> Router {
    Router::new()
        .route("/", post(api_create_user))
        .route("/login", post(api_login))
        .route("/logout", post(api_logout))
        .route("/:id", get(api_get_user))
        .route("/:id", put(api_update_user))
        .route("/profile-pic/:id", get(api_get_profile_image))
        .route("/profile-pic", post(api_upload_profile_image))
        .route("/current", get(api_get_current_user))
        .route("/storage", get(api_get_storage_usage))
        .route("/user-storage", get(api_get_user_storage))
}
