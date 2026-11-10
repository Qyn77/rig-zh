#![allow(unused, unreachable_code)]
use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};
use rig::providers::anthropic::{self, Anthropic};
use rig::integrations::cli_chatbot::ChatBotBuilder;
use rig::agent::{AgentHook, HookContext, ModelSelection, ModelSelectionAction};
use rig::{DynModel, ProviderError};
use rig::agent::{Agent, AgentRun, MultiTurnStreamItem};
use rig::completion::{CompletionRequest, FinishReason, PromptError};
use rig::message::{Message, ToolResultContent};
use rig::tool::{ToolContext, builtin::ThinkTool};
use rig::extractor::ExtractorBuilder;
use rig::embeddings::Embedding;
use rig::vector_store::{VectorSearchResult, VectorSearchIdResult, VectorStoreError};
use rig::vector_store::request::SearchFilter;
use rig::image_generation::ImageGenerationRequestBuilder;
use rig::test_utils::MockAddTool as Add;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use schemars::JsonSchema;
use serde_json::json;
use futures::{StreamExt, TryStreamExt};
use std::future::IntoFuture;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, rig::Embed)]
struct WordDefinition { id: String, word: String, #[embed] definitions: Vec<String> }
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct Person { name: Option<String>, age: Option<u8>, profession: Option<String> }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
let client = OpenAI::from_env()?;
let model = client.completion(openai::GPT_5_5);
let agent = AgentBuilder::new(model.clone()).build();
let embedding_model = client.embedding(openai::TEXT_EMBEDDING_3_SMALL, None);
let embeddings: Vec<(crate::WordDefinition, Vec<Embedding>)> = Vec::new();
let store = InMemoryVectorStore::from_documents(embeddings.clone());
let index = store.clone().index(embedding_model.clone());
let extractor = ExtractorBuilder::<Person>::new(model.clone()).build();
let saved = serde_json::to_string(&AgentRun::new("hello"))?;
// BEGIN DOCUMENTATION BLOCK
use rig::agent::{Agent, PromptResponse};
use rig::completion::PromptError;
use std::time::Duration;


/// Prompt an agent, retrying transient failures with exponential backoff.
async fn prompt_with_retry(
    agent: &Agent,
    input: &str,
    max_attempts: u32,
) -> Result<PromptResponse, PromptError> {
    let mut attempt = 0;
    loop {
        attempt += 1;
        let error = match agent.prompt(input).await {
            Ok(response) => return Ok(response),
            Err(error) => error,
        };
        let retryable = match &error {
            PromptError::Provider(e) => e.is_retryable(),
            PromptError::Report(report) => report.is_retryable(),
            _ => false,
        };
        if !retryable || attempt >= max_attempts {
            return Err(error);
        }
        // Prefer the provider's Retry-After header, else back off exponentially.
        let backoff = error
            .provider_response_headers()
            .and_then(|headers| headers.get("retry-after")?.to_str().ok()?.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_millis(200 * 2u64.pow(attempt - 1)));
        eprintln!("attempt {attempt} failed ({error}); retrying in {backoff:?}");
        tokio::time::sleep(backoff).await;
    }
}
// END DOCUMENTATION BLOCK
Ok(())
}
