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
use std::collections::HashMap;


let openai = OpenAI::from_env()?;
let anthropic = Anthropic::from_env()?;


// Specialists from different providers share the `Agent` type.
let routes: HashMap<&str, Agent> = HashMap::from([
    (
        "rust",
        AgentBuilder::new(anthropic.completion(anthropic::CLAUDE_SONNET_5_5))
            .preamble("You are an expert Rust programmer.")
            .build(),
    ),
    (
        "maths",
        AgentBuilder::new(openai.completion(openai::GPT_5_5))
            .preamble("You are a mathematician who explains each step.")
            .build(),
    ),
]);


let router = AgentBuilder::new(openai.completion(openai::GPT_5_MINI))
    .preamble(
        "Classify the user's question. Reply with exactly one word from: rust, maths. \
         No other text.",
    )
    .build();


let question = "How do I use async with Rust?";
let topic = router.prompt(question).await?.output();


let Some(agent) = routes.get(topic.trim()) else {
    anyhow::bail!("no route for {topic:?}");
};
println!("{}", agent.prompt(question).await?.output());
// END DOCUMENTATION BLOCK
Ok(())
}
