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
// BEGIN DOCUMENTATION BLOCK
use rig::test_utils::{sent_documents, MockCompletionModel, MockEmbeddings};


let embeddings = EmbeddingsBuilder::new(MockEmbeddings::model())
    .documents(["a green alien", "an ancient farming tool"])?
    .build()
    .await?;
let index = InMemoryVectorStore::from_documents(embeddings).index(MockEmbeddings::model());


let model = MockCompletionModel::text("ok");
let probe = model.clone();
let agent = AgentBuilder::new(model).dynamic_context(2, index).build();
agent.prompt("What is a flurbo?").await?;


// Both documents were retrieved and sent as context.
assert_eq!(sent_documents(&probe.requests()[0]).len(), 2);
// END DOCUMENTATION BLOCK
Ok(())
}
