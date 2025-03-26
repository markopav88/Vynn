/*
/ src/controllers/proj_controller.rs
/ Request Handlers
/
/ File containing various API Backend endpoints for manipulating a project
/
/ API Summary:
/
/
*/

use crate::models::project::{CreateProjectPayload, Project, UpdateProjectPayload};
use axum::routing::{get, post, put};
use axum::{
    extract::{Extension, Json, Path},
    Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower_cookies::{Cookie, Cookies};

use crate::{Error, Result};
use backend::get_user_id_from_cookie;

async fn get_projects(cookies: Cookies, Extension(pool): Extension<PgPool>) -> Result<Json<Value>> {
    // get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    let result = sqlx::query!(
        r#"SELECT id, name, user_id 
           FROM projects
           WHERE user_id = $1"#,
        user_id
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(_) => {
            let projects_json: Vec<Value> = result
                .into_iter()
                .map(|project| {
                    json!({
                        "id": project.id,
                        "name": project.name,
                        "user_id": project.user_id,
                    })
                })
                .collect();

            Ok(Json(json!({ "project": projects_json })))
        }
        Err(_) => return Err(Error::ProjectNotFoundError),
    }
}

async fn api_get_project(
    cookies: Cookies,
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_get_project", "HANDLER");

    // get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    let result = sqlx::query!(
        r#"SELECT id, name, user_id
           FROM projects 
           WHERE id = $1 AND user_id = $2"#,
        id,
        user_id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(_) => {
            return Ok(Json(json!({
                "result": {
                    "id": result.id,
                    "name": result.name,
                    "user_id": result.user_id
                }
            })))
        }
        Err(_) => return Err(Error::ProjectNotFoundError),
    }
}

async fn api_create_project(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateProjectPayload>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;
    let result=sqlx::query!(
        r#"
        INSERT INTO projects (name,user_id)
        VALUES ($1, $2)
        RETURNING id,name,user_id;
        "#,
        payload.name,
        user_id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(_) => {
            return Ok(Json(json!({
                "result": {
                    "id": result.id,
                    "name": result.name,
                    "user_id": result.user_id
                }
            })))
        }
        Err(_) => return Err(Error::ProjectNotFoundError),
    }
    
        
    } 
        
    

//
async fn api_update_project(
    cookies: Cookies,
    //Grabing the id of the project from the path
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<UpdateProjectPayload>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_update_project", "HANDLER");
    //Result<Json<Value>> is the return type, meaning on success this function will produce a Json<Value> 
    //(Axum’s wrapper for returning JSON data), and on failure it will return an error variant (Err(...)).

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?; 
    //Validate inputs?
    /*if let Some(name) = &payload.name {
        if name.trim().is_empty() {
            // Return your error or custom error type
            // e.g. Err(Error::ProjectValidationError("Project name cannot be empty".into()))
            return Err(Error::ProjectNotFoundError);
        }
    }
    */



    let result= sqlx::query!(
        r#"
        UPDATE projects 
        SET name = $1
        WHERE id = $2 AND user_id = $3
        RETURNING id,name,user_id;
        "#,
        payload.name,
        id,
        user_id
    )
        .fetch_one(&pool)
    .await;

    match result {
        Ok(_) => {
            Ok(Json(json!({
                "result": {
                    "id": project.id,
                    "name": project.name,
                    "user_id": project.user_id
                }
            })))
        }
        Err(_) => return Err(Error::ProjectNotFoundError),
    }
}
    

async fn api_delete_project(
    cookies: Cookies,
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    println!("->> {:<12} - api_delete_project", "HANDLER");

    // Get user ID from cookie
    let user_id = match get_user_id_from_cookie(&cookies) {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Not authenticated").into_response(),
    };

    // delete project make sure user has permissions to do so
    match sqlx::query!(
        r#"DELETE FROM projects WHERE id = $1 AND owner_id = $2 RETURNING id"#,
        id,
        user_id
    )
    .fetch_one(&pool)
    .await
}FINISH

pub fn routes() -> Router {
    Router::new()
        .route("/", get(fetch_projects))
        .route("/:id", get(api_get_project))
        .route("/", post(api_create_project))
        .route("/:id", put(api_update_project))
        .route("/:id", delete(api_delete_project))
}
