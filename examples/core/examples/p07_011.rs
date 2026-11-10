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
use rig::agent::{AgentHook, HookContext, ModelSelection, ModelSelectionAction};


/// A cheap model for the first call; the stronger one once tool results are in.
struct CheapFirst;


impl AgentHook for CheapFirst {
    fn on_model_select(&self, ctx: &HookContext, _event: ModelSelection<'_>) -> ModelSelectionAction {
        if ctx.turn() == 1 {
            ModelSelectionAction::select("fast")
        } else {
            ModelSelectionAction::select("strong")
        }
    }
}


let client = OpenAI::from_env()?;
let agent = AgentBuilder::named_model("fast", client.completion(openai::GPT_5_4_MINI))
    .model_route("strong", client.completion(openai::GPT_5_5))
    .add_hook(CheapFirst)
    .build();


// Or skip the hook and pick one model for a whole run:
let response = agent.prompt("Summarize Rust's ownership rules.").using_model("strong").await?;
// END DOCUMENTATION BLOCK
Ok(())
}
