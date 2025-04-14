/*
/ src/controllers/proj_controller.rs
/ Request Handlers
/
/ File containing various API Backend endpoints for manipulating a project and its permissions
/
/ API Summary:
/ api_get_all_projects       GET     /                          - Get All Projects For Current User
/ api_get_project            GET     /:id                       - Get Project By ID
/ api_create_project         POST    /                          - Create New Project
/ api_update_project         PUT     /:id                       - Update Project By ID
/ api_delete_project         DELETE  /:id                       - Delete Project By ID
/ api_add_permissions        POST    /:id/permissions           - Add Permissions to User on Project
/ api_get_permissions        GET     /:id/permissions           - Get Users With Permissions to Project
/ api_update_permission      PUT     /:id/permissions           - Update Permissions on User to Project
/ api_remove_permissions     DELETE  /:id/permissions/:user_id  - Delete Permissions on User to Project
/ api_force_delete_project   DELETE  /:id/force                 - Delete Project and All Associated Documents
/ api_add_document           POST    /:id/documents/:doc_id     - Add Document to Project
/ api_get_documents          GET     /:id/documents             - Get All Documents in Project
/ api_remove_document        DELETE  /:id/documents/:doc_id     - Remove Document from Project
/
*/

use axum::routing::{delete, get, post, put};
use axum::{
    extract::{Extension, Json, Path},
    Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower_cookies::Cookies;

use crate::models::project::{CreateProjectPayload, Project, UpdateProjectPayload};
use crate::models::project_permission::{
    CreateProjectPermissionPayload, ProjectPermission, UpdateProjectPermissionPayload,
    UserProjectPermissions,
};
use crate::web::middleware::middleware::check_project_permission;
use crate::{Error, Result};

use crate::models::document::Document;
use backend::get_user_id_from_cookie;

/// GET handler for retrieving all projects for a user.
/// Accessible via: GET /api/project
/// Test: test_projects.rs/test_get_all_projects()
/// Frontend: project.ts/get_all_projects()
async fn api_get_all_projects(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<Project>>> {
    // get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Get all projects where the user has any permission
    let result = sqlx::query_as!(
        Project,
        r#"SELECT p.id, p.name, p.user_id, p.created_at, p.updated_at,is_trashed, is_starred
           FROM projects p
           JOIN project_permissions pp ON p.id = pp.project_id
           WHERE pp.user_id = $1"#,
        user_id
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(projects) => Ok(Json(projects)),
        Err(_) => Err(Error::ProjectNotFoundError),
    }
}

/// GET handler for retrieving a project by ID.
/// Accessible via: GET /api/project/:id
/// Test: test_projects.rs/test_get_project()
/// Frontend: project.ts/get_project()
async fn api_get_project(
    cookies: Cookies,
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Project>> {
    println!("->> {:<12} - api_get_project", "HANDLER");

    // get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Check if user has at least viewer permission
    let has_permission = check_project_permission(&pool, user_id, id, "viewer").await?;

    if !has_permission {
        return Err(Error::PermissionError);
    }

    let result = sqlx::query_as!(
        Project,
        r#"SELECT id, name, user_id, created_at, updated_at,is_trashed, is_starred
           FROM projects 
           WHERE id = $1"#,
        id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(project) => Ok(Json(project)),
        Err(_) => return Err(Error::ProjectNotFoundError),
    }
}

/// POST handler for creating a new project.
/// Accessible via: POST /api/project
/// Test: test_projects.rs/test_create_project()
/// Frontend: project.ts/create_project()
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
        RETURNING id, name, user_id, created_at, updated_at,is_trashed, is_starred
        "#,
        payload._name,
        user_id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(project) => {
            // Add owner permission for the creator
            let permissions = sqlx::query!(
                "INSERT INTO project_permissions (project_id, user_id, role)
                VALUES ($1, $2, 'owner')",
                project.id,
                user_id
            )
            .execute(&pool)
            .await;

            if let Err(_) = permissions {
                return Err(Error::PermissionCreationError);
            }

            Ok(Json(project))
        }
        Err(_) => Err(Error::ProjectNotFoundError),
    }
}

/// PUT handler for updating a project.
/// Accessible via: PUT /api/project/:id
/// Test: test_projects.rs/test_update_project()
/// Frontend: project.ts/update_project()
async fn api_update_project(
    cookies: Cookies,
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<UpdateProjectPayload>,
) -> Result<Json<Project>> {
    println!("->> {:<12} - api_update_project", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Check if user has editor or owner permission
    let has_permission = check_project_permission(&pool, user_id, id, "editor").await?;

    if !has_permission {
        return Err(Error::PermissionError);
    }

    // Update the project
    let result = sqlx::query_as!(
        Project,
        r#"
        UPDATE projects 
        SET name = $1, updated_at = CURRENT_TIMESTAMP
        WHERE id = $2
        RETURNING id, name, user_id, created_at, updated_at,is_trashed, is_starred
        "#,
        payload._name,
        id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(project) => Ok(Json(project)),
        Err(_) => return Err(Error::ProjectNotFoundError),
    }
}

/// DELETE handler for deleting a project.
/// Accessible via: DELETE /api/project/:id
/// Test: test_projects.rs/test_delete_project()
/// Frontend: project.ts/delete_project()
async fn api_delete_project(
    cookies: Cookies,
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Project>> {
    println!("->> {:<12} - api_delete_project", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Check if user has owner permission
    let has_permission = check_project_permission(&pool, user_id, id, "owner").await?;

    if !has_permission {
        return Err(Error::PermissionError);
    }

    // First delete all permissions
    let _ = sqlx::query!(
        "DELETE FROM project_permissions
        WHERE project_id = $1",
        id
    )
    .execute(&pool)
    .await;

    // Then delete the project
    let result = sqlx::query_as!(
        Project,
        r#"DELETE FROM projects 
        WHERE id = $1
        RETURNING id, name, user_id, created_at, updated_at, is_trashed, is_starred
        "#,
        id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(project) => Ok(Json(project)),
        Err(_) => Err(Error::ProjectNotFoundError),
    }
}

/// POST handler for granting permission to a user for a project.
/// Accessible via: POST /api/project/:id/permissions
/// Test: test_projects.rs/test_add_permissions()
/// Frontend: project.ts/add_project_permissions()
async fn api_add_permissions(
    cookies: Cookies,
    Path(project_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateProjectPermissionPayload>,
) -> Result<Json<ProjectPermission>> {
    println!("->> {:<12} - grant_project_permission", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Check if user has owner permission
    let has_permission = check_project_permission(&pool, user_id, project_id, "owner").await?;

    if !has_permission {
        return Err(Error::PermissionError);
    }

    // Insert the project permission
    let result = sqlx::query_as!(
        ProjectPermission,
        "INSERT INTO project_permissions (project_id, user_id, role)
        VALUES ($1, $2, $3)
        ON CONFLICT (project_id, user_id) 
        DO UPDATE SET role = $3
        RETURNING project_id, user_id, role, created_at",
        project_id,
        payload.user_id,
        payload.role
            )
            .fetch_one(&pool)
            .await;

            match result {
        Ok(permission) => {
            // Get all documents in the project
            let documents = sqlx::query!(
                "SELECT document_id FROM document_projects WHERE project_id = $1",
                project_id
            )
            .fetch_all(&pool)
            .await
            .map_err(|_| Error::DatabaseError)?;

            // For each document, add document permissions
            for doc in documents {
                // Check if permission already exists
                let existing = sqlx::query!(
                    "SELECT 1 as exists FROM document_permissions 
                     WHERE document_id = $1 AND user_id = $2",
                    doc.document_id,
                    payload.user_id
                )
                .fetch_optional(&pool)
                .await
                .map_err(|_| Error::DatabaseError)?;

                if existing.is_none() {
                    // Add document permission with the same role as project permission
                    sqlx::query!(
                        "INSERT INTO document_permissions (document_id, user_id, role)
                         VALUES ($1, $2, $3)",
                        doc.document_id,
                        payload.user_id,
                        payload.role
                    )
                    .execute(&pool)
                    .await
                    .map_err(|_| Error::DatabaseError)?;
                }
            }

            Ok(Json(permission))
        }
        Err(_) => Err(Error::PermissionError)
    }
}

/// GET handler for retrieving all users with access to a project.
/// Accessible via: GET /api/project/:id/permissions
/// Test: test_projects.rs/test_get_permissions()
/// Frontend: project.ts/get_project_permissions()
async fn api_get_permissions(
    cookies: Cookies,
    Path(project_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<UserProjectPermissions>>> {
    println!("->> {:<12} - get_project_users", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Check if user has at least viewer permission
    let has_permission = check_project_permission(&pool, user_id, project_id, "viewer").await?;

    if !has_permission {
        return Err(Error::PermissionError);
    }

    let result = sqlx::query_as!(
        UserProjectPermissions,
        r#"SELECT pp.user_id, u.name, u.email, pp.role 
           FROM project_permissions pp
           JOIN users u ON pp.user_id = u.id
           WHERE pp.project_id = $1"#,
        project_id
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(users) => Ok(Json(users)),
        Err(_) => Err(Error::ProjectNotFoundError),
    }
}

/// PUT handler for updating a user's permission for a project.
/// Accessible via: PUT /api/project/:id/permissions
/// Test: test_projects.rs/test_update_permission()
/// Frontend: project.ts/update_project_permission()
async fn api_update_permission(
    cookies: Cookies,
    Path(project_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<UpdateProjectPermissionPayload>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - update_project_permission", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Check if user has owner permission
    let has_permission = check_project_permission(&pool, user_id, project_id, "owner").await?;

    if !has_permission {
        return Err(Error::PermissionError);
    }

    // Check if this is an ownership transfer
    if payload.role == "owner" {
        // Get the current owner's role
        let current_owner = sqlx::query!(
            "SELECT user_id, role FROM project_permissions 
             WHERE project_id = $1 AND role = 'owner'",
            project_id
        )
        .fetch_one(&pool)
        .await
        .map_err(|_| Error::DatabaseError)?;

        // If the current owner is different from the user being made owner
        if current_owner.user_id != payload.user_id {
            // Update the current owner to editor
            sqlx::query!(
                "UPDATE project_permissions 
                 SET role = 'editor'
                 WHERE project_id = $1 AND user_id = $2",
                project_id,
                current_owner.user_id
            )
            .execute(&pool)
            .await
            .map_err(|_| Error::DatabaseError)?;

            // Update all document permissions for the current owner
            let documents = sqlx::query!(
                "SELECT document_id FROM document_projects WHERE project_id = $1",
                project_id
            )
            .fetch_all(&pool)
            .await
            .map_err(|_| Error::DatabaseError)?;

            for doc in documents {
                sqlx::query!(
                    "UPDATE document_permissions 
                     SET role = 'editor'
                     WHERE document_id = $1 AND user_id = $2",
                    doc.document_id,
                    current_owner.user_id
                )
                .execute(&pool)
                .await
                .map_err(|_| Error::DatabaseError)?;
            }
        }
    }

    // Update the project permission
    let result = sqlx::query!(
        "UPDATE project_permissions 
         SET role = $1
         WHERE project_id = $2 AND user_id = $3",
        payload.role,
        project_id,
        payload.user_id
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            // Get all documents in the project
            let documents = sqlx::query!(
                "SELECT document_id FROM document_projects WHERE project_id = $1",
                project_id
            )
            .fetch_all(&pool)
            .await
            .map_err(|_| Error::DatabaseError)?;

            // Update permissions for all documents
            for doc in documents {
                // Check if permission exists
                let existing = sqlx::query!(
                    "SELECT 1 as exists FROM document_permissions 
                     WHERE document_id = $1 AND user_id = $2",
                    doc.document_id,
                    payload.user_id
                )
                .fetch_optional(&pool)
                .await
                .map_err(|_| Error::DatabaseError)?;

                if existing.is_some() {
                    // Update existing document permission
                    sqlx::query!(
                        "UPDATE document_permissions 
                         SET role = $1
                         WHERE document_id = $2 AND user_id = $3",
                        payload.role,
                        doc.document_id,
                        payload.user_id
                    )
                    .execute(&pool)
                    .await
                    .map_err(|_| Error::DatabaseError)?;
                }
            }

            Ok(Json(json!({
                "result": {
                    "success": true,
                }
            })))
        }
        Err(e) => {
            println!("Error updating permission: {:?}", e);
            Err(Error::PermissionError)
        }
    }
}

/// DELETE handler for removing a user's permission for a project.
/// Accessible via: DELETE /api/project/:id/permissions/:user_id
/// Test: test_projects.rs/test_remove_permissions()
/// Frontend: project.ts/remove_project_permissions()
async fn api_delete_permissions(
    cookies: Cookies,
    Path((project_id, target_id)): Path<(i32, i32)>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - remove_project_permission", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;
    
    // Check if user has owner permission
    let has_permission = check_project_permission(&pool, user_id, project_id, "owner").await?;

    if !has_permission {
        return Err(Error::PermissionError);
    }

    // Prevent removing the last owner
    let owners_count_result = sqlx::query!(
        "SELECT COUNT(*) as count FROM project_permissions 
         WHERE project_id = $1 AND role = 'owner'",
        project_id
    )
    .fetch_one(&pool)
    .await;

    let is_target_owner = sqlx::query!(
        "SELECT role FROM project_permissions 
         WHERE project_id = $1 AND user_id = $2",
        project_id,
        target_id
    )
    .fetch_optional(&pool)
    .await;

    // If we're removing an owner and there's only one owner, prevent it
    if let (Ok(owners_count), Ok(Some(record))) = (&owners_count_result, &is_target_owner) {
        if record.role == "owner" && owners_count.count.unwrap_or(0) <= 1 {
            return Err(Error::PermissionError);
        }
    }

    // Get all documents in the project
    let documents = sqlx::query!(
        "SELECT document_id FROM document_projects WHERE project_id = $1",
        project_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    // Remove permissions from all documents
    for doc in documents {
        sqlx::query!(
            "DELETE FROM document_permissions 
             WHERE document_id = $1 AND user_id = $2",
            doc.document_id,
            target_id
        )
        .execute(&pool)
        .await
        .map_err(|_| Error::DatabaseError)?;
    }

    // Remove the project permission
    let result = sqlx::query!(
        "DELETE FROM project_permissions 
         WHERE project_id = $1 AND user_id = $2",
        project_id,
        target_id
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Ok(Json(json!({
            "result": {
                "success": true,
                "message": "Permission removed successfully"
            }
        }))),
        Err(_) => Err(Error::PermissionError),
    }
}

/// DELETE handler for deleting a project and all its documents.
/// Accessible via: DELETE /api/project/:id/force
/// Test: test_projects.rs/test_force_delete_project()
/// Frontend: project.ts/force_delete_project()
async fn api_force_delete_project(
    cookies: Cookies,
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_force_delete_project", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Check if user has owner permission
    let has_permission = check_project_permission(&pool, user_id, id, "owner").await?;

    if !has_permission {
        return Err(Error::PermissionError);
    }

    // 1. Get all document IDs in this project
    let document_ids = sqlx::query!(
        "SELECT document_id FROM document_projects WHERE project_id = $1",
        id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| Error::ProjectNotFoundError)?;

    // 2. For each document, delete permissions and then the document
    for doc_record in document_ids {
        let doc_id = doc_record.document_id;

        // Delete document permissions
        sqlx::query!(
            "DELETE FROM document_permissions WHERE document_id = $1",
            doc_id
        )
        .execute(&pool)
        .await
        .map_err(|_| Error::DocumentDeletionError)?;

        // Delete document
        sqlx::query!("DELETE FROM documents WHERE id = $1", doc_id)
            .execute(&pool)
            .await
            .map_err(|_| Error::DocumentDeletionError)?;
    }

    // 3. Delete project permissions
    sqlx::query!("DELETE FROM project_permissions WHERE project_id = $1", id)
        .execute(&pool)
        .await
        .map_err(|_| Error::PermissionError)?;

    // 4. Delete the project
    sqlx::query!("DELETE FROM projects WHERE id = $1", id)
        .execute(&pool)
        .await
        .map_err(|_| Error::ProjectNotFoundError)?;

    Ok(Json(json!({
        "result": {
            "success": true,
            "message": "Project and all associated documents deleted successfully"
        }
    })))
}

/// GET handler for retrieving all documents in a project.
/// Accessible via: GET /api/project/:id/documents
/// Test: test_projects.rs/test_get_project_documents()
/// Frontend: project.ts/get_project_documents()
async fn api_get_documents(
    cookies: Cookies,
    Path(project_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<Document>>> {
    println!("->> {:<12} - api_get_documents", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Get all documents in the project that the user has at least viewer access to
    let documents = sqlx::query_as!(
        Document,
        r#"SELECT DISTINCT d.id, d.name, d.content, d.created_at, d.updated_at, d.user_id, d.is_trashed, d.is_starred
        FROM documents d
        JOIN document_projects dp ON d.id = dp.document_id
        LEFT JOIN document_permissions dp2 ON d.id = dp2.document_id
        WHERE dp.project_id = $1
        AND (
            d.user_id = $2  -- User is the owner
            OR dp2.user_id = $2  -- User has direct document permissions
            OR EXISTS (  -- User has project permissions
                SELECT 1 FROM project_permissions pp 
                WHERE pp.project_id = $1 
                AND pp.user_id = $2 
                AND pp.role IN ('owner', 'editor', 'viewer')
            )
        )
        ORDER BY d.id"#,
        project_id,
        user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| Error::ProjectNotFoundError)?;

    Ok(Json(documents))
}

/// POST handler for adding a document to a project.
/// Accessible via: POST /api/project/:id/documents/:doc_id
/// Test: test_projects.rs/test_add_document_to_project()
/// Frontend: project.ts/add_document_to_project()
async fn api_add_document(
    cookies: Cookies,
    Path((project_id, document_id)): Path<(i32, i32)>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_add_document", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Check if user has at least editor permission on the project
    let has_project_permission =
        check_project_permission(&pool, user_id, project_id, "editor").await?;

    if !has_project_permission {
        return Err(Error::PermissionError);
    }

    // Check if user has at least editor permission on the document
    let has_document_permission = sqlx::query!(
        "SELECT role FROM document_permissions 
         WHERE document_id = $1 AND user_id = $2 AND role IN ('editor', 'owner')",
        document_id,
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| Error::PermissionError)?;

    if has_document_permission.is_none() {
        return Err(Error::PermissionError);
    }

    // Check if the document is already in the project
    let existing = sqlx::query!(
        "SELECT 1 as exists_flag FROM document_projects 
         WHERE document_id = $1 AND project_id = $2",
        document_id,
        project_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    if existing.is_some() {
        return Ok(Json(json!({
            "result": {
                "success": true,
                "message": "Document already in project"
            }
        })));
    }

    // Add the document to the project
    let result = sqlx::query!(
        "INSERT INTO document_projects (document_id, project_id) 
         VALUES ($1, $2)",
        document_id,
        project_id
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Ok(Json(json!({
            "result": {
                "success": true,
                "message": "Document added to project successfully"
            }
        }))),
        Err(_) => Err(Error::DatabaseError),
    }
}

/// DELETE handler for removing a document from a project.
/// Accessible via: DELETE /api/project/:id/documents/:doc_id
/// Test: test_projects.rs/test_remove_document_from_project()
/// Frontend: project.ts/remove_document_from_project()
async fn api_remove_document(
    cookies: Cookies,
    Path((project_id, document_id)): Path<(i32, i32)>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_remove_document", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Check if user has at least editor permission
    let has_permission = check_project_permission(&pool, user_id, project_id, "editor").await?;

    if !has_permission {
        return Err(Error::PermissionError);
    }

    // Remove the document from the project
    let result = sqlx::query!(
        "DELETE FROM document_projects 
         WHERE document_id = $1 AND project_id = $2",
        document_id,
        project_id
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Ok(Json(json!({
            "result": {
                "success": true,
                "message": "Document removed from project successfully"
            }
        }))),
        Err(_) => Err(Error::DatabaseError),
    }
}

/// PUT handler for starring a project.
/// Accessible via: PUT /api/project/:id/star
async fn api_toggle_star_project(
    cookies: Cookies,
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_toggle_star_project", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Check if user has at least editor permission
    let has_permission = check_project_permission(&pool, user_id, id, "editor").await?;

    if !has_permission {
        return Err(Error::PermissionError);
    }

    // Get current star status
    let project = sqlx::query!(
        r#"
        SELECT is_starred
        FROM projects 
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| Error::ProjectNotFoundError)?;

    // Toggle the star status
    let new_status = !project.is_starred.unwrap_or(false);

    // Update the project
    let _ = sqlx::query!(
        r#"
        UPDATE projects 
        SET is_starred = $1
        WHERE id = $2
        "#,
        new_status,
        id
    )
    .execute(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(json!({
                "result": {
                        "success": true,
            "message": "Project star status updated",
            "is_starred": new_status
                    }
            })))
}

/// PUT handler for moving a project to trash.
/// Accessible via: PUT /api/project/:id/trash
async fn api_trash_project(
    cookies: Cookies,
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_trash_project", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Check if user has owner permission
    let has_permission = check_project_permission(&pool, user_id, id, "owner").await?;

    if !has_permission {
        return Err(Error::PermissionError);
    }

    // Update the project
    let _ = sqlx::query!(
        r#"
        UPDATE projects 
        SET is_trashed = true
        WHERE id = $1
        "#,
        id
    )
    .execute(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(json!({
        "result": {
            "success": true,
            "message": "Project moved to trash"
        }
    })))
}

/// PUT handler for restoring a project from trash.
/// Accessible via: PUT /api/project/:id/restore
async fn api_restore_project(
    cookies: Cookies,
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_restore_project", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Check if user has owner permission
    let has_permission = check_project_permission(&pool, user_id, id, "owner").await?;

    if !has_permission {
        return Err(Error::PermissionError);
    }

    // Update the project
    let _ = sqlx::query!(
        r#"
        UPDATE projects 
        SET is_trashed = false
        WHERE id = $1
        "#,
        id
    )
    .execute(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(json!({
        "result": {
            "success": true,
            "message": "Project restored from trash"
        }
    })))
}

/// GET handler for retrieving all starred projects for a user.
/// Accessible via: GET /api/project/starred
async fn api_get_starred_projects(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<Project>>> {
    println!("->> {:<12} - api_get_starred_projects", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Get all starred projects for this user
    let result = sqlx::query_as!(
        Project,
        r#"SELECT p.id, p.name, p.user_id, p.created_at, p.updated_at, p.is_starred, p.is_trashed
           FROM projects p
           JOIN project_permissions pp ON p.id = pp.project_id
           WHERE pp.user_id = $1 AND p.is_starred = true AND p.is_trashed = false"#,
        user_id
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(projects) => Ok(Json(projects)),
        Err(_) => Err(Error::ProjectNotFoundError),
    }
}

/// GET handler for retrieving all trashed projects for a user.
/// Accessible via: GET /api/project/trash
async fn api_get_trashed_projects(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<Project>>> {
    println!("->> {:<12} - api_get_trashed_projects", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Get all trashed projects for this user
    let result = sqlx::query_as!(
        Project,
        r#"SELECT p.id, p.name, p.user_id, p.created_at, p.updated_at, p.is_starred, p.is_trashed
           FROM projects p
           JOIN project_permissions pp ON p.id = pp.project_id
           WHERE pp.user_id = $1 AND p.is_trashed = true"#,
        user_id
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(projects) => Ok(Json(projects)),
        Err(_) => Err(Error::ProjectNotFoundError),
    }
}

/// GET handler for retrieving all shared projects for a user (where user is not owner but has viewer/editor permissions).
/// Accessible via: GET /api/project/shared
async fn api_get_shared_projects(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<Project>>> {
    println!("->> {:<12} - api_get_shared_projects", "HANDLER");

    // Get user ID from cookie
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Get all projects where the user has editor/viewer permissions but is not the owner
    let result = sqlx::query_as!(
        Project,
        r#"SELECT p.id, p.name, p.user_id, p.created_at, p.updated_at, is_trashed, is_starred
           FROM projects p
           JOIN project_permissions pp ON p.id = pp.project_id
           WHERE pp.user_id = $1 
           AND pp.role IN ('editor', 'viewer')
           AND COALESCE(p.is_trashed, false) = false"#,
        user_id
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(projects) => Ok(Json(projects)),
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
        .route("/:id/force", delete(api_force_delete_project))
        .route("/:id/permissions", post(api_add_permissions))
        .route("/:id/permissions", get(api_get_permissions))
        .route("/:id/permissions", put(api_update_permission))
        .route("/:id/permissions/:user_id", delete(api_delete_permissions))
        .route("/:id/documents", get(api_get_documents))
        .route("/:id/documents/:doc_id", post(api_add_document))
        .route("/:id/documents/:doc_id", delete(api_remove_document))
        .route("/:id/star", put(api_toggle_star_project))
        .route("/:id/trash", put(api_trash_project))
        .route("/:id/restore", put(api_restore_project))
        .route("/starred", get(api_get_starred_projects))
        .route("/trash", get(api_get_trashed_projects))
        .route("/shared", get(api_get_shared_projects))
}
