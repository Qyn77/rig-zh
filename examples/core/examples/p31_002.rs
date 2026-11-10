use rig::embeddings::EmbeddingsBuilder;
use rig::integrations::cli_chatbot::ChatBotBuilder;
use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};
use rig::vector_store::in_memory_store::InMemoryVectorStore;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = OpenAI::from_env()?;
    let embedding_model = client.embedding(openai::TEXT_EMBEDDING_3_SMALL, None).erase();


    let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
        .documents(vec![
            "Rig is a Rust library for building LLM applications.".to_string(),
            "A Rig agent combines a model, a preamble, context and tools.".to_string(),
        ])?
        .build()
        .await?;
    let index = InMemoryVectorStore::from_documents(embeddings).index(embedding_model);


    let agent = AgentBuilder::new(client.completion(openai::GPT_5_5))
        .preamble("Answer using the retrieved context.")
        .dynamic_context(2, index)
        .build();


    ChatBotBuilder::new().agent(agent).build().run().await?;


    Ok(())
}
