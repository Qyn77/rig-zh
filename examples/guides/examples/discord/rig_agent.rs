use std::path::Path;


use anyhow::{Context, Result};
use rig::embeddings::EmbeddingsBuilder;
use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};
use rig::vector_store::in_memory_store::InMemoryVectorStore;


const PREAMBLE: &str = "\
You are a helpful assistant in a Discord server. Answer questions using the documents \
provided in your context, and say so when they do not cover the question. Keep answers \
short: Discord messages are limited to 2000 characters. Format code as Markdown code \
blocks tagged with their language.";


pub async fn build_agent(documents_dir: &Path) -> Result<Agent> {
    let client = OpenAI::from_env()?;
    let embedding_model = client.embedding(openai::TEXT_EMBEDDING_3_SMALL, None);


    let documents = load_markdown(documents_dir)?;
    let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
        .documents(documents)?
        .build()
        .await?;


    let index = InMemoryVectorStore::from_documents(embeddings).index(embedding_model);


    Ok(AgentBuilder::new(client.completion(openai::GPT_5_5))
        .preamble(PREAMBLE)
        .dynamic_context(2, index)
        .build())
}


fn load_markdown(dir: &Path) -> Result<Vec<String>> {
    let mut documents = Vec::new();
    for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "md") {
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            documents.push(text);
        }
    }
    Ok(documents)
}