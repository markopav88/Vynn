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
use std::collections::HashMap;

use crate::models::ai::{
    WritingAssistantSession, WritingAssistantMessage, SessionWithMessages, 
    CreateSessionPayload, SendMessagePayload, MessageRole, SelectedTextContext,
    RewritePayload, WritingAssistantSessionWithSnippet, SessionWithMessageContent,
    ApplySuggestionPayload, SuggestedDocumentChange, LlmDocChange,
    DecisionAgentPayload, DecisionAgentResponse,
    SanitizeTextPayload, SanitizeTextResponse
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
/// Test: test_ai.rs/test_get_all_writing_sessions_success()
/// Frontend: ai.ts/get_all_writing_sessions()
/// Returns a list of all writing assistant sessions belonging to the authenticated user.
/// Sessions are ordered by last updated, with most recent first, and include a snippet of the last message.
pub async fn api_get_all_writing_sessions(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<WritingAssistantSessionWithSnippet>>> { // Update return type
    println!("->> {:<12} - get_all_writing_sessions", "HANDLER");

    // Get user_id from cookies
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Query to get sessions and the content of the last message for each
    let sessions_raw = sqlx::query_as!(
        SessionWithMessageContent,
        r#"
        WITH RankedMessages AS (
            SELECT 
                session_id, 
                content,
                ROW_NUMBER() OVER(PARTITION BY session_id ORDER BY created_at DESC) as rn
            FROM writing_assistant_messages
        )
        SELECT 
            s.id,
            s.user_id,
            s.document_id,
            s.title,
            s.created_at,
            s.updated_at,
            rm.content AS last_message_content
        FROM writing_assistant_sessions s
        LEFT JOIN RankedMessages rm ON s.id = rm.session_id AND rm.rn = 1
        WHERE s.user_id = $1
        ORDER BY s.updated_at DESC
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        eprintln!("Database error fetching sessions with messages: {:?}", e);
        Error::DatabaseError
    })?;

    // Map raw results to the final response struct with truncated snippet
    let sessions_with_snippet = sessions_raw
        .into_iter()
        .map(|raw| {
            let snippet = raw.last_message_content.map(|content| {
                let max_len = 30;
                if content.chars().count() > max_len {
                    format!("{}...", content.chars().take(max_len).collect::<String>())
                } else {
                    content
                }
            });
            WritingAssistantSessionWithSnippet {
                id: raw.id,
                user_id: raw.user_id,
                document_id: raw.document_id,
                title: raw.title,
                created_at: raw.created_at,
                updated_at: raw.updated_at,
                last_message_snippet: snippet,
            }
        })
        .collect();

    Ok(Json(sessions_with_snippet))
}

/// POST handler for creating a new writing assistant session.
/// Accessible via: POST /api/writing-assistant
/// Test: test_ai.rs/test_create_writing_session_success()
/// Frontend: ai.ts/create_writing_session()
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
/// Test: test_ai.rs/test_get_writing_session_success()
/// Frontend: ai.ts/get_writing_session()
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
/// Test: test_ai.rs/test_send_writing_message_success()
/// Frontend: ai.ts/send_writing_message()
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

    // Check and decrement credits before proceeding
    check_and_decrement_ai_credits(&pool, user_id).await?;

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
    
    // Make relevant_chunks mutable
    let mut relevant_chunks = retrieval::semantic_search(
        &pool, 
        project_id_for_context,
        &user_embedding,
        k_value // Use k_value variable
    ).await?;
    
    // --- Fallback Context Retrieval: Full Project Content --- 
    if relevant_chunks.is_empty() && session.document_id.is_some() {
        println!("->> {:<12} - No relevant chunks found. Retrieving full project content as fallback.", "RETRIEVAL");
        
        let current_doc_id = session.document_id.unwrap(); // Safe due to check above
        
        // 1. Find the project_id for the current document
        let project_info = sqlx::query!(
            "SELECT project_id FROM document_projects WHERE document_id = $1",
            current_doc_id
        )
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            eprintln!("Database error fetching project_id for fallback: {:?}", e);
            Error::DatabaseError
        })?;

        if let Some(info) = project_info {
            let project_id = info.project_id;
            println!("->> {:<12} - Found project_id {} for fallback context.", "RETRIEVAL", project_id);

            // 2. Fetch all documents in that project
            // Define a temporary struct for document content
            struct DocumentContent {
                id: i32,
                name: Option<String>,
                content: Option<String>,
            }
            let project_docs = sqlx::query_as!(DocumentContent,
                r#"
                SELECT id, name, content 
                FROM documents 
                WHERE id IN (SELECT document_id FROM document_projects WHERE project_id = $1)
                  AND is_trashed = false
                ORDER BY name ASC -- Or some other consistent order
                "#,
                project_id
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| {
                eprintln!("Database error fetching project documents for fallback: {:?}", e);
                Error::DatabaseError
            })?;

            // Store length before moving the vector
            let project_docs_count = project_docs.len();
            
            // 3. Concatenate content
            let mut full_project_content = String::new();
            for doc in project_docs {
                let doc_name = doc.name.unwrap_or_else(|| "Untitled".to_string());
                let doc_content = doc.content.unwrap_or_default(); // Use empty string if null
                // Add a header for each document
                full_project_content.push_str(&format!("\n--- Document: {} (ID: {}) ---\n", doc_name, doc.id));
                full_project_content.push_str(&doc_content);
                full_project_content.push('\n');
            }

            if !full_project_content.is_empty() {
                 println!("->> {:<12} - Concatenated content from {} documents ({} chars) for fallback.", "RETRIEVAL", project_docs_count, full_project_content.len());
                // 4. Create a single fallback chunk
                let fallback_chunk = retrieval::RetrievedChunk {
                    document_id: -1, // Placeholder ID for full project context
                    document_name: "Full Project Context".to_string(),
                    content: full_project_content,
                };
                // 5. Replace relevant_chunks
                relevant_chunks = vec![fallback_chunk];
            } else {
                 println!("->> {:<12} - Fallback triggered, but project documents have no content.", "RETRIEVAL");
            }
        } else {
             println!("->> {:<12} - Fallback triggered, but could not find project_id for document {}.", "RETRIEVAL", current_doc_id);
        }
    }

    println!("->> {:<12} - Total context chunks to use: {}", "RETRIEVAL", relevant_chunks.len());
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
/// Test: test_ai.rs/test_delete_writing_session_success()
/// Frontend: ai.ts/delete_writing_session()
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
/// Test: test_ai.rs/test_check_grammar_success()
/// Frontend: ai.ts/check_grammar()
pub async fn api_check_grammer(
    cookies: Cookies,
    pool: Extension<PgPool>,
    Json(payload): Json<SelectedTextContext>
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_check_grammer", "HANDLER");

    // Get user_id and check/decrement credits
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;
    check_and_decrement_ai_credits(&pool, user_id).await?;

    let prompt = prompt::construct_grammar_check_prompt(&payload.content);
    
    let query_model = QueryModel::new()?;
    let response = query_model.query_model(&prompt).await?;

    Ok(Json(json!({ "response": response })))
}

