use rig::prelude::*;
use serde_json::json;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
use mongodb::bson::Bson;
use rig::mongodb::MongoDbSearchFilter;
use rig::vector_store::request::SearchFilter;


let req = VectorSearchRequest::<MongoDbSearchFilter>::builder()
    .query("What does glarb-glarb mean?")
    .samples(3)
    .filter(MongoDbSearchFilter::eq("id", Bson::from("doc1")))
    .build();
Ok(())
}
