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
use rig::tool::ToolEmbedding;


#[derive(Debug, thiserror::Error)]
#[error("init error")]
struct InitError;


#[derive(Deserialize)]
struct AddArgs { x: i64, y: i64 }


struct Add;


impl Tool for Add {
    const NAME: &'static str = "add";
    type Args = AddArgs;
    type Output = i64;
    type Error = std::convert::Infallible;


    fn description(&self) -> String {
        "Add x and y together".to_string()
    }


    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": { "x": { "type": "integer" }, "y": { "type": "integer" } },
            "required": ["x", "y"]
        })
    }


    async fn call(&self, _ctx: &mut ToolContext, args: AddArgs) -> Result<i64, Self::Error> {
        Ok(args.x + args.y)
    }
}


impl ToolEmbedding for Add {
    type InitError = InitError;
    type Context = ();
    type State = ();


    fn init(_state: (), _context: ()) -> Result<Self, InitError> {
        Ok(Add)
    }


    fn embedding_docs(&self) -> Vec<String> {
        vec!["Add x and y together".into(), "sum, plus, addition".into()]
    }


    fn context(&self) {}
}
// BEGIN DOCUMENTATION BLOCK
use rig::tool::ToolSet;


let client = OpenAI::from_env()?;
let embedding_model = client.embedding(openai::TEXT_EMBEDDING_3_SMALL, None).erase();


let mut toolset = ToolSet::default();
toolset.add_retrieved_tool(Add)?;


let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
    .documents(toolset.schemas()?)?
    .build()
    .await?;
let index = InMemoryVectorStore::from_documents_with_id_f(embeddings, |tool| tool.name.clone())
    .index(embedding_model);


let agent = AgentBuilder::new(client.completion(openai::GPT_5_5))
    .preamble("You are a calculator.")
    .retrieved_tools(1, index, toolset)
    .build();
// END DOCUMENTATION BLOCK
Ok(())
}
