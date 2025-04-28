// TODO Retrieval from vector db using top k relevant docs

use sqlx::{PgPool, Row};
use pgvector::Vector;
use crate::{Error, Result};
use regex::Regex;
// Import necessary models
use crate::models::ai::{WritingAssistantMessage, ChatHistory, MessageRole};

// Add this struct definition near the top
#[derive(Debug)] // For logging
pub struct RetrievedChunk {
    pub document_id: i32,
    pub document_name: String, // Assuming name is always available
    pub content: String,
}

/// Retrieves the top 'k' most relevant document chunks.
/// Returns a vector of RetrievedChunk containing ID, name, and content.
pub async fn semantic_search(
    pool: &PgPool, 
    project_id: Option<i32>, 
    query_embedding: &Vector, 
    k: i64
) -> Result<Vec<RetrievedChunk>> {
    
    println!("->> {:<12} - Retrieving relevant chunks (k={}) for project_id: {:?}", "RETRIEVAL", k, project_id);
    // Log a snippet of the query embedding
    println!("->> {:<12} - Using query embedding (first 5 dims): {:?}", "RETRIEVAL", query_embedding.as_slice().iter().take(5).collect::<Vec<_>>());

    // Define the base query selecting necessary fields
    let base_select = "SELECT d.id, d.name, d.content "; // <-- Select id and name
    let order_limit = "ORDER BY d.embedding <=> $vector::vector LIMIT $lim";

    let rows = if let Some(p_id) = project_id {
        let query_str = format!(
            "{} \
             FROM documents d \
             JOIN document_projects dp ON d.id = dp.document_id \
             WHERE dp.project_id = $1 \
               AND d.embedding IS NOT NULL \
               AND d.is_trashed = false \
             {}", base_select, order_limit
        );
        // Query within a specific project
        sqlx::query(&query_str.replace("$vector", "$2").replace("$lim", "$3")) // Replace placeholders
            .bind(p_id)
            .bind(query_embedding)
            .bind(k)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                eprintln!("DB Error retrieving project chunks: {:?}", e);
                Error::DatabaseError
            })?
    } else {
        let query_str = format!(
            "{} \
             FROM documents d \
             WHERE d.embedding IS NOT NULL \
               AND d.is_trashed = false \
             {}", base_select, order_limit
        );
        // Query across all documents
         sqlx::query(&query_str.replace("$vector", "$1").replace("$lim", "$2")) // Replace placeholders
            .bind(query_embedding)
            .bind(k)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                 eprintln!("DB Error retrieving general chunks: {:?}", e);
                 Error::DatabaseError
             })?
    };

    println!("->> {:<12} - Rows fetched from DB: {}", "RETRIEVAL_DEBUG", rows.len());

    let html_tag_regex = Regex::new("<[^>]*>")
        .map_err(|e| {
            eprintln!("Regex creation failed: {:?}", e);
            Error::DatabaseError // Map to DatabaseError (adjust if a better variant exists)
        })?;
    

    // Map rows to Vec<RetrievedChunk>
    let chunks: Vec<RetrievedChunk> = rows.iter()
        .filter_map(|row| { // Use filter_map to handle potential errors in getting columns
            let id_res = row.try_get("id"); // Infer type
            let name_res = row.try_get("name"); // Infer type
            let content_res = row.try_get::<String, _>("content"); // Keep String type here

            match (id_res, name_res, content_res) {
                (Ok(id), Ok(name), Ok(html_content)) => {
                    let plain_text = html_tag_regex.replace_all(&html_content, "").to_string();
                    Some(RetrievedChunk {
                        document_id: id,
                        document_name: name,
                        content: plain_text,
                    })
                }
                _ => {
                    eprintln!("->> {:<12} - Failed to get id, name, or content from row", "RETRIEVAL_DEBUG");
                    None // Skip row if any column is missing/wrong type
                }
            }
        })
        .collect();

    println!("->> {:<12} - Retrieved {} relevant chunks (after stripping HTML)", "RETRIEVAL", chunks.len());
    Ok(chunks) // <--- Return Vec<RetrievedChunk>
}

// Updated function to retrieve chat history for a given session_id
pub async fn retrieve_chat_history(
    pool: &PgPool, 
    session_id: i32
) -> Result<ChatHistory> { // Use Result<ChatHistory> instead of Result<ChatHistory, Error>
    println!("->> {:<12} - Retrieving chat history for session {}", "RETRIEVAL", session_id);
    let db_messages = sqlx::query_as!(
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
    .fetch_all(pool) // Use the passed pool reference
    .await
    .map_err(|e| {
        eprintln!("DB Error retrieving chat history: {:?}", e);
        Error::DatabaseError
    })?;

    // Build ChatHistory struct
    let mut chat_history = ChatHistory::new();
    for msg in &db_messages {
        if msg.role == MessageRole::User {
            chat_history.add_user_message(msg.content.clone());
        } else if msg.role == MessageRole::Assistant {
            chat_history.add_assistant_message(msg.content.clone());
        }
    }
    Ok(chat_history) // Return the history
}