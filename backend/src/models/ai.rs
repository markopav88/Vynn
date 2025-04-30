use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

// Define the Rust enum corresponding to the SQL enum
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type, PartialEq)]
#[sqlx(type_name = "message_role_enum", rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

/// Represents a chat session between a user and the writing assistant
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WritingAssistantSession {
    pub id: i32,
    pub user_id: i32,
    pub document_id: Option<i32>,
    pub title: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Represents a single message in a chat session
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WritingAssistantMessage {
    pub id: i32,
    pub session_id: i32,
    pub role: MessageRole,  // Changed to use the enum
    pub content: String,
    pub created_at: NaiveDateTime,
}

/// Payload for creating a new chat session
#[derive(Debug, Deserialize)]
pub struct CreateSessionPayload {
    pub document_id: Option<i32>,
    pub title: String,
}

/// Payload for sending a new message
#[derive(Debug, Deserialize)]
pub struct SendMessagePayload {
    pub content: String,
}

/// Complete session with messages for API responses
#[derive(Debug, Serialize)]
pub struct SessionWithMessages {
    pub session: WritingAssistantSession,
    pub messages: Vec<WritingAssistantMessage>,
}

/// Represents a chat message from either the user or assistant (langchain format)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: MessageRole, // Changed to use the enum here too for consistency
    pub content: String,   // The actual message content
}

/// Represents a complete conversation history 
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatHistory {
    pub messages: Vec<ChatMessage>,
}

impl ChatHistory {
    /// Create a new chat history with a system prompt
    pub fn new() -> Self {
        let system_prompt = "You are a helpful writing assistant. Your goal is to help the user improve their writing, \
                            provide suggestions, and answer questions about their documents. Focus on being constructive \
                            and providing clear, actionable feedback that helps the user improve their writing.";
        
        Self {
            messages: vec![
                ChatMessage {
                    role: MessageRole::User,
                    content: system_prompt.to_string(),
                }
            ],
        }
    }

    /// Add a user message to the chat history
    pub fn add_user_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content,
        });
    }

    /// Add an assistant message to the chat history
    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content,
        });
    }
}