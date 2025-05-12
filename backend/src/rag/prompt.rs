use crate::models::ai::{ChatHistory, MessageRole, ContextDocument, ProactiveDiffContextPayload};
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

    prompt.push_str(
        "You are a helpful writing assistant. \
        Use the following 'Relevant Context' retrieved from the user's documents \
        and the 'Chat History' to answer the 'User Query'. \
        Synthesize information from the context and history to provide a specific and helpful response. \
        If the context contains information relevant to the query, use it directly in your answer. \
        Your response should be plain text only, without any markdown, HTML, or code formatting.\n\n"
    );

    prompt.push_str("Current Document Focus:\n");
    match (current_doc_id, current_doc_name) {
        (Some(id), Some(name)) => prompt.push_str(&format!("- ID: {}, Name: {}\n\n", id, name)),
        (Some(id), None) => prompt.push_str(&format!("- ID: {}\n\n", id)),
        _ => prompt.push_str("- No specific document associated with this chat.\n\n"),
    }
    prompt.push_str("---\n\n");

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
        history_str.insert_str(0, &message_line);
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
    // Add a stronger, final instruction for plain text output
    prompt.push_str("\n\nIMPORTANT: Generate the response as plain text ONLY. Do NOT use any Markdown (like **, lists, etc.), HTML, or other formatting.\n\nAssistant Response:");
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

/// Constructs a prompt for applying an AI suggestion across project documents.
pub fn construct_apply_suggestion_prompt(
    project_documents: &[(i32, String, String)], // List of (id, name, content)
    suggestion_to_apply: &str,
    active_document_id: Option<i32>, // New parameter
) -> Result<String, serde_json::Error> { 
    let mut prompt = String::new();

    // --- System Instructions ---
    let mut system_instruction = String::from("You are an AI assistant tasked with applying a given suggestion to a set of documents within a project. ");

    if let Some(active_id) = active_document_id {
        system_instruction.push_str(&format!(
            "The user is currently focused on Document ID: {}. Prioritize applying the suggestion to this document. ", active_id
        ));
        
        // Check if the active document is empty
        if let Some((_, _, active_doc_content)) = project_documents.iter().find(|(id, _, _)| *id == active_id) {
            if active_doc_content.trim().is_empty() {
                system_instruction.push_str(
                    "This active document is currently empty. If the 'Suggestion to Apply' is suitable as new content for an empty document (e.g., a complete story, article, or section), then the 'new_content' for this active document should be the 'Suggestion to Apply' itself. "
                );
            } else {
                system_instruction.push_str(
                    "This active document has existing content. Determine how the 'Suggestion to Apply' modifies this existing content. "
                );
            }
        }
    } else {
        system_instruction.push_str(
            "No specific document is marked as active. Analyze the provided 'Project Documents' and the 'Suggestion to Apply'. "
        );
    }

    system_instruction.push_str(
        "Determine which documents need modification based on the suggestion. For ONLY the documents that need changes, generate their complete new content. Your response MUST be a JSON array containing objects, where each object represents a changed document and has the following structure: { \"document_id\": <integer>, \"new_content\": \"<full_new_document_content_as_string>\" }. Do NOT include documents that remain unchanged in the JSON array. Ensure the 'new_content' is the complete text of the document after applying the suggestion. If the suggestion cannot be applied or no documents need changes, return an empty JSON array []. Output ONLY the JSON array, with no other text before or after it. Do not return any markdown text!\n\n"
    );
    prompt.push_str(&system_instruction);

    // --- Add Project Documents ---
    prompt.push_str("Project Documents:\n");
    let context_docs: Vec<ContextDocument> = project_documents
        .iter()
        .map(|(id, name, content)| ContextDocument {
            id: *id,
            name: name.clone(),
            content: content.clone(),
        })
        .collect();

    // Serialize the documents context into a JSON string for clarity in the prompt
    let docs_json = serde_json::to_string_pretty(&context_docs)?; // Use pretty print for readability
    prompt.push_str("```json\n");
    prompt.push_str(&docs_json);
    prompt.push_str("\n```\n\n");
    prompt.push_str("---\n\nSuggestion to Apply:\n");
    prompt.push_str(suggestion_to_apply);
    prompt.push_str("\n\n---\n\n");
    prompt.push_str("JSON Response (array of changed documents, or [] if none):\n");

    println!("->> {:<12} - Apply Suggestion Prompt constructed ({} chars)", "PROMPT", prompt.len()); // Use char count for large prompts

    Ok(prompt)
}

