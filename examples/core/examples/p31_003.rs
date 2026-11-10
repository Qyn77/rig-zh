use rig::completion::PromptError;
use rig::integrations::cli_chatbot::{Chat, ChatBotBuilder};
use rig::message::Message;
use rig::wasm_compat::WasmCompatSend;


/// Replies with the length of the conversation so far.
struct Echo;


impl Chat for Echo {
    async fn chat(
        &self,
        prompt: impl Into<Message> + WasmCompatSend,
        history: &mut Vec<Message>,
    ) -> Result<String, PromptError> {
        let prompt: Message = prompt.into();
        history.push(prompt);
        let reply = format!("turn {}", history.len());
        history.push(Message::assistant(reply.clone()));
        Ok(reply)
    }
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    ChatBotBuilder::new().chat(Echo).build().run().await?;
    Ok(())
}
