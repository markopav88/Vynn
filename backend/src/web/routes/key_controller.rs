/*
/ src/controllers/key_conroller.rs
/ Request Handlers
/
/ File containing various API Backend endpoints for getting commands and manipulating user keybindings
/
/ API Summary:
/ api_get_all_commands          GET         /default    - Get all commands from database
/ api_get_all_keybindings       GET         /           - Get all users custom keybindings
/ api_add_update_keybinding     PUT         /:id        - Update or Add a keybinding by command id
/ api_delete_keybinding         DELETE      /:id        - Delete a keybinding by command id (Reset to Default)
/ api_reset_all_keybindings     DELETE      /reset      - Reset all user keybindings to default
*/

use axum::routing::{delete, get, put};
use axum::{
    extract::{Extension, Json, Path},
    Router,
};
use sqlx::PgPool;
use tower_cookies::Cookies;
use serde_json::{Value, json};

use crate::models::commands::{Command, UserKeybinding, UpdateKeybindingPayload};
use crate::{Error, Result};

use backend::get_user_id_from_cookie;

/// GET handler for retrieving all database-registered commands
/// Accessible via: GET /api/command/default
/// Test: test_documents.rs/test_get_all_commands()
/// Frontend: document.ts/get_all_commands()
pub async fn api_get_all_commands(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>
) -> Result<Json<Vec<Command>>> {
    println!("->> {:<12} - get_all_commands", "HANDLER");
    
    // get user_id from cookies
    let _user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Return all commands from database
    let commands = sqlx::query_as!(
        Command,
        "SELECT command_id, command_name, command_description, default_keybinding FROM commands"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(commands))
}

/// GET handler for retrieving all user keybindings
/// Accessible via: GET /api/command/
/// Test: test_documents.rs/test_get_all_keybindings()
/// Frontend: document.ts/get_all_keybindings()
pub async fn api_get_all_keybindings(
    cookies: Cookies, 
    Extension(pool): Extension<PgPool>
) -> Result<Json<Vec<UserKeybinding>>> {
    println!("->> {:<12} - get_all_keybindings", "HANDLER");

    // get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Return all custom keybindings owned by user in user keybindings table
    let keybindings = sqlx::query_as!(
        UserKeybinding,
        "SELECT user_id, command_id, keybinding FROM user_keybindings WHERE user_id = $1",
        user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(keybindings))
}

/// PUT handler for adding or updating a keybinding by command id
/// Accessible via: PUT /api/command/:id
/// Test: test_documents.rs/test_add_update_keybinding()
/// Frontend: document.ts/add_update_keybinding()
pub async fn api_add_update_keybinding(
    cookies: Cookies, 
    Path(command_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<UpdateKeybindingPayload>
) -> Result<Json<UserKeybinding>> {
    println!("->> {:<12} - add_update_keybinding", "HANDLER");

    // get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Upsert the keybinding (insert or update)
    let keybinding = sqlx::query_as!(
        UserKeybinding,
        "INSERT INTO user_keybindings (user_id, command_id, keybinding) 
         VALUES ($1, $2, $3)
         ON CONFLICT (user_id, command_id) 
         DO UPDATE SET keybinding = $3
         RETURNING user_id, command_id, keybinding",
        user_id,
        command_id,
        payload.keybinding
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| Error::AddUpdateKeybindingError { command_id })?;

    Ok(Json(keybinding))
}

/// DELETE handler for removing/resetting to default a user keybinding by command id.
/// Accessible via: DELETE /api/command/:id
/// Test: test_documents.rs/test_delete_keybinding()
/// Frontend: document.ts/delete_keybinding()
pub async fn api_delete_keybinding(
    cookies: Cookies, 
    Path(command_id): Path<i32>, 
    Extension(pool): Extension<PgPool>
) -> Result<Json<Command>> {
    println!("->> {:<12} - delete_keybinding", "HANDLER");

    // get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Delete from user keybindings table
    sqlx::query!(
        "DELETE FROM user_keybindings 
         WHERE user_id = $1 AND command_id = $2",
        user_id,
        command_id
    )
    .execute(&pool)
    .await
    .map_err(|_| Error::DeleteKeybindingError { command_id })?;

    // Return the default command associated with the deleted row
    let command = sqlx::query_as!(
        Command,
        "SELECT command_id, command_name, command_description, default_keybinding 
         FROM commands WHERE command_id = $1",
        command_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(command))
}

/// DELETE handler for resetting all user keybindings to default.
/// Accessible via: DELETE /api/command/reset
pub async fn api_reset_all_keybindings(
    cookies: Cookies, 
    Extension(pool): Extension<PgPool>
) -> Result<Json<Value>> {
    println!("->> {:<12} - reset_all_keybindings", "HANDLER");

    // Get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Delete all user keybindings
    sqlx::query!(
        "DELETE FROM user_keybindings WHERE user_id = $1",
        user_id
    )
    .execute(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(json!({
        "result": {
            "success": true,
            "message": "All keybindings reset to default successfully"
        }
    })))
}

pub fn key_routes() -> Router {
    Router::new()
        .route("/default", get(api_get_all_commands))
        .route("/", get(api_get_all_keybindings))
        .route("/reset", delete(api_reset_all_keybindings))
        .route("/:id", put(api_add_update_keybinding))
        .route("/:id", delete(api_delete_keybinding))
}