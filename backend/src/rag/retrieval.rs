// TODO Retrieval from vector db using top k relevant docs

use sqlx::{PgPool, Row, Error as SqlxError};
use pgvector::Vector;
use crate::{Error, Result};
// Import necessary models
use crate::models::ai::{WritingAssistantMessage, ChatHistory, MessageRole};

/// Retrieves the top 'k' most relevant document chunks (currently full documents) 
/// based on embedding similarity to the query_embedding.
/// Optionally filters by project_id if provided.
pub async fn semantic_search(
    pool: &PgPool, 
    project_id: Option<i32>, 
    query_embedding: &Vector, 
    k: i64
) -> Result<Vec<String>> {
    
    println!("->> {:<12} - Retrieving relevant chunks (k={}) for project_id: {:?}", "RETRIEVAL", k, project_id);

    let rows = if let Some(p_id) = project_id {
        // Query within a specific project using sqlx::query
        sqlx::query(
            r#"
            SELECT d.content
            FROM documents d
            JOIN document_projects dp ON d.id = dp.document_id
            WHERE dp.project_id = $1 
              AND d.embedding IS NOT NULL 
              AND d.is_trashed = false
            ORDER BY d.embedding <=> $2::vector
            LIMIT $3
            "#
        )
        .bind(p_id)
        .bind(query_embedding) // Bind the vector explicitly
        .bind(k)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            eprintln!("DB Error retrieving project chunks: {:?}", e);
            Error::DatabaseError
        })?
    } else {
        // Query across all documents using sqlx::query
        sqlx::query(
            r#"
            SELECT content
            FROM documents
            WHERE embedding IS NOT NULL 
              AND is_trashed = false
            ORDER BY embedding <=> $1::vector -- Keep cast for clarity/pg side
            LIMIT $2
            "#
        )
        .bind(query_embedding) // Bind the vector explicitly
        .bind(k)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            eprintln!("DB Error retrieving general chunks: {:?}", e);
            Error::DatabaseError
        })?
    };

    // Manually map rows to Vec<String>
    let content_list: Vec<String> = rows.iter()
        .map(|row| row.try_get("content"))
        .filter_map(|res| res.ok())
        .collect();
    
    println!("->> {:<12} - Retrieved {} relevant chunks", "RETRIEVAL", content_list.len());
    Ok(content_list)
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