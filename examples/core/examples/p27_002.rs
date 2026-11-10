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
/// One route. The description and examples are embedded; `name` is the key.
#[derive(Embed, Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
struct Route {
    name: String,
    #[embed]
    description: String,
    #[embed]
    examples: Vec<String>,
}


let embedding_model = OpenAI::from_env()?
    .embedding(openai::TEXT_EMBEDDING_3_SMALL, None)
    .erase();


let routes = vec![
    Route {
        name: "rust".into(),
        description: "Programming and software development in Rust".into(),
        examples: vec![
            "How do I write an async function in Rust?".into(),
            "Why does the borrow checker reject this code?".into(),
        ],
    },
    Route {
        name: "maths".into(),
        description: "Mathematics, calculations and equations".into(),
        examples: vec!["Solve this equation".into(), "What is 15% of 200?".into()],
    },
];


let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
    .documents(routes)?
    .build()
    .await?;
let index = InMemoryVectorStore::from_documents(embeddings).index(embedding_model);


let request = VectorSearchRequest::builder()
    .query("How do I use async with Rust?")
    .samples(1)
    .build();


let route = index
    .top_n::<Route>(request)
    .await?
    .into_iter()
    .next()
    .map_or_else(|| "general".to_owned(), |hit| hit.document.name);
println!("route: {route}");
// END DOCUMENTATION BLOCK
Ok(())
}
