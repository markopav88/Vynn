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
use axum::routing::{get, post, put, delete};
use axum::{
    extract::{Extension, Json, Path},
    Router,
};
use sqlx::PgPool;
use tower_cookies::Cookies;

use crate::{Error, Result};
use backend::get_user_id_from_cookie;

async fn api_get_all_projects(cookies: Cookies, Extension(pool): Extension<PgPool>) -> Result<Json<Vec<Project>>> {
    // get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    let result = sqlx::query_as!(
        Project,
        r#"SELECT id, name, user_id 
           FROM projects
           WHERE user_id = $1"#,
        user_id
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(projects) => Ok(Json(projects)),
        Err(_) => Err(Error::ProjectNotFoundError),
    }
}

async fn api_get_project(
    cookies: Cookies,
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Project>> {
    println!("->> {:<12} - api_get_project", "HANDLER");

    // get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    let result = sqlx::query_as!(
        Project,
        r#"SELECT id, name, user_id
           FROM projects 
           WHERE id = $1 AND user_id = $2"#,
        id,
        user_id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(project) => Ok(Json(project)),
        Err(_) => return Err(Error::ProjectNotFoundError),
    }
}

async fn api_create_project(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateProjectPayload>,
) -> Result<Json<Project>> {
    println!("->> {:<12} - api_create_project", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;
    
    // Use query_as! to directly map to Project struct
    let result = sqlx::query_as!(
        Project,
        r#"
        INSERT INTO projects (name, user_id)
        VALUES ($1, $2)
        RETURNING id, name, user_id
        "#,
        payload._name,
        user_id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(project) => Ok(Json(project)),
        Err(_) => Err(Error::ProjectNotFoundError),
    }
}

//
async fn api_update_project(
    cookies: Cookies,
    //Grabing the id of the project from the path
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<UpdateProjectPayload>,
) -> Result<Json<Project>> {
    println!("->> {:<12} - api_update_project", "HANDLER");
    //Result<Json<Value>> is the return type, meaning on success this function will produce a Json<Value>
    //(Axum's wrapper for returning JSON data), and on failure it will return an error variant (Err(...)).

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    let result = sqlx::query_as!(
        Project,
        r#"
        UPDATE projects 
        SET name = $1
        WHERE id = $2 AND user_id = $3
        RETURNING id,name,user_id;
        "#,
        payload._name,
        id,
        user_id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(project) => Ok(Json(project)),
        Err(_) => return Err(Error::ProjectNotFoundError),
    }
}

async fn api_delete_project(
    cookies: Cookies,
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Project>> {
    println!("->> {:<12} - api_delete_project", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // ! Confirm user ID is owner of the project

    // delete the project id
    let result = sqlx::query_as!(
        Project,
        r#"DELETE FROM projects 
        WHERE id = $1 and user_id = $2
        RETURNING id, name, user_id;
        "#,
        id,
        user_id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(project) => Ok(Json(project)),
        Err(_) => Err(Error::ProjectNotFoundError),
    }
}

pub fn project_routes() -> Router {
    Router::new()
        .route("/", get(api_get_all_projects))
        .route("/", post(api_create_project))
        .route("/:id", get(api_get_project))
        .route("/:id", put(api_update_project))
        .route("/:id", delete(api_delete_project))
}
