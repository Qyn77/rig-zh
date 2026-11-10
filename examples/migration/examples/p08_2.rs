use rig::{AgentBuilder, completion::PromptError};
use rig::memory::MemoryStore;
use rig::providers::openai::{self, OpenAI};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5))
        .memory(MemoryStore::default()).build();
    match agent.prompt("What did I ask earlier?").conversation("user-42").await {
        Ok(response) => println!("{}", response.output()),
        Err(error) => {
            if let PromptError::Memory { error: store_error, response } = &error {
                eprintln!("conversation persistence failed: {store_error}");
                if let Some(finished) = response {
                    eprintln!("finished answer (save not confirmed): {}", finished.output());
                    // Recover/reconcile persistence; do not repeat a tool-using run blindly.
                } else {
                    eprintln!("history could not be loaded; the model was not called");
                }
            }
            return Err(error.into());
        }
    }
    Ok(())
}
