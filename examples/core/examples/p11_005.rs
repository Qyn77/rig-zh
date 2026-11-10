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
use rig::tool::ToolExecutionError;


#[derive(Debug, thiserror::Error)]
enum LookupError {
    #[error("no customer with id {0}")]
    NotFound(String),
    #[error("database unreachable at {0}")]
    Database(String),
}


#[derive(Deserialize)]
struct LookupArgs {
    id: String,
}


struct LookupCustomer;


impl Tool for LookupCustomer {
    const NAME: &'static str = "lookup_customer";
    type Args = LookupArgs;
    type Output = String;
    type Error = LookupError;


    fn description(&self) -> String {
        "Look up a customer by id".to_string()
    }


    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": { "id": { "type": "string" } },
            "required": ["id"]
        })
    }


    fn map_error(&self, error: LookupError) -> ToolExecutionError {
        match error {
            // Safe and useful for the model: say what went wrong.
            LookupError::NotFound(_) => ToolExecutionError::not_found(error.to_string()),
            // The message names an internal host: log it, tell the model less.
            LookupError::Database(_) => ToolExecutionError::network(error.to_string())
                .with_model_feedback("the customer database is unavailable; try again later")
                .with_code("DB_DOWN")
                .with_source(error),
        }
    }


    async fn call(&self, _ctx: &mut ToolContext, args: LookupArgs) -> Result<String, LookupError> {
        Err(LookupError::NotFound(args.id))
    }
}
// END DOCUMENTATION BLOCK
Ok(())
}
