use rig::prelude::*;
use serde_json::json;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
use rig::qdrant::QdrantFilter;
use rig::vector_store::request::SearchFilter;


let req = VectorSearchRequest::<QdrantFilter>::builder()
    .query("What is a linglingdong?")
    .samples(1)
    .filter(QdrantFilter::eq("id", json!("doc1")))
    .build();
Ok(())
}
