use futures::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5))
        .preamble("You are a helpful assistant.")
        .build();


    // Await the runner: run the loop to completion and get a `PromptResponse`.
    let response = agent
        .prompt("Find the answer and show your work.")
        .max_turns(5)
        .await?;


    println!("answer: {}", response.output());
    println!("model calls: {}", response.completion_calls.len());
    println!("tokens: {:?}", response.usage.total_tokens);


    // Or stream the same run: deltas and tool activity, then the final response.
    let mut stream = agent.prompt("Now explain it to a child.").max_turns(5).stream();
    while let Some(item) = stream.next().await {
        if let MultiTurnStreamItem::FinalResponse(done) = item? {
            println!("\n{}", done.output());
        }
    }


    Ok(())
}
