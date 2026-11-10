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
use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};


async fn research_and_write(topic: &str) -> anyhow::Result<String> {
    let model = OpenAI::from_env()?.completion(openai::GPT_5_5).erase();


    let historian = AgentBuilder::new(model.clone())
        .preamble("You summarize the history of a topic in five bullet points.")
        .build();
    let economist = AgentBuilder::new(model.clone())
        .preamble("You summarize the economics of a topic in five bullet points.")
        .build();
    let writer = AgentBuilder::new(model)
        .preamble("You turn research notes into a short, readable article.")
        .build();


    // Two independent workers, run concurrently.
    let (history, economics) = tokio::join!(historian.prompt(topic), economist.prompt(topic));


    // One dependent step that combines their output.
    let notes = format!(
        "History:\n{}\n\nEconomics:\n{}",
        history?.output(),
        economics?.output()
    );
    Ok(writer.prompt(notes).await?.output())
}
// END DOCUMENTATION BLOCK
Ok(())
}
