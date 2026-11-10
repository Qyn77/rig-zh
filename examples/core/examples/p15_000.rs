use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};


#[derive(rig::Embed, serde::Serialize, Clone)]
struct WordDefinition {
    id: String,
    word: String,
    #[embed]
    definitions: Vec<String>,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = OpenAI::from_env()?;
    let embedding_model = client.embedding(openai::TEXT_EMBEDDING_3_SMALL, None);


    // Ingestion: embed the documents and load them into an in-memory store.
    let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
        .documents(vec![
            WordDefinition {
                id: "doc0".into(),
                word: "flurbo".into(),
                definitions: vec!["A green alien that lives on cold planets.".into()],
            },
            WordDefinition {
                id: "doc1".into(),
                word: "glarb-glarb".into(),
                definitions: vec!["An ancient farming tool from planet Jiro.".into()],
            },
        ])?
        .build()
        .await?;
    let index = InMemoryVectorStore::from_documents(embeddings).index(embedding_model);


    // Retrieval: the agent fetches the best match for every prompt.
    let agent = AgentBuilder::new(client.completion(openai::GPT_5_5))
        .preamble("You are a dictionary assistant. Use the definitions provided as context.")
        .dynamic_context(1, index)
        .build();


    let answer = agent.prompt("What does \"glarb-glarb\" mean?").await?.output();
    println!("{answer}");
    Ok(())
}
