use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

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
    pub role: String,  // "user" or "assistant"
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
    pub role: String,      // "user", "assistant", or "system"
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
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                }
            ],
        }
    }

    /// Add a user message to the chat history
    pub fn add_user_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: "user".to_string(),
            content,
        });
    }

    /// Add an assistant message to the chat history
    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: "assistant".to_string(),
            content,
        });
    }
}