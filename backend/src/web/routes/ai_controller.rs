/*
/ src/web/routes/ai_controller.rs
/ Request Handlers
/
/ File containing various API Backend endpoints for the writing assistant feature
/
/ API Summary:
/ api_get_all_writing_sessions   GET     /                   - Get All Writing Sessions For Current User
/ api_create_writing_session     POST    /                   - Create A New Writing Session
/ api_get_writing_session        GET     /:id                - Get Writing Session By ID With Messages
/ api_send_writing_message       POST    /:id/message        - Send Message And Get AI Response
/ api_delete_writing_session     DELETE  /:id                - Delete Writing Session And All Messages
/ api_get_document_suggestions   GET     /:id/suggestions    - NOT IMPLEMENTED: Get Writing Suggestions For Document
/ api_analyze_document           POST    /analyze            - NOT IMPLEMENTED: Analyze Document For Writing Issues
/ api_get_session_summary        GET     /:id/summary        - NOT IMPLEMENTED: Get Summary Of Writing Session
/
*/

use axum::{
    extract::{Extension, Json, Path},
    routing::{get, post, delete},
    Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower_cookies::Cookies;
use chrono::Utc;

use crate::models::ai::{
    WritingAssistantSession, WritingAssistantMessage, SessionWithMessages, 
    CreateSessionPayload, SendMessagePayload, ChatHistory, ChatMessage
};
// Commented out until implemented
// use crate::cag::retrieval::semantic_search;
use crate::{Error, Result};

use backend::get_user_id_from_cookie;

/// GET handler for retrieving all writing sessions for current user.
/// Accessible via: GET /api/writing-assistant
/// Test: NULL
/// Frontend: NULL
/// Returns a list of all writing assistant sessions belonging to the authenticated user.
/// Sessions are ordered by last updated, with most recent first.
pub async fn api_get_all_writing_sessions(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<WritingAssistantSession>>> {
    println!("->> {:<12} - get_all_writing_sessions", "HANDLER");

    // Get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Get all chat sessions for this user
    let sessions = sqlx::query_as!(
        WritingAssistantSession,
        r#"
        SELECT id, user_id, document_id, title, created_at, updated_at 
        FROM writing_assistant_sessions 
        WHERE user_id = $1
        ORDER BY updated_at DESC
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(sessions))
}

/// POST handler for creating a new writing assistant session.
/// Accessible via: POST /api/writing-assistant
/// Test: NULL
/// Frontend: NULL
/// Creates a new writing assistant session and initializes it with a welcome message.
/// Can optionally be linked to a document by providing a document_id in the payload.
pub async fn api_create_writing_session(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateSessionPayload>,
) -> Result<Json<WritingAssistantSession>> {
    println!("->> {:<12} - create_writing_session", "HANDLER");

    // Get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Create a new chat session
    let session = sqlx::query_as!(
        WritingAssistantSession,
        r#"
        INSERT INTO writing_assistant_sessions (user_id, document_id, title, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, document_id, title, created_at, updated_at
        "#,
        user_id,
        payload.document_id,
        payload.title,
        Utc::now().naive_utc(),
        Utc::now().naive_utc()
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    // Add a system message to initialize the chat
    let system_message = "I'm your writing assistant. How can I help you today?";
    sqlx::query!(
        r#"
        INSERT INTO writing_assistant_messages (session_id, role, content, created_at)
        VALUES ($1, $2, $3, $4)
        "#,
        session.id,
        "assistant",
        system_message,
        Utc::now().naive_utc()
    )
    .execute(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(session))
}

/// GET handler for retrieving specific writing session with messages.
/// Accessible via: GET /api/writing-assistant/:id
/// Test: NULL
/// Frontend: NULL
/// Returns detailed information about a specific writing session including all messages.
/// Only the owner of the session can access it.
pub async fn api_get_writing_session(
    cookies: Cookies,
    Path(session_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<SessionWithMessages>> {
    println!("->> {:<12} - get_writing_session", "HANDLER");

    // Get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Get the chat session
    let session = sqlx::query_as!(
        WritingAssistantSession,
        r#"
        SELECT id, user_id, document_id, title, created_at, updated_at
        FROM writing_assistant_sessions
        WHERE id = $1 AND user_id = $2
        "#,
        session_id,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    // Get all messages for this session
    let messages = sqlx::query_as!(
        WritingAssistantMessage,
        r#"
        SELECT id, session_id, role, content, created_at
        FROM writing_assistant_messages
        WHERE session_id = $1
        ORDER BY created_at ASC
        "#,
        session_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    Ok(Json(SessionWithMessages {
        session,
        messages,
    }))
}

/// POST handler for sending a message and getting AI response.
/// Accessible via: POST /api/writing-assistant/:id/message
/// Test: NULL
/// Frontend: NULL
/// Sends a user message to the AI writing assistant and returns the AI's response.
/// If the session is linked to a document, the document content will be used as context for the AI.
pub async fn api_send_writing_message(
    cookies: Cookies,
    Path(session_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<SendMessagePayload>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - send_writing_message", "HANDLER");

    // Get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Verify the session belongs to this user
    let session = sqlx::query_as!(
        WritingAssistantSession,
        r#"
        SELECT id, user_id, document_id, title, created_at, updated_at
        FROM writing_assistant_sessions
        WHERE id = $1 AND user_id = $2
        "#,
        session_id,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| Error::PermissionError)?;

    // TODO need to embed the users message

    // Record the user's message
    sqlx::query!(
        r#"
        INSERT INTO writing_assistant_messages (session_id, role, content, created_at)
        VALUES ($1, $2, $3, $4)
        "#,
        session_id,
        "user",
        &payload.content,
        Utc::now().naive_utc()
    )
    .execute(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    // Update the session's updated_at timestamp
    sqlx::query!(
        r#"
        UPDATE writing_assistant_sessions
        SET updated_at = $1
        WHERE id = $2
        "#,
        Utc::now().naive_utc(),
        session_id
    )
    .execute(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    // Retrieve all messages for this session to build the context
    let db_messages = sqlx::query_as!(
        WritingAssistantMessage,
        r#"
        SELECT id, session_id, role, content, created_at
        FROM writing_assistant_messages
        WHERE session_id = $1
        ORDER BY created_at ASC
        "#,
        session_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    // Build the chat history for the prompt
    let mut chat_history = ChatHistory::new();
    for msg in &db_messages {
        if msg.role == "user" {
            chat_history.add_user_message(msg.content.clone());
        } else if msg.role == "assistant" {
            chat_history.add_assistant_message(msg.content.clone());
        }
    }

    // Get relevant context from document if linked to a document
    let context = match session.document_id {
        Some(doc_id) => {
            // Fetch the document content
            let doc_content = sqlx::query!(
                r#"
                SELECT name, content
                FROM documents
                WHERE id = $1
                "#,
                doc_id
            )
            .fetch_optional(&pool)
            .await
            .map_err(|_| Error::DatabaseError)?;

            // If we found the document, use it as context
            if let Some(doc) = doc_content {
                // Get the content or default to empty string
                let content = doc.content.unwrap_or_default();
                
                if !content.is_empty() {
                    Some(format!("Document: {}\n{}", doc.name, content))
                } else {
                    None
                }
            } else {
                None
            }
        }
        None => None,
    };

    // TODO Get the LLM response
    

    Ok(Json(json!({})))
}

/// DELETE handler for removing a writing session and all its messages.
/// Accessible via: DELETE /api/writing-assistant/:id
/// Test: NULL
/// Frontend: NULL
/// This will automatically delete all associated messages due to CASCADE delete constraint.
/// Only the owner of the session can delete it.
pub async fn api_delete_writing_session(
    cookies: Cookies,
    Path(session_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - delete_writing_session", "HANDLER");

    // Get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Verify the session belongs to this user and delete it
    // All associated messages will be deleted automatically due to ON DELETE CASCADE
    let result = sqlx::query!(
        r#"
        DELETE FROM writing_assistant_sessions
        WHERE id = $1 AND user_id = $2
        RETURNING id
        "#,
        session_id,
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    if result.is_none() {
        return Err(Error::PermissionError);
    }

    Ok(Json(json!({
        "status": "success",
        "message": "Writing assistant session and all its messages deleted successfully"
    })))
}

/// GET handler for retrieving writing suggestions for a document.
/// Accessible via: GET /api/writing-assistant/:id/suggestions
/// Test: NULL
/// Frontend: NULL
/// NOT IMPLEMENTED: Will provide AI-generated suggestions for improving the document.
pub async fn api_get_document_suggestions(
    _cookies: Cookies,
    _path: Path<i32>,
    _pool: Extension<PgPool>,
) -> Result<Json<Value>> {
    // This is a placeholder for future implementation
    Ok(Json(json!({
        "status": "error",
        "message": "This feature is not implemented yet"
    })))
}

/// POST handler for analyzing a document for writing issues.
/// Accessible via: POST /api/writing-assistant/analyze
/// Test: NULL
/// Frontend: NULL
/// NOT IMPLEMENTED: Will analyze a document for grammar, style, and other writing issues.
pub async fn api_analyze_document(
    _cookies: Cookies,
    _pool: Extension<PgPool>,
    _payload: Json<Value>,
) -> Result<Json<Value>> {
    // This is a placeholder for future implementation
    Ok(Json(json!({
        "status": "error",
        "message": "This feature is not implemented yet"
    })))
}

/// Generate routes for the writing assistant controller
pub fn writing_assistant_routes() -> Router {
    Router::new()
        .route("/", get(api_get_all_writing_sessions))
        .route("/", post(api_create_writing_session))
        .route("/:id", get(api_get_writing_session))
        .route("/:id/message", post(api_send_writing_message))
        .route("/:id", delete(api_delete_writing_session))
        // The following routes are placeholders for future implementation
        // .route("/:id/suggestions", get(api_get_document_suggestions))
        // .route("/analyze", post(api_analyze_document))
        // .route("/:id/summary", get(api_get_session_summary))
}