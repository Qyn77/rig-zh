use futures::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};
use rig::streaming::{Item, StreamEvent};
use std::io::Write;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5))
        .preamble("You are a storyteller.")
        .build();


    let mut stream = agent.prompt("Tell me a short story about a robot.").stream();


    while let Some(item) = stream.next().await {
        match item? {
            MultiTurnStreamItem::StreamAssistantItem(Item::Event(StreamEvent::Text { text, .. })) => {
                print!("{text}");
                std::io::stdout().flush()?;
            }
            MultiTurnStreamItem::FinalResponse(response) => {
                println!("\n\n{} tokens used", response.usage().total_tokens.unwrap_or(0));
            }
            _ => {}
        }
    }


    Ok(())
}