pub fn construct_proactive_diff_decision_prompt(
    ai_response_content: &str,
    context: &ProactiveDiffContextPayload,
    document_content_snippet: Option<&str>, 
) -> String {
    // System instruction for the LLM
    // You are a decision-making AI. Based on the AI's response content, the user's context, and an optional snippet of the current document, decide if it is appropriate to proactively show a diff to the user. 

    // Restore user_intent_description logic
    let user_intent_description = match context.r#type.as_str() {
        "chat" => format!("User asked: '{}' in chat.", context.user_prompt.as_deref().unwrap_or("N/A")),
        "command" => format!("User invoked command: '{}' with prompt: '{}'.", 
                                context.command_name.as_deref().unwrap_or("N/A"), 
                                context.user_prompt.as_deref().unwrap_or("N/A")),
        _ => "User performed an unspecified action.".to_string(),
    };

    // Create document_context_description
    let document_context_description = if let Some(snippet) = document_content_snippet {
        if snippet.trim().is_empty() {
            "The current document is empty.".to_string()
        } else {
            format!("The current document contains: \"...{}...\".", snippet.chars().take(100).collect::<String>())
        }
    } else {
        "No specific document content snippet provided (assume it might be empty or irrelevant to this decision).".to_string()
    };
    
    // Construct the prompt with clear instructions for the LLM
    let mut prompt_segments: Vec<String> = Vec::new();

    prompt_segments.push("You are an AI assistant that helps decide if a user interface should proactively show a diff view. Your sole output MUST be 'True' or 'False'.".to_string());
    prompt_segments.push(format!("User's action: {}", user_intent_description));
    prompt_segments.push(format!("Current document state: {}", document_context_description));
    prompt_segments.push(format!("AI's response to user: \"{}\".", ai_response_content.chars().take(500).collect::<String>())); // Limit length of AI response in prompt
    prompt_segments.push("Decision criteria:".to_string());
    prompt_segments.push("- If the AI's response is a direct answer, question, or general statement that doesn't imply changes to a document, output: False".to_string());
    prompt_segments.push("- If the AI's response explicitly suggests or implies content to be added, removed, or modified in a document (e.g., writing a story, suggesting a paragraph, providing code), output: True".to_string());
    prompt_segments.push("- If the current document is empty and the AI's response is substantial content, output: True".to_string());
    prompt_segments.push("- If unsure, lean towards False.".to_string());
    prompt_segments.push("Based on the above, should a diff be proactively shown to the user? Output True or False.".to_string());

    // Join segments to form the final prompt string
    prompt_segments.join("\n\n")
}

/// Constructs a prompt for sanitizing text by removing HTML and Markdown.
pub fn construct_sanitize_text_prompt(text_with_markdown_html: &str) -> String {
    format!(
        "You are a text sanitization AI. Your task is to remove ALL HTML tags (e.g., <p>, <div>, <span>, <img>) and ALL Markdown syntax (e.g., **, _, #, ##, ```, [link](url), ![image](url), lists like * or - or 1.) from the provided text. \
        Preserve the original textual content and its meaning as much as possible. \
        If the input text consists *only* of HTML/Markdown and would result in an empty string after sanitization, return an empty string. \
        Do NOT add any explanations, apologies, or introductory/concluding phrases. \
        Return ONLY the sanitized plain text.\
\n\
        Text to Sanitize:\
        ```text\
        {}\n        ```\n\n\
        Sanitized Text:",
        text_with_markdown_html
    )
}
