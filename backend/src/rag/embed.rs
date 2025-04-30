use langchain_rust::embedding::{openai::OpenAiEmbedder, Embedder};
use langchain_rust::llm::OpenAIConfig;
use std::env;
use crate::Error;
use pgvector::Vector;
use crate::models::ai::WritingAssistantMessage;
use sqlx::PgPool;
use chrono::Utc;
use crate::models::ai::MessageRole;

pub struct EmbeddingModel {
    model: OpenAiEmbedder<OpenAIConfig>
}

impl EmbeddingModel {
    pub fn new() -> Result<Self, Error> {    
        let embedding_model = OpenAiEmbedder::new({
            OpenAIConfig::default().with_api_key(env::var("OPENAI_API_KEY")
        .map_err(|_| Error::APIKeyError)?)
        });
          
        Ok(Self { model: embedding_model })
    }

    pub async fn embed_message(&self, message: &WritingAssistantMessage) -> Result<Vector, Error> {
        let embedding_vec_f64 = self.model.embed_query(&message.content).await
            .map_err(|e| {
                eprintln!("OpenAI embedding query failed for message: {:?}", e);
                Error::EmbeddingError
            })?;
        
        // Map to pgvector f32
        let embedding_vec_f32: Vec<f32> = embedding_vec_f64.into_iter().map(|f| f as f32).collect();
        Ok(Vector::from(embedding_vec_f32))
    }

    pub async fn embed_document(&self, content: &str) -> Result<Vector, Error> {
        let embedding_vec_f64 = self.model.embed_query(content).await
            .map_err(|e| {
                eprintln!("OpenAI embedding query failed for document: {:?}", e);
                Error::EmbeddingError
            })?;
        
        // Map to pgvector f32
        let embedding_vec_f32: Vec<f32> = embedding_vec_f64.into_iter().map(|f| f as f32).collect();
        Ok(Vector::from(embedding_vec_f32))
    }
}

// Function to embed and store a user message, now returns the embedding Vector
pub async fn embed_and_store_user_message(
    embedding_model: &EmbeddingModel,
    pool: &PgPool,
    session_id: i32,
    content: &WritingAssistantMessage,
) -> Result<Vector, Error> {
    println!("->> {:<12} - Embedding user message content", "EMBED");
    let user_embedding = embedding_model.embed_message(content).await?;

    // Store User Message (with embedding)
    println!("->> {:<12} - Storing user message", "EMBED");
    sqlx::query!(
        r#"
        INSERT INTO writing_assistant_messages (session_id, role, content, created_at, embedding)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        session_id,
        MessageRole::User as _,
        content.content,
        Utc::now().naive_utc(),
        user_embedding as _
    )
    .execute(pool)
    .await
    .map_err(|e| {
        eprintln!("DB Error storing user message: {:?}", e);
        Error::DatabaseError
    })?;

    // Remove the session timestamp update from here, it belongs in the controller
    // sqlx::query!( ... ).execute(pool).await.map_err(|_| Error::DatabaseError)?;

    Ok(user_embedding) // Return the calculated embedding
}

// Function to embed and store an assistant message
pub async fn embed_and_store_assistant_message(
    embedding_model: &EmbeddingModel,
    pool: &PgPool,
    session_id: i32,
    content: &str,
) -> Result<(), Error> { // Return Result<(), Error> for now
    println!("->> {:<12} - Embedding assistant message content", "EMBED");
    // Embed the provided content directly using the passed model instance
    let assistant_embedding = embedding_model.embed_document(content).await?;

    // Store Assistant Message (with embedding)
    println!("->> {:<12} - Storing assistant message", "EMBED");
    sqlx::query!(
        r#"
        INSERT INTO writing_assistant_messages (session_id, role, content, created_at, embedding)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        session_id,
        MessageRole::Assistant as _, // Use the enum
        content, // Use the passed content
        Utc::now().naive_utc(),
        assistant_embedding as _
    )
    .execute(pool) // Use the passed pool
    .await
    .map_err(|e| {
        eprintln!("DB Error storing assistant message: {:?}", e);
        Error::DatabaseError // Map to your custom DB error
    })?;
    Ok(())
}