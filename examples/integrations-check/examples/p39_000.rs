use rig::prelude::*;
use serde_json::json;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
use rig::prelude::*;
use rig::providers::ollama::{self, Ollama};


let client = Ollama::new();


let agent = AgentBuilder::new(client.completion(ollama::QWEN3))
    .preamble("You are a concise assistant.")
    .build();


println!("{}", agent.prompt("What is Rust's borrow checker?").await?.output());
Ok(())
}
