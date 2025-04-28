use langchain_rust::language_models::llm::LLM;
use langchain_rust::llm::openai::OpenAI;
use langchain_rust::llm::OpenAIConfig;
use std::env;
use crate::Error;

pub struct QueryModel {
    model: OpenAI<OpenAIConfig>
}

impl QueryModel {
    pub fn new() -> Result<Self, Error> {
        let open_ai = OpenAI::default().with_config(
            OpenAIConfig::default().with_api_key(
                env::var("OPENAI_API_KEY")
                    .map_err(|_| Error::APIKeyError)?
            )
        );
        Ok(Self { model: open_ai })
    }

    pub async fn query_model(&self, prompt: &str) -> Result<String, Error> {
        self.model.invoke(prompt).await
            .map_err(|_err| {
                eprintln!("LLM Query Error occurred");
                Error::LlmQueryError
            })
    }
}