// TODO prompts 
// Look at https://github.com/Abraxas-365/langchain-rust/blob/main/examples/conversational_retriever_chain_with_vector_store.rs

use crate::models::ai::{ChatHistory, MessageRole};

const MAX_HISTORY_TOKENS: usize = 1000; // Example token limit for history
const MAX_CONTEXT_TOKENS: usize = 1500; // Example token limit for context

// Basic token counting heuristic (split by space)
fn estimate_tokens(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Constructs a generic prompt for the LLM using chat history and context.
pub fn construct_generic_prompt(
    user_query: &str,
    chat_history: &ChatHistory, 
    context: Option<String>
) -> String {
    let mut prompt = String::new();

    // Add context if available (truncated if too long)
    if let Some(mut ctx) = context {
        if estimate_tokens(&ctx) > MAX_CONTEXT_TOKENS {
            // Simple truncation - more sophisticated methods exist
            let mut truncated_ctx = String::new();
            for word in ctx.split_whitespace().take(MAX_CONTEXT_TOKENS) {
                truncated_ctx.push_str(word);
                truncated_ctx.push(' ');
            }
            ctx = truncated_ctx.trim_end().to_string();
            println!("->> {:<12} - Context truncated due to length", "PROMPT");
        }
        prompt.push_str("Relevant Context:\n");
        prompt.push_str(&ctx);
        prompt.push_str("\n\n---\n\n");
    }

    // Add chat history (recent messages first, truncated if too long)
    prompt.push_str("Chat History (Recent first):\n");
    let mut current_history_tokens = 0;
    let mut history_str = String::new();
    for message in chat_history.messages.iter().rev() {
        let role_str = match message.role {
            MessageRole::User => "User",
            MessageRole::Assistant => "Assistant",
        };
        let message_line = format!("{}: {}\n", role_str, message.content);
        let message_tokens = estimate_tokens(&message_line);

        if current_history_tokens + message_tokens > MAX_HISTORY_TOKENS {
            println!("->> {:<12} - History truncated due to length", "PROMPT");
            break; // Stop adding history if limit exceeded
        }
        history_str.insert_str(0, &message_line); // Prepend to keep chronological order in final string
        current_history_tokens += message_tokens;
    }
    prompt.push_str(&history_str);
    prompt.push_str("\n---\n\n");

    // Add the current user query
    prompt.push_str("User Query:\n");
    prompt.push_str(user_query);
    prompt.push_str("\n\nAssistant Response:"); // Prompt the LLM for its response

    println!("->> {:<12} - Prompt constructed ({} tokens estimated)", "PROMPT", estimate_tokens(&prompt));
    prompt
}