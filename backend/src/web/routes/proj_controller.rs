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
use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};
use http::Error;
use serde_json::json;
use sqlx::PgPool;
use tower_cookies::Cookies;

async fn get_project(cookies: Cookies, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    // Get user ID from cookie
    let user_id = match get_user_id_from_cookie(&cookies) {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Not authenticated").into_response(),
    };

    match sqlx::query!(
        r#"SELECT id, name, description, owner_id, created_at, updated_at 
           FROM projects 
           WHERE owner_id = $1
           ORDER BY created_at DESC"#,
        user_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => {
            let projects: Vec<Project> = rows
                .into_iter()
                .filter_map(|row| {
                    // Only include rows where ownider_id is not null
                    let owner_id = row.owner_id?;

                    Some(Project {
                        id: row.id,
                        name: row.name,
                        description: row.description,
                        owner_id,
                        created_at: row.created_at.to_utc_datetime(),
                        updated_at: row.updated_at.to_utc_datetime(),
                    })
                })
                .collect();
            Json(projects).into_response()
        }
        Err(e) => {
            println!("Error fetching projects: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch projects: {}", e),
            )
                .into_response()
        }
    }
}

async fn api_get_project(
    cookies: Cookies,
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_get_project", "HANDLER");

    // Get user ID from cookie
    let user_id = match get_user_id_from_cookie(&cookies) {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Not authenticated").into_response(),
    };

    match sqlx::query!(
        r#"SELECT id, name, description, owner_id, created_at, updated_at 
           FROM projects 
           WHERE id = $1 AND owner_id = $2"#,
        id,
        user_id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => {
            if let Some(owner_id) = row.owner_id {
                let project = Project {
                    id: row.id,
                    name: row.name,
                    description: row.description,
                    owner_id,
                    created_at: row.created_at.to_utc_datetime(),
                    updated_at: row.updated_at.to_utc_datetime(),
                };
                Json(project).into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Invalid project data: missing owner_id",
                )
                    .into_response()
            }
        }
        Err(e) => {
            println!("Error fetching project: {:?}", e);
            (
                StatusCode::NOT_FOUND,
                "Project not found or you don't have access",
            )
                .into_response()
        }
    }
}

async fn api_create_project(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateProjectPayload>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login", "HANDLER");

    // Get user ID from cookie
    let owner_id = match get_user_id_from_cookie(&cookies) {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Not authenticated").into_response(),
    };

    match sqlx::query!(
        r#"
        INSERT INTO projects (name, description, owner_id)
        VALUES ($1, $2, $3)
        RETURNING id, name, description, owner_id, created_at, updated_at
        "#,IntoResponse
        payload.name,
        payload.description,
        owner_id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => {
            if let Some(owner_id) = row.owner_id {
                let project = Project {
                    id: row.id,
                    name: row.name,
                    description: row.description,
                    owner_id,
                    created_at: row.created_at.to_utc_datetime(),
                    updated_at: row.updated_at.to_utc_datetime(),
                };
                Json(project).into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to create project: owner_id is null",
                )
                    .into_response()
            }
        }
        Err(e) => {
            println!("Error creating project: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create project: {}", e),
            )
                .into_response()
        }
    }
}

async fn api_update_project(
    cookies: Cookies,
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<UpdateProjectPayload>,
) -> impl IntoResponse {
    println!("->> {:<12} - api_update_project", "HANDLER");

    // Get user ID from cookie
    let user_id = match get_user_id_from_cookie(&cookies) {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, "Not authenticated").into_response(),
    };

    // Validate inputs
    if let Some(name) = &payload.name {
        if name.trim().is_empty() {
            return (StatusCode::BAD_REQUEST, "Project name cannot be empty").into_response();
        }
    }

    // Get the current project first and verify ownership
    let current_project = sqlx::query!(
        r#"SELECT id, name, description, owner_id, created_at, updated_at 
           FROM projects 
           WHERE id = $1 AND owner_id = $2"#,
        id,
        user_id
    )
    .fetch_one(&pool)
    .await;

    // lets match the result of what we got for looking for the project
    match current_project {
        Ok(project) => {
            // Update with new values or keep the old ones
            let name = payload.name.unwrap_or(project.name);
            let description = payload.description.or(project.description);

            // Now lets query to try to update
            match sqlx::query!(
                r#"
                UPDATE projects
                SET name = $1, description = $2, updated_at = now()
                WHERE id = $3 AND owner_id = $4
                RETURNING id, name, description, owner_id, created_at, updated_at
                "#,
                name,
                description,
                id,
                user_id
            )
            .fetch_one(&pool)
            .await
            {
                Ok(row) => {
                    if let Some(owner_id) = row.owner_id {
                        let project = Project {
                            id: row.id,
                            name: row.name,
                            description: row.description,
                            owner_id,
                            created_at: row.created_at.to_utc_datetime(),
                            updated_at: row.updated_at.to_utc_datetime(),
                        };
                        Json(project).into_response()
                    } else {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Invalid project data: missing owner_id",
                        )
                            .into_response()
                    }
                }
                Err(e) => {
                    println!("Error updating project: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to update project: {}", e),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            println!("Error fetching project: {:?}", e);
            (
                StatusCode::NOT_FOUND,
                "Project not found or you don't have access",
            )
                .into_response()
        }
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
    {
        Ok(_) => {
            // Return 204 No Content for successful deletion
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => match e {
            sqlx::Error::RowNotFound => (
                StatusCode::NOT_FOUND,
                "Project not found or you don't have access",
            )
                .into_response(),
            _ => {
                println!("Error deleting project: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to delete project: {}", e),
                )
                    .into_response()
            }
        },
    }
}

pub fn routes() -> Router {
    Router::new()
        .route("/", get(fetch_projects))
        .route("/:id", get(api_get_project))
        .route("/", post(api_create_project))
        .route("/:id", put(api_update_project))
        .route("/:id", delete(api_delete_project))
}
