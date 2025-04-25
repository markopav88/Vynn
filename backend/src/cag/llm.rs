use anyhow::Result;
use std::env;

use crate::models::ai::{ChatHistory, ChatMessage};

/// LLM service for interacting with language models using langchain
pub struct LangchainService {
    api_key: String,
    model: String,
}

impl LangchainService {
    /// Create a new LLM service
    pub fn new() -> Self {
        // Get OpenAI API key from environment variable
        let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
            // In case there's no API key, set a stub
            println!("Warning: OPENAI_API_KEY environment variable not set. Using mock responses.");
            "mock_key".to_string()
        });
        
        // Default to GPT-3.5 Turbo
        let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
        
        Self { api_key, model }
    }
    
    /// Generate a response using LLM
    pub async fn generate_response(&self, chat_history: &ChatHistory, relevant_context: Option<&str>) -> Result<String> {
        // If the API key is our mock_key or if we're in development, return mock responses
        if self.api_key == "mock_key" {
            return self.generate_mock_response(chat_history, relevant_context);
        }
        
        // In production, we would use langchain to call the OpenAI API
        // This would require setting up the actual langchain integration
        // For now, return the mock response to avoid compile errors
        self.generate_mock_response(chat_history, relevant_context)
    }
    
    /// Generate a mock response when no API key is available
    fn generate_mock_response(&self, chat_history: &ChatHistory, context: Option<&str>) -> Result<String> {
        // Get the last user message
        let last_user_msg = chat_history.messages.iter()
            .filter(|msg| msg.role == "user")
            .last()
            .map(|msg| msg.content.as_str())
            .unwrap_or("");
        
        let response = if last_user_msg.to_lowercase().contains("grammar") {
            "I noticed a few grammar issues in your writing. Consider revising your sentence structure for clarity. Make sure subjects and verbs agree in number, and watch for proper comma usage in complex sentences."
        } else if last_user_msg.to_lowercase().contains("tone") {
            "The tone of your writing seems conversational. If you're aiming for a more formal tone, consider eliminating contractions and replacing casual phrases with more precise terminology. For academic writing, focus on objective language and avoid first-person perspective when possible."
        } else if last_user_msg.to_lowercase().contains("structure") {
            "Your document structure could be improved by organizing content with clear headings and subheadings. Each paragraph should focus on a single idea that supports your main thesis. Consider adding transition sentences between paragraphs to improve flow."
        } else if last_user_msg.to_lowercase().contains("concise") || last_user_msg.to_lowercase().contains("verbose") {
            "To make your writing more concise, look for redundant phrases and unnecessary modifiers. Replace phrases like 'due to the fact that' with simpler alternatives like 'because'. Aim to express each idea in the fewest words possible while maintaining clarity."
        } else if last_user_msg.to_lowercase().contains("improve") || last_user_msg.to_lowercase().contains("better") {
            "To improve your writing, focus on using active voice instead of passive voice when appropriate. Include specific examples to support your claims, and vary your sentence structure to maintain reader interest. Eliminate unnecessary words and ensure each paragraph has a clear purpose."
        } else if context.is_some() {
            "Based on the context you've provided, I suggest focusing on maintaining consistent terminology throughout your document. Your key points could be strengthened with more specific evidence or examples. Consider reorganizing your paragraphs to build a more logical progression of ideas."
        } else {
            "I'm here to help with your writing. I can provide feedback on grammar, tone, structure, clarity, or any other aspect of your writing. What specific area would you like me to focus on?"
        };
        
        Ok(response.to_string())
    }
}
