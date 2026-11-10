#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rig::memory::{MemoryStore, load};
    use rig::providers::openai::{self, OpenAI};
    let store = MemoryStore::default();
    let agent = rig::AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5))
        .preamble("You are a helpful assistant.")
        .memory(store.clone()).build();
    agent.prompt("My name is Ada.").conversation("user-42").await?;
    let reply = agent.prompt("What's my name?").conversation("user-42").await?;
    println!("{}", reply.output());
    assert_eq!(load(&store, "user-42")?.messages().len(), 4);
    Ok(())
}
