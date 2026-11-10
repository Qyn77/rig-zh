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
use rig::tool::ToolExecutionError;


/// The signed-in user. Set by the host, never by the model.
#[derive(Serialize, Deserialize, rig::ContextValue)]
struct UserId(String);


/// Host-only metadata the tool reports back about its call.
#[derive(Serialize, Deserialize, rig::ContextValue)]
#[context(key = "orders.rows_read")]
struct RowsRead(usize);


#[derive(Deserialize)]
struct NoArgs {}


struct ListOrders;


impl Tool for ListOrders {
    const NAME: &'static str = "list_orders";
    type Args = NoArgs;
    type Output = Vec<String>;
    type Error = ToolExecutionError;


    fn description(&self) -> String {
        "List the signed-in user's recent orders".to_string()
    }


    fn parameters(&self) -> serde_json::Value {
        json!({ "type": "object", "properties": {} })
    }


    async fn call(&self, ctx: &mut ToolContext, _args: NoArgs) -> Result<Vec<String>, ToolExecutionError> {
        let user = ctx.require::<UserId>()?;
        let orders = vec![format!("order #1001 for {}", user.0)];
        ctx.insert_result(RowsRead(orders.len()))?;
        Ok(orders)
    }
}
// BEGIN DOCUMENTATION BLOCK
use rig::tool::ToolContext;


let mut context = ToolContext::new();
context.insert(UserId("u_42".to_string()))?;


let reply = agent
    .prompt("What did I order recently?")
    .tool_context(context)
    .max_turns(3)
    .await?;
// END DOCUMENTATION BLOCK
Ok(())
}