/// POST handler for summarizing some text or a document
/// Accessible via: POST /api/writing-assistant/summarize
/// Test: test_ai.rs/test_summarize_success()
/// Frontend: ai.ts/summarize_text()
pub async fn api_summarize(
    cookies: Cookies,
    pool: Extension<PgPool>,
    Json(payload): Json<SelectedTextContext>
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_summarize", "HANDLER");
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;
    check_and_decrement_ai_credits(&pool, user_id).await?;

    let prompt = prompt::construct_summarize_prompt(&payload.content);
    
    let query_model = QueryModel::new()?;
    let response = query_model.query_model(&prompt).await?;

    Ok(Json(json!({ "response": response })))
}

/// POST handler for rephrasing some text or a document
/// Accessible via: POST /api/writing-assistant/rephrase
/// Test: test_ai.rs/test_rephrase_success()
/// Frontend: ai.ts/rephrase_text()
pub async fn api_rephrase(
    cookies: Cookies,
    pool: Extension<PgPool>,
    Json(payload): Json<SelectedTextContext>
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_rephrase", "HANDLER");
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;
    check_and_decrement_ai_credits(&pool, user_id).await?;

    let prompt = prompt::construct_rephrase_prompt(&payload.content);
    
    let query_model = QueryModel::new()?;
    let response = query_model.query_model(&prompt).await?;

    Ok(Json(json!({ "response": response })))
}

