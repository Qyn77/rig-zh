#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rig::catalog::Catalog;
    use rig::memory::{MemoryStore, load, save, Summarizer, SummaryState, CompactReason, apply_summary};
    use rig::message::Message;
    use rig::providers::openai::{self, OpenAI};
    let store = MemoryStore::default();
    let mut conversation = load(&store, "user-42")?;
    conversation.extend([Message::user("My name is Ada. I am learning Rust."),
        Message::assistant("I will remember that.")]);
    let model = OpenAI::from_env()?.completion(openai::GPT_5_5);
    let resolved = Catalog::builtin().resolve("openai/gpt-5.5")?;
    let reason = CompactReason::Asked { focus: "Keep the user's name and learning goal".into() };
    let request = Summarizer::default().request(conversation.messages(), "", &reason, &resolved.spec)?;
    let reply = model.call(request).await?;
    let state = SummaryState { summary: Summarizer::summary_text(&reply)?, ..Default::default() };
    // None summarizes all messages; pass a MessageId to keep a recent suffix.
    apply_summary(&mut conversation, None, &state)
        .ok_or_else(|| anyhow::anyhow!("summary was not applied"))?;
    save(&store, "user-42", &mut conversation)?;
    println!("{}", state.summary);
    Ok(())
}
