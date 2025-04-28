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
    CreateSessionPayload, SendMessagePayload, ChatHistory, MessageRole
};
// Commented out until implemented
// use crate::cag::retrieval::semantic_search;
use crate::{Error, Result};

use backend::get_user_id_from_cookie;

// Import RAG components
use crate::rag::embed::{EmbeddingModel, embed_and_store_user_message};
use crate::rag::llm::QueryModel;
use crate::rag::prompt;
use crate::rag::retrieval;
use pgvector::Vector;

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

    // Add an assistant message to initialize the chat
    let system_message = "I'm your writing assistant. How can I help you today?";
    sqlx::query!(
        r#"
        INSERT INTO writing_assistant_messages (session_id, role, content, created_at)
        VALUES ($1, $2, $3, $4)
        "#,
        session.id,
        MessageRole::Assistant as _,
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
        SELECT 
            id, 
            session_id, 
            role AS "role: MessageRole",
            content, 
            created_at
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

    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

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

    println!("->> {:<12} - Embedding user message", "RAG FUNCTION");
    let embedding_model = EmbeddingModel::new()?;
    let user_embedding: Vector = embed_and_store_user_message(
        &embedding_model,
        &pool,
        session_id,
        &payload.content
    ).await?;

    // Update time on session
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

    // Retrieve chat history using the dedicated function
    println!("->> {:<12} - Retrieving chat history", "RAG FUNCTION");
    let chat_history = retrieval::retrieve_chat_history(&pool, session_id).await?;
    
    // Determine Project ID for context retrieval
    let project_id_for_context: Option<i32> = match session.document_id {
        Some(doc_id) => {
            sqlx::query!("SELECT project_id FROM document_projects WHERE document_id = $1 LIMIT 1", doc_id)
                .fetch_optional(&pool)
                .await
                .map_err(|_| Error::DatabaseError)?
                .map(|info| info.project_id) // Get Option<i32>
        }
        None => None, // No document, so no project context
    };

    // Retrieve relevant document chunks using semantic search
    println!("->> {:<12} - Retrieving relevant context via semantic search", "RAG FUNCTION");
    let relevant_chunks = retrieval::semantic_search(
        &pool, 
        project_id_for_context,
        &user_embedding,
        3
    ).await?;

    // Combine chunks into context string
    let context = if !relevant_chunks.is_empty() {
        Some(relevant_chunks.join("\n\n---\n\n")) // Join with separator
    } else {
        None
    };

    // --- Construct Prompt --- 
    println!("->> {:<12} - Constructing prompt", "RAG FUNCTION");
    let final_prompt = prompt::construct_generic_prompt(
        &payload.content, 
        &chat_history, 
        context // Pass the semantically relevant context
    );

    // --- Query LLM --- 
    println!("->> {:<12} - Querying LLM", "RAG FUNCTION");
    let query_model = QueryModel::new()?;
    let llm_response_content = query_model.query_model(&final_prompt).await?;

    // --- Embed Assistant Response --- 
    println!("->> {:<12} - Embedding assistant response", "RAG FUNCTION");
    let assistant_message_struct = WritingAssistantMessage {
        id: 0, // Placeholder
        session_id,
        role: MessageRole::Assistant,
        content: llm_response_content.clone(),
        created_at: Utc::now().naive_utc(), // Placeholder
    };
    let assistant_embedding = embedding_model.embed_message(&assistant_message_struct).await?;

    // --- Store Assistant Response (with embedding) --- 
    println!("->> {:<12} - Storing assistant message", "RAG FUNCTION");
    sqlx::query!(
        r#"
        INSERT INTO writing_assistant_messages (session_id, role, content, created_at, embedding)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        session_id,
        MessageRole::Assistant as _,
        &llm_response_content,
        Utc::now().naive_utc(),
        assistant_embedding as _ // Add embedding
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        eprintln!("DB Error storing assistant message: {:?}", e);
        Error::DatabaseError
    })?;

    // --- Return Response --- 
    println!("->> {:<12} - Sending response", "RAG FUNCTION");
    Ok(Json(json!({ "role": "assistant", "content": llm_response_content })))
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