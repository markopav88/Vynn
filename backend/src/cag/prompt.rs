// Re-export from models to maintain backward compatibility
pub use crate::models::ai::{ChatHistory, ChatMessage};

/// Builds a full prompt for the LLM, combining user query with relevant context
pub fn build_writing_assistant_prompt(
    chat_history: &ChatHistory,
    relevant_context: Option<String>,
) -> String {
    let mut prompt = String::new();
    
    // Add relevant context if available
    if let Some(context) = relevant_context {
        prompt.push_str("### Relevant Document Context:\n");
        prompt.push_str(&context);
        prompt.push_str("\n\n");
    }
    
    // Add the chat history
    for message in &chat_history.messages {
        prompt.push_str(&format!("{}: {}\n\n", message.role, message.content));
    }
    
    prompt
}