/// POST handler for expanding some text or a document
/// Accessible via: POST /api/writing-assistant/expand
/// Test: test_ai.rs/test_expand_success()
/// Frontend: ai.ts/expand_text()
pub async fn api_expand(
    cookies: Cookies,
    pool: Extension<PgPool>,
    Json(payload): Json<SelectedTextContext>
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_expand", "HANDLER");
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;
    check_and_decrement_ai_credits(&pool, user_id).await?;

    let prompt = prompt::construct_expand_prompt(&payload.content);
    
    let query_model = QueryModel::new()?;
    let response = query_model.query_model(&prompt).await?;

    Ok(Json(json!({ "response": response })))
}

/// POST handler for shrinking some text or a document
/// Accessible via: POST /api/writing-assistant/shrink
/// Test: test_ai.rs/test_shrink_success()
/// Frontend: ai.ts/shrink_text()
pub async fn api_shrink(
    cookies: Cookies,
    pool: Extension<PgPool>,
    Json(payload): Json<SelectedTextContext>
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_shrink", "HANDLER");
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;
    check_and_decrement_ai_credits(&pool, user_id).await?;

    let prompt = prompt::construct_shrink_prompt(&payload.content);
    
    let query_model = QueryModel::new()?;
    let response = query_model.query_model(&prompt).await?;

    Ok(Json(json!({ "response": response })))
}

/// POST handler for rewriting some text or a document in a new style
/// Accessible via: POST /api/writing-assistant/rewrite
/// Test: test_ai.rs/test_rewrite_success()
/// Frontend: ai.ts/rewrite_text_as()
pub async fn api_rewrite(
    cookies: Cookies,
    pool: Extension<PgPool>,
    Json(payload): Json<RewritePayload>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_rewrite", "HANDLER");
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;
    check_and_decrement_ai_credits(&pool, user_id).await?;

    let prompt = prompt::construct_rewrite_prompt(&payload.content, &payload.style);
    
    let query_model = QueryModel::new()?;
    let response = query_model.query_model(&prompt).await?;

    Ok(Json(json!({ "response": response })))
}

/// POST handler for fact checking some text or a document
/// Accessible via: POST /api/writing-assistant/factcheck
/// Test: test_ai.rs/test_fact_check_success()
/// Frontend: ai.ts/fact_check_text()
pub async fn api_fact_check(
    cookies: Cookies,
    pool: Extension<PgPool>,
    Json(payload): Json<SelectedTextContext>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_fact_check", "HANDLER");
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;
    check_and_decrement_ai_credits(&pool, user_id).await?;

    let prompt = prompt::construct_fact_check_prompt(&payload.content);
    
    let query_model = QueryModel::new()?;
    let response = query_model.query_model(&prompt).await?;

    Ok(Json(json!({ "response": response })))
}

/// POST handler for spell checking some text or a document
/// Accessible via: POST /api/writing-assistant/spellcheck
/// Test: test_ai.rs/test_spell_check_success()
/// Frontend: ai.ts/check_spelling()
pub async fn api_spell_check(
    cookies: Cookies,
    pool: Extension<PgPool>,
    Json(payload): Json<SelectedTextContext>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_spell_check", "HANDLER");
    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;
    check_and_decrement_ai_credits(&pool, user_id).await?;

    let prompt = prompt::construct_spell_check_prompt(&payload.content);
    
    let query_model = QueryModel::new()?;
    let response = query_model.query_model(&prompt).await?;

    Ok(Json(json!({ "response": response })))
}

