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

pub fn construct_grammar_check_prompt(text: &str) -> String {
    format!(
        "Please correct the grammar and spelling of the following text. Only return the corrected text without any explanations or introductory phrase.\n\n\
        Text to Correct:\n\
        ```\n\
        {}\n\
        ```\n\n\
        If you have no recommended changes or are unable to fix the grammar/spelling for any reason, ONLY return the exact string '__VYNN_NO_CHANGE__'. Otherwise, return ONLY the corrected text.",
        text
    )
}

pub fn construct_spell_check_prompt(text: &str) -> String {
    format!(
        "Please correct only the spelling mistakes in the following text, keeping the original grammar and sentence structure intact. Only return the corrected text without any explanations or introductory phrase.\n\n\
        Text to Correct:\n\
        ```\n\
        {}\n\
        ```\n\n\
        If you find no spelling mistakes or are unable to correct spelling for any reason, ONLY return the exact string '__VYNN_NO_CHANGE__'. Otherwise, return ONLY the corrected text.",
        text
    )
}

pub fn construct_summarize_prompt(text: &str) -> String {
    format!(
        "Please provide a concise summary of the following text. Only return the summary without any explanations or introductory phrase.\n\n\
        Text to Summarize:\n\
        ```\n\
        {}\n\
        ```\n\n\
        If you are unable to summarize the text for any reason, ONLY return the exact string '__VYNN_NO_CHANGE__'. Otherwise, return ONLY the summary.",
        text
    )
}

pub fn construct_rephrase_prompt(text: &str) -> String {
    format!(
        "Please rephrase the following text to improve clarity and flow. Only return the rephrased text without any explanations or introductory phrases.\n\n\
        Text to Rephrase:\n\
        ```\n\
        {}\n\
        ```\n\n\
        If you have no recommended changes or are unable to rephrase for any reason, ONLY return the exact string '__VYNN_NO_CHANGE__'. Otherwise, return ONLY the rephrased text.",
        text
    )
}

pub fn construct_expand_prompt(text: &str) -> String {
    format!(
        "Please expand on the following text, adding more detail and explanation where appropriate. Only return the expanded text without any explanations or introductory phrases.\n\n\
        Text to Expand:\n\
        ```\n\
        {}\n\
        ```\n\n\
        If you have no recommended changes or are unable to expand for any reason, ONLY return the exact string '__VYNN_NO_CHANGE__'. Otherwise, return ONLY the expanded text.",
        text
    )
}

pub fn construct_shrink_prompt(text: &str) -> String {
    format!(
        "Please shrink the following text, making it more concise while retaining the core meaning. Only return the shrinked text without any explanations or introductory phrases.\n\n\
        Text to Shrink:\n\
        ```\n\
        {}\n\
        ```\n\n\
        If you have no recommended changes or are unable to shrink for any reason, ONLY return the exact string '__VYNN_NO_CHANGE__'. Otherwise, return ONLY the shrinked text.",
        text
    )
}

pub fn construct_rewrite_prompt(text: &str, style: &str) -> String {
    format!(
        "Please rewrite the following text in the style of '{}'. Only return the rewritten text without any explanations or introductory phrases.\n\n\
        Text to Rewrite:\n\
        ```\n\
        {}\n\
        ```\n\n\
        If you are unable to rewrite the text for any reason, ONLY return the exact string '__VYNN_NO_CHANGE__'. Otherwise, return ONLY the rewritten text.",
        style, text
    )
}

pub fn construct_fact_check_prompt(text: &str) -> String {
    format!(
        "Please critically evaluate the factual claims in the following text based on your knowledge. Identify any potential inaccuracies or statements that might require verification. Respond concisely.\n\n\
        Text to Fact-Check:\n\
        ```\n\
        {}\n\
        ```\n\n\
        If you are unable to fact-check the text for any reason, ONLY return the exact string '__VYNN_NO_CHANGE__'. Otherwise, return ONLY your concise evaluation.",
        text
    )
}