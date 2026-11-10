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
use rig::streaming::{Item, PartKind, StreamEvent, parse_partial_arguments};
use std::collections::HashMap;


let mut stream = agent.prompt("Write a haiku to notes.md").max_turns(3).stream();
let mut args_so_far: HashMap<usize, String> = HashMap::new();


while let Some(item) = stream.next().await {
    match item? {
        MultiTurnStreamItem::StreamAssistantItem(Item::Event(event)) => match event {
            StreamEvent::Start { part, kind: PartKind::ToolCall, name } => {
                println!("calling {name:?}");
                args_so_far.insert(part.index(), String::new());
            }
            StreamEvent::Arguments { part, json } => {
                if let Some(args) = args_so_far.get_mut(&part.index()) {
                    args.push_str(&json);
                    println!("  args so far: {:?}", parse_partial_arguments(args));
                }
            }
            _ => {}
        },
        MultiTurnStreamItem::ToolResult { tool_result } => {
            println!("result for {}", tool_result.call);
        }
        _ => {}
    }
}
// END DOCUMENTATION BLOCK
Ok(())
}
