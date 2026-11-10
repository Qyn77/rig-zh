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
use rig::prelude::*;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};


/// Messages exchanged between agents.
#[derive(Debug, Clone)]
enum AgentMessage {
    Task(String),
    Response(String, String), // (from_agent_id, content)
    Trigger(String),
    Shutdown,
}


/// An actor-based autonomous agent. Each holds its own Rig agent, an inbox,
/// its history, and channels to its peers.
struct AutonomousAgent {
    id: String,
    agent: Agent,
    history: Vec<String>,
    inbox: mpsc::Receiver<AgentMessage>,
    peers: Arc<RwLock<Vec<mpsc::Sender<AgentMessage>>>>,
}
// BEGIN DOCUMENTATION BLOCK
use tokio::time::{interval, Duration};


impl AutonomousAgent {
    /// Broadcast a message to every registered peer.
    async fn broadcast(&self, message: AgentMessage) {
        for peer in self.peers.read().await.iter() {
            let _ = peer.send(message.clone()).await;
        }
    }


    async fn run(mut self) {
        let mut tick = interval(Duration::from_secs(10));
        loop {
            tokio::select! {
                Some(msg) = self.inbox.recv() => match msg {
                    AgentMessage::Shutdown => break,
                    AgentMessage::Task(task) => {
                        if let Ok(response) = self.agent.prompt(task).await {
                            let result = response.output();
                            self.history.push(result.clone());
                            self.broadcast(AgentMessage::Response(self.id.clone(), result)).await;
                        }
                    }
                    AgentMessage::Trigger(msg) => {
                        let _ = self.agent.prompt(msg).await;
                    }
                    AgentMessage::Response(from, content) => {
                        self.history.push(format!("From {from}: {content}"));
                    }
                },
                // Autonomous periodic self-check (external trigger)
                _ = tick.tick() => {
                    // e.g. summarize progress, enqueue follow-up work, etc.
                }
            }
        }
    }
}
// END DOCUMENTATION BLOCK
Ok(())
}
