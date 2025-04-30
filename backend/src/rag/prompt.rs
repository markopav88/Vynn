use crate::models::ai::{ChatHistory, MessageRole};
use crate::rag::retrieval::RetrievedChunk;

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
    context_chunks: &[RetrievedChunk],
    current_doc_id: Option<i32>,
    current_doc_name: Option<&str>
) -> String {
    let mut prompt = String::new();

    // --- Add System Instructions ---
    prompt.push_str(
        "You are a helpful writing assistant. \
        Use the following 'Relevant Context' retrieved from the user's documents \
        and the 'Chat History' to answer the 'User Query'. \
        Synthesize information from the context and history to provide a specific and helpful response. \
        If the context contains information relevant to the query, use it directly in your answer.\n\n"
    );
    // --- End System Instructions ---

    // --- Add Current Document Info ---
    prompt.push_str("Current Document Focus:\n");
    match (current_doc_id, current_doc_name) {
        (Some(id), Some(name)) => prompt.push_str(&format!("- ID: {}, Name: {}\n\n", id, name)),
        (Some(id), None) => prompt.push_str(&format!("- ID: {}\n\n", id)),
        _ => prompt.push_str("- No specific document associated with this chat.\n\n"),
    }
    prompt.push_str("---\n\n");
    // --- End Current Document Info ---

    // Add context if available (truncated if too long)
    prompt.push_str("Relevant Context (from related documents):\n");
    if !context_chunks.is_empty() {
        let mut current_context_tokens = 0;
        for chunk in context_chunks {
            let chunk_header = format!("--- Source Document (ID: {}, Name: {}) ---\n", chunk.document_id, chunk.document_name);
            let chunk_content = &chunk.content;
            let chunk_tokens = estimate_tokens(&chunk_header) + estimate_tokens(chunk_content);

            if current_context_tokens + chunk_tokens > MAX_CONTEXT_TOKENS {
                 println!("->> {:<12} - Context truncated due to length (skipping remaining chunks)", "PROMPT");
                 break; // Stop adding context if limit exceeded
            }
            prompt.push_str(&chunk_header);
            prompt.push_str(chunk_content);
            prompt.push_str("\n---\n"); // Separator after each chunk
            current_context_tokens += chunk_tokens;
        }
        prompt.push_str("\n"); // Add a final newline after context section

    } else {
         prompt.push_str("(No relevant context found from other documents)\n\n"); // Indicate no context was found
    }
    prompt.push_str("---\n\n"); // Separator after context section

    // Add chat history (recent messages first, truncated if too long)
    prompt.push_str("Chat History (Recent first):\n");
    let mut current_history_tokens = 0;
    let mut history_str = String::new();
    // Skip the very first message if it's the initial system prompt from ChatHistory::new()
    for message in chat_history.messages.iter().rev().skip(1) { // <-- Added .skip(1)
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
    if history_str.is_empty() {
        prompt.push_str("(No relevant chat history)\n");
    } else {
        prompt.push_str(&history_str);
    }
    prompt.push_str("\n---\n\n");

    // Add the current user query
    prompt.push_str("User Query:\n");
    prompt.push_str(user_query);
    prompt.push_str("\n\nAssistant Response:");
    // Keep the final log statement
    println!("->> {:<12} - Prompt constructed ({} tokens estimated)", "PROMPT", estimate_tokens(&prompt));

    prompt
}