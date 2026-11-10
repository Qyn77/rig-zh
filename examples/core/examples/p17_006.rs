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
let writer = AgentBuilder::new(model.clone()).build();
let editor = AgentBuilder::new(model.clone()).build();
let coder = AgentBuilder::new(model.clone()).build();
let mathematician = AgentBuilder::new(model.clone()).build();
let generalist = AgentBuilder::new(model.clone()).build();
let sentiment_agent = AgentBuilder::new(model.clone()).build();
let topic_agent = AgentBuilder::new(model.clone()).build();
// BEGIN DOCUMENTATION BLOCK
#[derive(Deserialize, schemars::JsonSchema)]
struct Plan {
    /// One instruction per subtask.
    subtasks: Vec<String>,
}


let orchestrator = AgentBuilder::new(model.clone())
    .preamble("Split the task into 2-4 independent writing subtasks.")
    .build();
let worker = AgentBuilder::new(model.clone())
    .preamble("Complete the writing subtask you are given.")
    .build();
let editor = AgentBuilder::new(model)
    .preamble("Merge the drafts into one coherent text.")
    .build();


let task = "Write a product page for an insulated, plastic-free water bottle.";
let plan = orchestrator.prompt_typed::<Plan>(task).await?.output;


// Run every worker concurrently.
let drafts = futures::future::try_join_all(
    plan.subtasks.iter().map(|subtask| worker.prompt(subtask.as_str()).into_future()),
)
.await?;


let combined = drafts.iter().map(|d| d.output()).collect::<Vec<_>>().join("\n\n");
let page = editor.prompt(combined).await?.output();
println!("{page}");
// END DOCUMENTATION BLOCK
Ok(())
}