/// Helper function to check and decrement AI credits
async fn check_and_decrement_ai_credits(pool: &PgPool, user_id: i32) -> Result<()> {
    // Fetch current credits
    let user_credits = sqlx::query!(
        "SELECT ai_credits FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| Error::DatabaseError)?
    .ok_or(Error::UserNotFoundError { user_id })?
    .ai_credits;

    // Check if credits are sufficient
    if user_credits <= 0 {
        println!("->> {:<12} - User {} has insufficient AI credits ({})", "CREDIT_CHECK", user_id, user_credits);
        return Err(Error::InsufficientAiCredits);
    }

    // Decrement credits
    let update_result = sqlx::query!(
        "UPDATE users SET ai_credits = ai_credits - 1 WHERE id = $1 AND ai_credits > 0",
        user_id
    )
    .execute(pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    // Check if the update actually happened (in case of race condition where credits became 0 between check and update)
    if update_result.rows_affected() == 0 {
        println!("->> {:<12} - Failed to decrement credits for user {} (possible race condition or already 0)", "CREDIT_CHECK", user_id);
        return Err(Error::InsufficientAiCredits); // Treat as insufficient if update failed
    }

    println!("->> {:<12} - Decremented AI credits for user {}. Remaining: {}", "CREDIT_CHECK", user_id, user_credits - 1);
    Ok(())
}

/// POST handler for applying an AI suggestion to project documents.
/// Accessible via: POST /api/ai/writing-assistant/:id/apply-suggestion
/// Test: TODO
pub async fn api_apply_suggestion(
    cookies: Cookies,
    Path(session_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<ApplySuggestionPayload>,
) -> Result<Json<Vec<SuggestedDocumentChange>>> {
    println!("->> {:<12} - api_apply_suggestion for session {}", "HANDLER", session_id);

    let user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Check and decrement credits before proceeding
    check_and_decrement_ai_credits(&pool, user_id).await?;

    // 1. Fetch session to verify ownership and get linked document ID
    let session = sqlx::query_as!(
        WritingAssistantSession,
        "SELECT id, user_id, document_id, title, created_at, updated_at FROM writing_assistant_sessions WHERE id = $1 AND user_id = $2",
        session_id,
        user_id
    )
    .fetch_optional(&pool) // Use optional as session might exist but not belong to user
    .await
    .map_err(|_| Error::DatabaseError)?
    .ok_or(Error::PermissionError)?; // Return permission error if session not found for user

    // 2. Ensure session is linked to a document to find the project
    let current_doc_id = session.document_id.ok_or_else(|| {
        println!("->> {:<12} - Apply suggestion failed: Session {} not linked to a document.", "HANDLER", session_id);
        Error::InvalidRequestFormatError
    })?;

    // 3. Find the project_id for the current document
    let project_info = sqlx::query!(
        "SELECT project_id FROM document_projects WHERE document_id = $1",
        current_doc_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    let project_id = match project_info {
        Some(info) => info.project_id,
        None => {
            println!("->> {:<12} - Apply suggestion failed: Document {} not found in any project.", "HANDLER", current_doc_id);
            return Err(Error::DocumentNotFoundError { document_id: current_doc_id });
        }
    };
    println!("->> {:<12} - Found project_id {} for apply suggestion.", "HANDLER", project_id);

    // 4. Fetch original content of all documents in the project
    struct OriginalDoc { id: i32, name: Option<String>, content: Option<String> }
    let original_docs = sqlx::query_as!(OriginalDoc,
        r#"
        SELECT id, name, content FROM documents 
        WHERE id IN (SELECT document_id FROM document_projects WHERE project_id = $1)
        AND is_trashed = false
        "#,
        project_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| Error::DatabaseError)?;

    if original_docs.is_empty() {
         println!("->> {:<12} - Apply suggestion failed: Project {} has no documents.", "HANDLER", project_id);
         return Err(Error::ProjectNotFoundError { project_id });
    }
    println!("->> {:<12} - Fetched {} original documents for project {}.", "HANDLER", original_docs.len(), project_id);

    // Prepare data for prompt (id, name, content)
    let prompt_docs: Vec<(i32, String, String)> = original_docs
        .iter()
        .map(|doc| (
            doc.id,
            doc.name.clone().unwrap_or_else(|| "Untitled".to_string()),
            doc.content.clone().unwrap_or_default()
        ))
        .collect();

    // Store original content mapped by ID for later diff generation
    let original_content_map: HashMap<i32, String> = original_docs
        .into_iter() // Consume original_docs here
        .map(|doc| (doc.id, doc.content.unwrap_or_default()))
        .collect();


    // 5. Construct the prompt
    let final_prompt = prompt::construct_apply_suggestion_prompt(
        &prompt_docs,
        &payload.suggestion_content,
        payload.current_document_id
    ).map_err(|e| {
        eprintln!("Error serializing documents for prompt: {:?}", e);
        Error::FailedApplyChanges
    })?; // Handle potential serialization error

    // 6. Query LLM
    println!("->> {:<12} - Querying LLM for apply suggestion.", "HANDLER");
    let query_model = QueryModel::new()?;
    let llm_response_str = query_model.query_model(&final_prompt).await?;
    println!("->> {:<12} - LLM response received ({} chars).", "HANDLER", llm_response_str.len());

    // Trim markdown fences if present
    let trimmed_response = llm_response_str
        .strip_prefix("```json\n")
        .unwrap_or(&llm_response_str)
        .strip_suffix("\n```")
        .unwrap_or(&llm_response_str)
        .trim(); // Also trim leading/trailing whitespace just in case

    // 7. Parse LLM response (JSON array of LlmDocChange)
    let llm_changes: Vec<LlmDocChange> = serde_json::from_str(trimmed_response)
        .map_err(|e| {
            eprintln!("Error parsing LLM response JSON: {:?}\nTrimmed Response: {}", e, trimmed_response);
            Error::FailedApplyChanges
        })?;
    println!("->> {:<12} - Parsed {} changes from LLM response.", "HANDLER", llm_changes.len());

    // 8. Construct final response (Vec<SuggestedDocumentChange>)
    let mut suggested_changes: Vec<SuggestedDocumentChange> = Vec::new();
    for change in llm_changes {
        if let Some(old_content) = original_content_map.get(&change.document_id) {
            // Only include if the content actually changed
            if old_content != &change.new_content {
                 suggested_changes.push(SuggestedDocumentChange {
                    document_id: change.document_id,
                    old_content: old_content.clone(), // Clone original content
                    new_content: change.new_content, // Use new content from LLM
                });
            } else {
                 println!("->> {:<12} - LLM proposed no change for doc {}, skipping.", "HANDLER", change.document_id);
            }
        } else {
            // LLM returned an ID not in the original set - log warning but ignore
             println!("->> {:<12} - WARNING: LLM returned change for unknown document ID {}, ignoring.", "HANDLER", change.document_id);
        }
    }
     println!("->> {:<12} - Constructed {} SuggestedDocumentChange entries.", "HANDLER", suggested_changes.len());

    // 9. Return the suggested changes
    Ok(Json(suggested_changes))
}

/// POST handler for deciding if a diff should be proactively shown.
/// Accessible via: POST /api/ai/writing-assistant/decide-proactive-diff
/// This endpoint does NOT decrement AI credits as it's a meta-operation.
pub async fn api_decide_proactive_diff(
    cookies: Cookies,
    Json(payload): Json<DecisionAgentPayload>,
) -> Result<Json<DecisionAgentResponse>> {
    println!("->> {:<12} - api_decide_proactive_diff", "HANDLER");
    // Authenticate user via cookies
    let _user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Construct the prompt for the decision AI
    // Pass the document_content_snippet to the prompt construction function
    let decision_prompt = prompt::construct_proactive_diff_decision_prompt(
        &payload.ai_response_content,
        &payload.context, // This is ProactiveDiffContextPayload
        payload.document_content_snippet.as_deref(), // Pass as Option<&str>
    );

    println!("->> {:<12} - Decision Prompt: ...", "HANDLER"); // Avoid logging potentially large prompt for now

    // Query the LLM for a decision
    let llm = QueryModel::new().map_err(|e| {
        eprintln!("Error creating QueryModel for decision: {:?}", e);
        Error::FailedApplyChanges
    })?;
    let llm_decision_str = llm.query_model(&decision_prompt).await?;
    println!("->> {:<12} - LLM Decision Received: '{}'", "HANDLER", llm_decision_str);

    // Package and return the LLM's raw decision string
    let response = DecisionAgentResponse {
        decision: llm_decision_str.trim().to_string(), // Trim whitespace just in case
    };

    Ok(Json(response))
}

/// POST handler for sanitizing text by removing HTML and Markdown.
/// Accessible via: POST /api/ai/writing-assistant/sanitize-text
/// This endpoint does NOT decrement AI credits as it's a utility operation.
pub async fn api_sanitize_text(
    cookies: Cookies,
    Json(payload): Json<SanitizeTextPayload>,
) -> Result<Json<SanitizeTextResponse>> {
    println!("->> {:<12} - api_sanitize_text", "HANDLER");

    // Authenticate user - even if not billing, good for consistency and future use
    let _user_id = get_user_id_from_cookie(&cookies).ok_or(Error::PermissionError)?;

    // Construct the prompt for the sanitization AI
    let sanitize_prompt = prompt::construct_sanitize_text_prompt(&payload.text_to_sanitize);
    println!("->> {:<12} - Sanitize Prompt: ... (Brief)", "HANDLER"); // Avoid logging large text

    // Query the LLM for sanitization
    let llm = QueryModel::new().map_err(|e| {
        eprintln!("Error creating QueryModel for sanitization: {:?}", e);
        Error::LlmQueryError // Use LlmQueryError for LLM initialization issues too
    })?;
    let sanitized_text_str = llm.query_model(&sanitize_prompt).await.map_err(|e| {
        eprintln!("Error during LLM query for sanitization: {:?}", e);
        Error::LlmQueryError // Use LlmQueryError for query failures
    })?;
    println!("->> {:<12} - LLM Sanitized Text Received ({} chars): ...", "HANDLER", sanitized_text_str.len());

    // Package and return the sanitized text
    let response = SanitizeTextResponse {
        sanitized_text: sanitized_text_str.trim().to_string(), // Trim whitespace
    };

    Ok(Json(response))
}

/// Generate routes for the writing assistant controller
pub fn writing_assistant_routes() -> Router {
    Router::new()
        .route("/", get(api_get_all_writing_sessions))
        .route("/", post(api_create_writing_session))
        .route("/:id", get(api_get_writing_session))
        .route("/:id", delete(api_delete_writing_session))
        .route("/:id/message", post(api_send_writing_message))
        .route("/:id/apply-suggestion", post(api_apply_suggestion))
        .route("/grammer", post(api_check_grammer))
        .route("/spellcheck", post(api_spell_check))
        .route("/summarize", post(api_summarize))
        .route("/rephrase", post(api_rephrase))
        .route("/expand", post(api_expand))
        .route("/shrink", post(api_shrink))
        .route("/rewrite", post(api_rewrite))
        .route("/factcheck", post(api_fact_check))
        .route("/decide-proactive-diff", post(api_decide_proactive_diff))
        .route("/sanitize-text", post(api_sanitize_text))
}
