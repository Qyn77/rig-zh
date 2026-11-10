use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let model = OpenAI::from_env()?.completion(openai::GPT_5_5);


    let agent = AgentBuilder::new(model)
        .preamble("You are a helpful assistant.")
        .temperature(0.7)
        .build();


    let response = agent.prompt("Hello!").await?;
    println!("{}", response.output());


    Ok(())
}
