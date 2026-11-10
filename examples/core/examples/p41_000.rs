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
        WordDefinition { id: "doc0".into(), definition: "First document content".into() },
        WordDefinition { id: "doc1".into(), definition: "Second document content".into() },
    ];


    let embeddings = EmbeddingsBuilder::new(model.clone())
        .documents(documents)?
        .build()
        .await?;


    // Build the store, then an index that embeds queries with the same model.
    let index = InMemoryVectorStore::from_documents(embeddings).index(model);


    let req = VectorSearchRequest::builder()
        .query("search query")
        .samples(5)
        .build();


    for result in index.top_n::<WordDefinition>(req).await? {
        println!("{:.3} {} {:?}", result.score, result.id, result.document);
    }


    Ok(())
}
