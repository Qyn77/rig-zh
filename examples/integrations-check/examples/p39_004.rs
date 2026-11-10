use rig::prelude::*;
use serde_json::json;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
use rig::providers::openai::{OpenAIConfig, wire::LLAMACPP};


let client = OpenAIConfig::with_key(&LLAMACPP, "")
    .with_base_url("http://gpu-box:8080/v1")
    .client();
Ok(())
}
