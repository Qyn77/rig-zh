use rig::Embed;
use rig::embeddings::EmbeddingsBuilder;
use rig::integrations::cli_chatbot::ChatBotBuilder;
use rig::loaders::PdfFileLoader;
use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};
use rig::vector_store::in_memory_store::InMemoryVectorStore;
use serde::Serialize;


#[derive(Embed, Serialize, Clone)]
struct Page {
    id: String,
    #[embed]
    text: String,
}


fn load_pages(glob: &str) -> anyhow::Result<Vec<Page>> {
    let pages = PdfFileLoader::with_glob(glob)?
        .load()
        .ignore_errors()
        .by_page()
        .into_iter()
        .collect::<Result<Vec<String>, _>>()?;


    Ok(pages
        .into_iter()
        .enumerate()
        .filter(|(_, text)| !text.trim().is_empty())
        .map(|(i, text)| Page { id: format!("page-{i}"), text })
        .collect())
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let openai_client = OpenAI::from_env()?;
    let embedding_model = openai_client
        .embedding(openai::TEXT_EMBEDDING_3_SMALL, None)
        .erase();


    let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
        .documents(load_pages("documents/*.pdf")?)?
        .build()
        .await?;


    let index = InMemoryVectorStore::from_documents_with_id_f(embeddings, |page| page.id.clone())
        .index(embedding_model);


    let rag_agent = AgentBuilder::new(openai_client.completion(openai::GPT_5_5))
        .preamble("You are a helpful assistant that answers questions using the provided PDF context.")
        .dynamic_context(4, index)
        .build();


    let chatbot = ChatBotBuilder::new().agent(rag_agent).build();
    chatbot.run().await?;


    Ok(())
}
