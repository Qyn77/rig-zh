use rig::prelude::*;
use serde_json::json;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
use rig::prelude::*;
use rig::providers::ollama::extension::{KeepAlive, OllamaOptions};
use rig::providers::ollama::{self, Ollama};


let options = OllamaOptions::default()
    .num_ctx(16_384)
    .keep_alive(KeepAlive::duration("10m"));


let agent = AgentBuilder::new(Ollama::new().native_completion(ollama::QWEN3))
    .provider_option(options)
    .build();
Ok(())
}
