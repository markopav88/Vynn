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
    CreateSessionPayload, SendMessagePayload, MessageRole, SelectedTextContext,
    RewritePayload
};
// Commented out until implemented
// use crate::cag::retrieval::semantic_search;
use crate::{Error, Result};

use backend::get_user_id_from_cookie;

// Import RAG components
use crate::rag::embed::{EmbeddingModel, embed_and_store_user_message, embed_and_store_assistant_message};
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
    println!("->> {:<12} - Payload: {:?}", "HANDLER", payload);

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
    // Create the message struct to pass
    let user_message_to_store = WritingAssistantMessage {
        id: 0, // Placeholder
        session_id,
        role: MessageRole::User,
        content: payload.content.clone(), // Clone content from payload
        created_at: Utc::now().naive_utc(), // Placeholder, actual time set during INSERT
    };
    println!("->> {:<12} - User message content: \"{}\"", "RAG FUNCTION", payload.content);
    let user_embedding: Vector = embed_and_store_user_message(
        &embedding_model,
        &pool,
        session_id,
        &user_message_to_store
    ).await?;
    // Log a snippet of the embedding for verification
    println!("->> {:<12} - User embedding calculated (first 5 dims): {:?}", "RAG FUNCTION", user_embedding.as_slice().iter().take(5).collect::<Vec<_>>());

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
    println!("->> {:<12} - Retrieved {} messages from history", "RETRIEVAL", chat_history.messages.len());
    
    // Determine Project ID and Current Document Name for context retrieval
    let mut project_id_for_context: Option<i32> = None;
    let mut current_doc_name: Option<String> = None;
    if let Some(doc_id) = session.document_id {
        // Fetch project ID and document name if document is linked
        let doc_info = sqlx::query!(
            r#"
            SELECT dp.project_id, d.name 
            FROM documents d 
            LEFT JOIN document_projects dp ON d.id = dp.document_id 
            WHERE d.id = $1
            "#,
            doc_id
        )
        .fetch_optional(&pool)
        .await
        .map_err(|_| Error::DatabaseError)?;

        if let Some(info) = doc_info {
            project_id_for_context = Some(info.project_id); // Wrap in Some() to match Option type
            current_doc_name = Some(info.name); // Store the name
        }
    }

    // Retrieve relevant document chunks using semantic search
    println!("->> {:<12} - Retrieving relevant context via semantic search", "RAG FUNCTION");
    let k_value = 3;
    println!("->> {:<12} - Retrieving relevant chunks (k={}) for project_id: {:?}", "RETRIEVAL", k_value, project_id_for_context);
    let relevant_chunks = retrieval::semantic_search(
        &pool, 
        project_id_for_context,
        &user_embedding,
        k_value // Use k_value variable
    ).await?;
    println!("->> {:<12} - Retrieved {} relevant chunks", "RETRIEVAL", relevant_chunks.len());
    // Log retrieved chunks (or snippets)
    for (i, chunk) in relevant_chunks.iter().enumerate() {
        println!("->> {:<12} - Chunk {} (Doc ID: {}, Name: {}): \"{}...\"", 
                 "RETRIEVAL", 
                 i + 1, 
                 chunk.document_id, 
                 chunk.document_name, 
                 chunk.content.chars().take(70).collect::<String>());
    }

    // --- Construct Prompt --- 
    println!("->> {:<12} - Constructing prompt", "RAG FUNCTION");
    let final_prompt = prompt::construct_generic_prompt(
        &payload.content, 
        &chat_history, 
        &relevant_chunks, // Pass the Vec<RetrievedChunk>
        session.document_id, // Pass current doc ID
        current_doc_name.as_deref() // Pass current doc name as &str
    );
    // Log prompt snippet and estimated tokens (simple space split estimate)
    let estimated_tokens = final_prompt.split_whitespace().count();
    println!("->> {:<12} - Prompt constructed ({} tokens estimated):\n---\n{}\n---", "PROMPT", estimated_tokens, final_prompt);

    // --- Query LLM --- 
    println!("->> {:<12} - Querying LLM", "RAG FUNCTION");
    let query_model = QueryModel::new()?;
    let llm_response_content = query_model.query_model(&final_prompt).await?;
    println!("->> {:<12} - LLM response received: \"{}...\"", "RAG FUNCTION", llm_response_content.chars().take(70).collect::<String>());

    // --- Embed and Store Assistant Response --- 
    println!("->> {:<12} - Assistant response content: \"{}\"", "RAG FUNCTION", llm_response_content);
    embed_and_store_assistant_message(
        &embedding_model,
        &pool,
        session_id,
        &llm_response_content // Pass LLM response content
    ).await?;

    // --- Return Response --- 
    println!("->> {:<12} - Sending response", "RAG FUNCTION");
    let response_json = json!({ "role": "assistant", "content": llm_response_content });
    println!("->> {:<12} - Response JSON: {:?}", "RES_MAPPER", response_json);
    Ok(Json(response_json))
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

