use rig::prelude::*;
use serde_json::json;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
use rig::prelude::*;
use rig::providers::llamacpp;


let client = llamacpp::new("");
let agent = AgentBuilder::new(client.chat(llamacpp::LLAMA_CPP))
    .preamble("You are a concise assistant.")
    .build();
Ok(())
}
