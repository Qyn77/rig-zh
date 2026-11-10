use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // A client for OpenAI, configured from OPENAI_API_KEY.
    let client = OpenAI::from_env()?;


    // An agent: a completion model plus a system prompt (the "preamble").
    let agent = AgentBuilder::new(client.completion(openai::GPT_5_5))
        .preamble("You are a helpful assistant.")
        .build();


    // Send a prompt and await the model's reply.
    let response = agent.prompt("What is the Rust programming language?").await?;


    println!("{}", response.output());


    Ok(())
}
