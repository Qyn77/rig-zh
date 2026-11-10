use rig::prelude::*;
use serde_json::json;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
use rig::providers::ollama::{self, Ollama};


let embedder = Ollama::new().embedding(ollama::NOMIC_EMBED_TEXT, None);
Ok(())
}
