use sqlx::PgPool;
use crate::{Error, Result};

/// Helper function to check if a user has permission for a document.
pub async fn check_document_permission(
    pool: &PgPool,
    user_id: i32,
    document_id: i32,
    required_role: &str,
) -> Result<bool> {
    let result = sqlx::query!(
        r#"SELECT role FROM document_permissions 
           WHERE document_id = $1 AND user_id = $2"#,
        document_id,
        user_id
    )
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(record)) => {
            let has_permission = match required_role {
                "viewer" => true, // Any role can view
                "editor" => record.role == "editor" || record.role == "owner",
                "owner" => record.role == "owner",
                _ => false,
            };

            Ok(has_permission)
        }
        Ok(None) => Ok(false),
        Err(e) => {
            println!("Error checking permission: {:?}", e);
            Err(Error::PermissionError)
        }
    }
}

/// Helper function to check if a user is the owner of a project.
/// Returns true if the user is the owner, false otherwise.
pub async fn check_project_ownership(
    pool: &PgPool,
    user_id: i32,
    project_id: i32,
) -> Result<bool> {
    let result = sqlx::query!(
        r#"SELECT user_id FROM projects 
           WHERE id = $1"#,
        project_id
    )
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(record)) => {
            // Check if the user_id matches the project's user_id
            Ok(record.user_id == Some(user_id))
        }
        Ok(None) => {
            // Project not found
            Ok(false)
        }
        Err(e) => {
            println!("Error checking project ownership: {:?}", e);
            Err(Error::PermissionError)
        }
    }
}