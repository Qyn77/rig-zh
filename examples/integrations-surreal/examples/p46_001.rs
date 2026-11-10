use rig::prelude::*;
use serde::{Deserialize, Serialize};
use rig::providers::openai::{self, OpenAI};
use rig::surrealdb::{Mem, SurrealVectorStore};
use rig::vector_store::InsertDocuments;
use surrealdb::Surreal;


#[derive(Embed, Serialize, Deserialize, Clone, Debug, Default)]
struct WordDefinition {
    word: String,
    #[embed]
    definition: String,
}


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let model = OpenAI::from_env()?.embedding(openai::TEXT_EMBEDDING_3_SMALL, None);


    // In-memory instance; use `Surreal::new::<Ws>("localhost:8000")` for a server.
    let surreal = Surreal::new::<Mem>(()).await?;
    surreal.use_ns("example").use_db("example").await?;


    let words = vec![
        WordDefinition { word: "flurbo".into(), definition: "A fictional currency from Rick and Morty.".into() },
        WordDefinition { word: "glarb-glarb".into(), definition: "A creature from the marshlands of Glibbo.".into() },
    ];
    let documents = EmbeddingsBuilder::new(model.clone())
        .documents(words)?
        .build()
        .await?;


    let vector_store = SurrealVectorStore::with_defaults(model, surreal);
    vector_store.insert_documents(documents).await?;


    let req = VectorSearchRequest::builder()
        .query("weird alien creature")
        .samples(2)
        .build();


    for result in vector_store.top_n::<WordDefinition>(req).await? {
        println!("{:.3} {}", result.score, result.document.word);
    }


    Ok(())
}