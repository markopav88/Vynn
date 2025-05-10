use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type, PartialEq)]
#[sqlx(type_name = "message_role_enum", rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WritingAssistantSession {
    pub id: i32,
    pub user_id: i32,
    pub document_id: Option<i32>,
    pub title: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WritingAssistantMessage {
    pub id: i32,
    pub session_id: i32,
    pub role: MessageRole,
    pub content: String,
    pub created_at: NaiveDateTime,
}

pub struct SessionWithMessageContent {
    pub id: i32,
    pub user_id: i32,
    pub document_id: Option<i32>,
    pub title: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub last_message_content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionPayload {
    pub document_id: Option<i32>,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct SendMessagePayload {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct SelectedTextContext {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct RewritePayload {
    pub content: String,
    pub style: String,
}

#[derive(Debug, Serialize)]
pub struct SessionWithMessages {
    pub session: WritingAssistantSession,
    pub messages: Vec<WritingAssistantMessage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
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

/// Struct for API response when getting all sessions, including a snippet of the last message.
#[derive(Debug, Serialize, sqlx::FromRow)] 
pub struct WritingAssistantSessionWithSnippet {
    pub id: i32,
    pub user_id: i32,
    pub document_id: Option<i32>,
    pub title: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub last_message_snippet: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApplySuggestionPayload {
    pub suggestion_content: String,
    #[serde(rename = "current_document_id")]
    pub current_document_id: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct SuggestedDocumentChange {
    pub document_id: i32,
    pub old_content: String,
    pub new_content: String,
}

#[derive(Debug, Deserialize)]
pub struct LlmDocChange {
    pub document_id: i32,
    pub new_content: String,
}

#[derive(Serialize)]
pub struct ContextDocument {
    pub id: i32,
    pub name: String,
    pub content: String,
}


#[derive(Debug, Deserialize, Clone)]
pub struct ProactiveDiffContextPayload {
    pub r#type: String,
    #[serde(rename = "commandName")]
    pub command_name: Option<String>,
    #[serde(rename = "userPrompt")]
    pub user_prompt: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DecisionAgentPayload {
    #[serde(rename = "aiResponseContent")]
    pub ai_response_content: String,
    pub context: ProactiveDiffContextPayload,
    #[serde(rename = "documentContentSnippet")]
    pub document_content_snippet: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DecisionAgentResponse {
    pub decision: String,
}

#[derive(Debug, Deserialize)]
pub struct SanitizeTextPayload {
    pub text_to_sanitize: String,
}

#[derive(Debug, Serialize)]
pub struct SanitizeTextResponse {
    pub sanitized_text: String,
}
