use rig::integrations::cli_chatbot::ChatBotBuilder;
use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5))
        .preamble("You are a helpful assistant.")
        .build();


    // Start the REPL. Type "exit" to quit.
    ChatBotBuilder::new().agent(agent).build().run().await?;


    Ok(())
}