/// POST handler for suggesting grammer changes for the document or selected text
/// Accessible via: POST /api/writing-assistant/:id/grammer
/// Test: NULL
/// Frontend: NULL
pub async fn api_check_grammer(
    cookies: Cookies,
    pool: Extension<PgPool>,
    Json(payload): Json<SelectedTextContext>
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_check_grammer", "HANDLER");

    // Get user_id to ensure permission (optional, could be removed if direct text analysis is allowed)
    get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    let prompt = prompt::construct_grammar_check_prompt(&payload.content);
    
    let query_model = QueryModel::new()?;
    let response = query_model.query_model(&prompt).await?;

    Ok(Json(json!({ "response": response })))
}

/// POST handler for summarizing some text or a document
/// Accessible via: POST /api/writing-assistant/summarize
/// Test: NULL
/// Frontend: NULL
pub async fn api_summarize(
    cookies: Cookies,
    pool: Extension<PgPool>,
    Json(payload): Json<SelectedTextContext>
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_summarize", "HANDLER");
    get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    let prompt = prompt::construct_summarize_prompt(&payload.content);
    
    let query_model = QueryModel::new()?;
    let response = query_model.query_model(&prompt).await?;

    Ok(Json(json!({ "response": response })))
}

/// POST handler for rephrasing some text or a document
/// Accessible via: POST /api/writing-assistant/rephrase
/// Test: NULL
/// Frontend: NULL
pub async fn api_rephrase(
    cookies: Cookies,
    pool: Extension<PgPool>,
    Json(payload): Json<SelectedTextContext>
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_rephrase", "HANDLER");
    get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    let prompt = prompt::construct_rephrase_prompt(&payload.content);
    
    let query_model = QueryModel::new()?;
    let response = query_model.query_model(&prompt).await?;

    Ok(Json(json!({ "response": response })))
}

/// POST handler for expanding some text or a document
/// Accessible via: POST /api/writing-assistant/expand
/// Test: NULL
/// Frontend: NULL
pub async fn api_expand(
    cookies: Cookies,
    pool: Extension<PgPool>,
    Json(payload): Json<SelectedTextContext>
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_expand", "HANDLER");
    get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    let prompt = prompt::construct_expand_prompt(&payload.content);
    
    let query_model = QueryModel::new()?;
    let response = query_model.query_model(&prompt).await?;

    Ok(Json(json!({ "response": response })))
}

/// POST handler for shrinking some text or a document
/// Accessible via: POST /api/writing-assistant/shrink
/// Test: NULL
/// Frontend: NULL
pub async fn api_shrink(
    cookies: Cookies,
    pool: Extension<PgPool>,
    Json(payload): Json<SelectedTextContext>
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_shrink", "HANDLER");
    get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    let prompt = prompt::construct_shrink_prompt(&payload.content);
    
    let query_model = QueryModel::new()?;
    let response = query_model.query_model(&prompt).await?;

    Ok(Json(json!({ "response": response })))
}

/// POST handler for rewriting some text or a document in a new style
/// Accessible via: POST /api/writing-assistant/rewrite
/// Test: NULL
/// Frontend: NULL
pub async fn api_rewrite(
    cookies: Cookies,
    pool: Extension<PgPool>,
    Json(payload): Json<RewritePayload>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_rewrite", "HANDLER");
    get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    let prompt = prompt::construct_rewrite_prompt(&payload.content, &payload.style);
    
    let query_model = QueryModel::new()?;
    let response = query_model.query_model(&prompt).await?;

    Ok(Json(json!({ "response": response })))
}

/// POST handler for fact checking some text or a document
/// Accessible via: POST /api/writing-assistant/factcheck
/// Test: NULL
/// Frontend: NULL
pub async fn api_fact_check(
    cookies: Cookies,
    pool: Extension<PgPool>,
    Json(payload): Json<SelectedTextContext>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_fact_check", "HANDLER");
    get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    let prompt = prompt::construct_fact_check_prompt(&payload.content);
    
    let query_model = QueryModel::new()?;
    let response = query_model.query_model(&prompt).await?;

    Ok(Json(json!({ "response": response })))
}

/// POST handler for spell checking some text or a document
/// Accessible via: POST /api/writing-assistant/spellcheck
/// Test: NULL
/// Frontend: NULL
pub async fn api_spell_check(
    cookies: Cookies,
    pool: Extension<PgPool>,
    Json(payload): Json<SelectedTextContext>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_spell_check", "HANDLER");
    get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    let prompt = prompt::construct_spell_check_prompt(&payload.content);
    
    let query_model = QueryModel::new()?;
    let response = query_model.query_model(&prompt).await?;

    Ok(Json(json!({ "response": response })))
}

/// Generate routes for the writing assistant controller
pub fn writing_assistant_routes() -> Router {
    Router::new()
        .route("/", get(api_get_all_writing_sessions))
        .route("/", post(api_create_writing_session))
        .route("/:id", get(api_get_writing_session))
        .route("/:id", delete(api_delete_writing_session))
        .route("/:id/message", post(api_send_writing_message))
        .route("/grammer", post(api_check_grammer))
        .route("/spellcheck", post(api_spell_check))
        .route("/summarize", post(api_summarize))
        .route("/rephrase", post(api_rephrase))
        .route("/expand", post(api_expand))
        .route("/shrink", post(api_shrink))
        .route("/rewrite", post(api_rewrite))
        .route("/factcheck", post(api_fact_check))
}
