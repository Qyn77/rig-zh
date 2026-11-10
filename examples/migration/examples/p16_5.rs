#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rig::memory::{MemoryStore, load, save};
    use rig::memory::recent::{view, TokenBudget};
    use rig::providers::openai::{self, OpenAI};
    let store = MemoryStore::default();
    let mut conversation = load(&store, "user-42")?;
    let selected = view(conversation.messages(), &TokenBudget::new(4_000)).to_vec();
    let agent = rig::AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5)).build();
    // Restrict only the request window; retain the full stored conversation.
    let response = agent.prompt("Hello!").history(selected).await?;
    println!("{}", response.output());
    conversation.extend(response.messages);
    save(&store, "user-42", &mut conversation)?;
    Ok(())
}
