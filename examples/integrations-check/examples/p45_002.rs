use qdrant_client::{
    Qdrant,
    qdrant::{CreateCollectionBuilder, Distance, QueryPointsBuilder, VectorParamsBuilder},
};
use rig::prelude::*;
use serde::{Deserialize, Serialize};
use rig::providers::openai::{self, OpenAI};
use rig::qdrant::QdrantVectorStore;
use rig::vector_store::InsertDocuments;


const COLLECTION_NAME: &str = "rig-collection";


#[derive(Embed, Serialize, Deserialize, Debug)]
struct Word {
    id: String,
    #[embed]
    definition: String,
}


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = Qdrant::from_url("http://localhost:6334").build()?;


    if !client.collection_exists(COLLECTION_NAME).await? {
        client
            .create_collection(
                CreateCollectionBuilder::new(COLLECTION_NAME)
                    .vectors_config(VectorParamsBuilder::new(1536, Distance::Cosine)),
            )
            .await?;
    }


    let model = OpenAI::from_env()?.embedding(openai::TEXT_EMBEDDING_3_SMALL, None);


    let documents = EmbeddingsBuilder::new(model.clone())
        .document(Word {
            id: "doc0".to_string(),
            definition: "A flurbo is a green alien that lives on cold planets.".to_string(),
        })?
        .document(Word {
            id: "doc1".to_string(),
            definition: "A linglingdong is a term used to describe humans.".to_string(),
        })?
        .build()
        .await?;


    // The store queries the collection named in these parameters.
    let query_params = QueryPointsBuilder::new(COLLECTION_NAME).with_payload(true);
    let vector_store = QdrantVectorStore::new(client, model, query_params.build());


    vector_store.insert_documents(documents).await?;


    let req = VectorSearchRequest::builder()
        .query("What is a linglingdong?")
        .samples(1)
        .build();


    for result in vector_store.top_n::<Word>(req).await? {
        println!("{:.3} {} {}", result.score, result.id, result.document.definition);
    }


    Ok(())
}