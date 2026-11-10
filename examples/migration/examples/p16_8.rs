#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rig::message::Message;
    use rig::providers::openai::{self, OpenAI};
    let agent = rig::AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5)).build();
    let mut history = vec![Message::user("What is Rust?"),
        Message::assistant("Rust is a systems programming language.")];
    let response = agent.prompt("Who created it?").history(history.clone()).await?;
    println!("{}", response.output());
    history.extend(response.messages);
    Ok(())
}
