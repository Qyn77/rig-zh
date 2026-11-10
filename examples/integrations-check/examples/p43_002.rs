use mongodb::{Client as MongoClient, Collection, bson::{self, doc}};
use rig::mongodb::{MongoDbVectorIndex, SearchParams};
use rig::prelude::*;
use serde::{Deserialize, Serialize};
use rig::providers::openai::{self, OpenAI};


#[derive(Embed, Clone, Serialize, Deserialize, Debug)]
struct Word {
    id: String,
    #[embed]
    definition: String,
}


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let model = OpenAI::from_env()?.embedding(openai::TEXT_EMBEDDING_3_SMALL, None);


    let mongo = MongoClient::with_uri_str(std::env::var("MONGODB_CONNECTION_STRING")?).await?;
    let collection: Collection<bson::Document> =
        mongo.database("knowledgebase").collection("context");


    // Embed the documents and store each with its vector in `embedding`.
    let words = vec![
        Word { id: "doc0".into(), definition: "A flurbo is a green alien.".into() },
        Word { id: "doc1".into(), definition: "A glarb-glarb is an ancient farming tool.".into() },
    ];
    let embeddings = EmbeddingsBuilder::new(model.clone())
        .documents(words)?
        .build()
        .await?;
    let records = embeddings
        .iter()
        .map(|(word, embeddings)| {
            doc! {
                "id": word.id.clone(),
                "definition": word.definition.clone(),
                "embedding": embeddings.first().map(|e| e.vec.clone()),
            }
        })
        .collect::<Vec<_>>();
    collection.insert_many(records).await?;


    // "vector_index" is the Atlas vector search index on this collection.
    let index = MongoDbVectorIndex::new(collection, model, "vector_index", SearchParams::new()).await?;


    let req = VectorSearchRequest::builder()
        .query("What does glarb-glarb mean?")
        .samples(1)
        .build();


    for result in index.top_n::<Word>(req).await? {
        println!("{:.3} {}", result.score, result.document.definition);
    }


    Ok(())
}