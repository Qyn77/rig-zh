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
use anyhow::Result;
use rig::extractor::ExtractorBuilder;
use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize, JsonSchema)]
enum Sentiment {
    Positive,
    Negative,
    Neutral,
}


#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct SentimentClassification {
    sentiment: Sentiment,
    confidence: f32,
}


#[derive(Debug, Deserialize, Serialize, JsonSchema)]
enum EntityType {
    Person,
    Organization,
    Location,
}


#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct Entity {
    text: String,
    entity_type: EntityType,
}


#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct ExtractedEntities {
    entities: Vec<Entity>,
}


// BEGIN DOCUMENTATION BLOCK
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct TextAnalysis {
    sentiment: SentimentClassification,
    entities: Vec<Entity>,
}


let analyzer = ExtractorBuilder::<TextAnalysis>::new(model)
    .append_preamble(
        "For the given text: classify the overall sentiment with a confidence score, \
         and extract named entities (Person, Organization, Location).",
    )
    .build();


let text = "I had a great time visiting Google's headquarters in Mountain View. \
            Sundar Pichai's leadership has been impressive.";
let result = analyzer.extract(text).await?.output;


println!(
    "Sentiment: {:?} ({:.2})",
    result.sentiment.sentiment, result.sentiment.confidence
);
for entity in &result.entities {
    println!("- {:?}: {}", entity.entity_type, entity.text);
}
// END DOCUMENTATION BLOCK
Ok(())
}
