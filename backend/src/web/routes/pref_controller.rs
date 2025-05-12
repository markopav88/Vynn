/*
/ src/controllers/pref_controller.rs
/ Request Handlers
/
/ File containing various API Backend endpoints for manipulating user preferences
/
/ API Summary:
/ api_get_all_preferences     GET     /                - Get all preferences for current user
/ api_get_preference          GET     /:id             - Get a specific preference for current user
/ api_update_preference       PUT     /:id             - Update a specific preference for current user
/ api_reset_preference        DELETE  /:id             - Reset a specific preference to default value
/ api_reset_all_preferences   DELETE  /                - Reset all preferences to default values
/ api_upload_background_image POST    /background        - Upload a background image for the current user
/ api_get_background_image    GET     /background        - Get the current user's background image
/ api_delete_background_image DELETE  /background        - Delete the current user's background image
/
*/

use axum::{
    extract::{Extension, Json, Path, Multipart},
    routing::{delete, get, put, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower_cookies::Cookies;

use crate::{Error, Result};
use backend::get_user_id_from_cookie;

#[derive(Debug, Serialize, Deserialize)]
pub struct Preference {
    pub preference_id: i32,
    pub preference_name: String,
    pub preference_value: String,
    pub preference_description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePreferencePayload {
    pub preference_value: String,
}

// Function to get the default background image data
fn get_default_background_image() -> Vec<u8> {
    // Assuming the default image is stored as bytes in the project
    let default_image = include_bytes!("../../../../frontend/src/lib/assets/editor-background.jpg").to_vec();
    default_image
}

/// GET handler for retrieving all preferences for the current user.
/// Accessible via: GET /api/preference/
/// Test: TODO: Add tests
/// Frontend: TODO: Add frontend call
pub async fn api_get_all_preferences(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<Preference>>> {
    println!("->> {:<12} - get_all_preferences", "HANDLER");

    // Get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Get all preferences for the user, including defaults where user preferences don't exist
    let preferences = sqlx::query_as!(
        Preference,
        r#"
        SELECT 
            dp.preference_id,
            dp.preference_name as "preference_name!",
            COALESCE(up.preference_value, dp.preference_value) as "preference_value!",
            dp.preference_description as "preference_description!"
        FROM 
            default_preferences dp
        LEFT JOIN 
            user_preferences up ON dp.preference_id = up.preference_id AND up.user_id = $1
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(preferences))
}

/// GET handler for retrieving a specific preference for the current user.
/// Accessible via: GET /api/preference/:id
/// Test: TODO: Add tests
/// Frontend: TODO: Add frontend call
pub async fn api_get_preference(
    cookies: Cookies,
    Path(preference_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Preference>> {
    println!("->> {:<12} - get_preference", "HANDLER");

    // Get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Get the specific preference for the user, or the default if user preference doesn't exist
    let preference = sqlx::query_as!(
        Preference,
        r#"
        SELECT 
            dp.preference_id,
            dp.preference_name as "preference_name!",
            COALESCE(up.preference_value, dp.preference_value) as "preference_value!",
            dp.preference_description as "preference_description!"
        FROM 
            default_preferences dp
        LEFT JOIN 
            user_preferences up ON dp.preference_id = up.preference_id AND up.user_id = $1
        WHERE 
            dp.preference_id = $2
        "#,
        user_id,
        preference_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?
    .ok_or(Error::PreferenceNotFoundError { preference_id })?;

    Ok(Json(preference))
}

/// PUT handler for updating a specific preference for the current user.
/// Accessible via: PUT /api/preference/:id
/// Test: TODO: Add tests
/// Frontend: TODO: Add frontend call
pub async fn api_update_preference(
    cookies: Cookies,
    Path(preference_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<UpdatePreferencePayload>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - update_preference", "HANDLER");

    // Get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Check if the preference exists
    let preference_exists = sqlx::query!(
        "SELECT 1 as exists FROM default_preferences WHERE preference_id = $1",
        preference_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    if preference_exists.is_none() {
        return Err(Error::PreferenceNotFoundError { preference_id });
    }

    // Upsert the user preference
    sqlx::query!(
        r#"
        INSERT INTO user_preferences (user_id, preference_id, preference_value)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, preference_id) 
        DO UPDATE SET preference_value = $3
        "#,
        user_id,
        preference_id,
        payload.preference_value
    )
    .execute(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(json!({
        "result": {
            "success": true,
            "message": "Preference updated successfully"
        }
    })))
}

/// DELETE handler for resetting a specific preference to its default value.
/// Accessible via: DELETE /api/preference/:id
/// Test: TODO: Add tests
/// Frontend: TODO: Add frontend call
pub async fn api_reset_preference(
    cookies: Cookies,
    Path(preference_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - reset_preference", "HANDLER");

    // Get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Check if the preference exists
    let preference_exists = sqlx::query!(
        "SELECT 1 as exists FROM default_preferences WHERE preference_id = $1",
        preference_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    if preference_exists.is_none() {
        return Err(Error::PreferenceNotFoundError { preference_id });
    }

    // Delete the user preference to revert to default
    sqlx::query!(
        "DELETE FROM user_preferences WHERE user_id = $1 AND preference_id = $2",
        user_id,
        preference_id
    )
    .execute(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(json!({
        "result": {
            "success": true,
            "message": "Preference reset to default successfully"
        }
    })))
}

/// DELETE handler for resetting all preferences to their default values.
/// Accessible via: DELETE /api/preference/
/// Test: TODO: Add tests
/// Frontend: TODO: Add frontend call
pub async fn api_reset_all_preferences(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - reset_all_preferences", "HANDLER");

    // Get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Delete all user preferences to revert to defaults
    sqlx::query!(
        "DELETE FROM user_preferences WHERE user_id = $1",
        user_id
    )
    .execute(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(json!({
        "result": {
            "success": true,
            "message": "All preferences reset to defaults successfully"
        }
    })))
}

/// POST handler for uploading a background image
/// Accessible via: POST /api/preference/background
/// Test: TODO: Add tests
/// Frontend: TODO: Add frontend call
pub async fn api_upload_background_image(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
    mut multipart: Multipart,
) -> Result<Json<Value>> {
    println!("->> {:<12} - upload_background_image", "HANDLER");

    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    let mut background_image_data = Vec::new();
    let mut content_type = String::new();

    while let Some(field) = multipart.next_field().await.map_err(|_| Error::InvalidRequestFormatError)? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "background_image" {
            content_type = field.content_type().unwrap_or("image/jpeg").to_string();
            background_image_data = field.bytes().await.map_err(|_| Error::InvalidRequestFormatError)?.to_vec();
        }
    }

    if background_image_data.is_empty() {
        return Err(Error::InvalidRequestFormatError);
    }

    // Store the image data in the database
    sqlx::query!(
        r#"
        INSERT INTO user_backgrounds (user_id, image_data, content_type)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id) 
        DO UPDATE SET image_data = $2, content_type = $3
        "#,
        user_id,
        background_image_data,
        content_type
    )
    .execute(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(json!({
        "result": {
            "success": true,
            "message": "Background image uploaded successfully"
        }
    })))
}

/// GET handler for retrieving a background image
/// Accessible via: GET /api/preference/background
/// If no user_id is provided, returns the current user's background
/// Test: TODO: Add tests
/// Frontend: TODO: Add frontend call
pub async fn api_get_background_image(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>
) -> Result<Json<Vec<u8>>> {
    println!("->> {:<12} - get_background_image", "HANDLER");

    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Retrieve the background image data from the database
    let background_data = sqlx::query!(
        "SELECT image_data, content_type FROM user_backgrounds WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    if let Some(data) = background_data {
        let file_bytes = data.image_data;

        // Return the image data as JSON
        Ok(Json(file_bytes))
    } else {
        // Use the function to get the default background image
        let default_image = get_default_background_image();
        Ok(Json(default_image))
    }
}

/// DELETE handler for deleting a background image
/// Accessible via: DELETE /api/preference/background
/// Test: TODO: Add tests
/// Frontend: TODO: Add frontend call
pub async fn api_delete_background_image(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - delete_background_image", "HANDLER");

    // Get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Delete background from database
    sqlx::query!(
        "DELETE FROM user_backgrounds WHERE user_id = $1",
        user_id
    )
    .execute(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(json!({
        "result": {
            "success": true,
            "message": "Background image deleted successfully"
        }
    })))
}

pub fn pref_routes() -> Router {
    Router::new()
        .route("/", get(api_get_all_preferences))
        .route("/", delete(api_reset_all_preferences))
        .route("/background", get(api_get_background_image))
        .route("/background", post(api_upload_background_image))
        .route("/background", delete(api_delete_background_image))
        .route("/:id", get(api_get_preference))
        .route("/:id", put(api_update_preference))
        .route("/:id", delete(api_reset_preference))
} 