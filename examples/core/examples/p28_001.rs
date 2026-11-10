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


/// Manager-worker pattern with two agents, Alice (manager) and Bob (worker).
async fn manager_worker_agent() -> anyhow::Result<()> {
    // Erase the model once and clone the handle for each agent.
    let model = OpenAI::from_env()?.completion(openai::GPT_5_5).erase();


    let bob = AgentBuilder::new(model.clone())
        .name("bob")
        .description("An employee who works in admin at FooBar Inc.")
        .preamble(
            "You are Bob, an employee in admin at FooBar Inc. Alice, your manager, \
             may ask you to do things. You need to do them.",
        )
        .build();


    let alice = AgentBuilder::new(model)
        .name("alice")
        .preamble("You are a manager in the admin department at FooBar Inc. You manage Bob.")
        .dynamic_tool(bob.into_tool()?)
        .default_max_turns(3)
        .build();


    let res = alice
        .prompt("Ask Bob to write an email for you and let me know what he has written.")
        .await?;


    println!("Response: {}", res.output());
    Ok(())
}
// END DOCUMENTATION BLOCK
Ok(())
}
