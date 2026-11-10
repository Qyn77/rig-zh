#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rig::message::Message;
    use rig::providers::openai::{self, OpenAI};
    let agent = rig::AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5))
        .preamble("You are a helpful assistant. Be concise.").build();
    let mut history: Vec<Message> = Vec::new();
    agent.chat("My name is Ada.", &mut history).await?;
    let reply = agent.chat("What's my name?", &mut history).await?;
    println!("{}", reply.output());
    Ok(())
}
