use langchain_rust::embedding::{openai::OpenAiEmbedder, Embedder, EmbedderError};
use langchain_rust::language_models::llm::LLM;
use langchain_rust::llm::openai::OpenAI;
use langchain_rust::llm::OpenAIConfig;
use multipart::server::nickel::nickel::hyper::http::message;
use std::env;
use std::error::Error;

pub struct EmbeddingModel {
    model: OpenAiEmbedder<OpenAIConfig>
}

impl EmbeddingModel {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let embedding_model = OpenAiEmbedder::new({
            OpenAIConfig::default().with_api_key(env::var("OPENAI_API_KEY")?)
        });
        
        Ok(Self { model: embedding_model })
    }

    pub async fn embed_query(&self, message: &str) -> Result<Vec<f64>, EmbedderError> {
        self.model.embed_query(message).await
    }
}

pub struct QueryModel {
    model: OpenAI<OpenAIConfig>
}

impl QueryModel {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let open_ai = OpenAI::default().with_config(
            OpenAIConfig::default()
                .with_api_key(env::var("OPEN_API_KEY")?),
        );

        Ok(Self { model: open_ai})
    }

    pub async fn query_model(&self, message: &str) -> String {
        self.model.invoke(message).await.unwrap()
    }
}