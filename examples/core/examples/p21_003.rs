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
use rig::test_utils::{MockAddTool, MockCompletionModel, MockTurn};


let model = MockCompletionModel::from_turns([
    // Turn 1: the model asks to call the `add` tool.
    MockTurn::tool_call("call_1", "add", json!({ "x": 2, "y": 3 })),
    // Turn 2: after seeing the tool result, it answers.
    MockTurn::text("2 + 3 = 5"),
]);
let probe = model.clone();


// Attach your real tool here; `MockAddTool` is a ready-made one.
let agent = AgentBuilder::new(model).tool(MockAddTool).build();
let answer = agent.prompt("What is 2 + 3?").max_turns(3).await?;


assert_eq!(answer.output(), "2 + 3 = 5");
assert_eq!(probe.request_count(), 2); // the loop called the model twice
// END DOCUMENTATION BLOCK
Ok(())
}
