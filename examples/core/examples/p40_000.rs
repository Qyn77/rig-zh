use rig::prelude::*;
use serde::{Deserialize, Serialize};
use rig::providers::openai::{self, OpenAI};


#[derive(Embed, Clone, Serialize, Deserialize, Debug)]
struct WordDefinition {
    id: String,
    #[embed]
    definition: String,
}


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let model = OpenAI::from_env()?.embedding(openai::TEXT_EMBEDDING_3_SMALL, None);


    let documents = vec![
        WordDefinition { id: "doc0".into(), definition: "A flurbo is a green alien.".into() },
        WordDefinition { id: "doc1".into(), definition: "A glarb-glarb is an ancient farming tool.".into() },
    ];


    // Embed each document's `#[embed]` field.
    let embeddings = EmbeddingsBuilder::new(model.clone())
        .documents(documents)?
        .build()
        .await?;


    // Store the documents and build an index that embeds queries with the same model.
    let index = InMemoryVectorStore::from_documents(embeddings).index(model);


    let req = VectorSearchRequest::builder()
        .query("What is a flurbo?")
        .samples(1)
        .build();


    for result in index.top_n::<WordDefinition>(req).await? {
        println!("{:.3} {}: {}", result.score, result.id, result.document.definition);
    }


    Ok(())
}
