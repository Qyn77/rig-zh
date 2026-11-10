use rig::neo4j::{Neo4jClient, vector_index::IndexConfig};
use rig::prelude::*;
use serde::{Deserialize, Serialize};
use rig::providers::openai::{self, OpenAI};
use rig::vector_store::InsertDocuments;


#[derive(Embed, Clone, Serialize, Deserialize, Debug)]
struct Word {
    id: String,
    #[embed]
    definition: String,
}


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let model = OpenAI::from_env()?.embedding(openai::TEXT_EMBEDDING_3_SMALL, None);


    let client = Neo4jClient::connect("neo4j://localhost:7687", "neo4j", "password").await?;


    // A vector index named "words" over the `embedding` property of `Word` nodes,
    // as wide as the model's vectors.
    client
        .create_vector_index(IndexConfig::new("words"), "Word", model.capabilities().ndims)
        .await?;


    // Reuse the model that produced (or will produce) the stored embeddings.
    let index = client.get_index(model.clone(), "words").await?;


    let embeddings = EmbeddingsBuilder::new(model)
        .document(Word { id: "doc0".into(), definition: "A flurbo is a green alien.".into() })?
        .document(Word { id: "doc1".into(), definition: "A glarb-glarb is an ancient farming tool.".into() })?
        .build()
        .await?;
    index.insert_documents(embeddings).await?;


    let req = VectorSearchRequest::builder()
        .query("What is a glarb-glarb?")
        .samples(1)
        .build();


    for result in index.top_n::<Word>(req).await? {
        println!("{:.3} {} {}", result.score, result.id, result.document.definition);
    }


    Ok(())
}