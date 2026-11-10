use rig::memory::{MemoryStore, load, save};
use rig::message::Message;

fn main() -> anyhow::Result<()> {
    // Store / AsyncStore replace ConversationMemory. load/save manage session format.
    let store = MemoryStore::default();
    let mut conversation = load(&store, "user-42")?;
    conversation.extend([Message::user("My name is Ada."), Message::assistant("Hello, Ada!")]);
    save(&store, "user-42", &mut conversation)?;
    let resumed = load(&store, "user-42")?;
    assert_eq!(resumed.messages(), conversation.messages());
    println!("saved and restored {} messages", resumed.messages().len());
    Ok(())
}
