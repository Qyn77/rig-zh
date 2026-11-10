#[tokio::main]
async fn main() -> anyhow::Result<()> {
use rig::fastembed::{Fastembed, FastembedModel};
use rig::prelude::*;


let model_id = FastembedModel::AllMiniLML6V2Q;
let embedder = Fastembed::load(&model_id)?.embedding(&model_id, None)?;


// Use it like any embedding model: EmbeddingsBuilder, vector stores, ...
let index = InMemoryVectorStore::<String>::default().index(embedder);
Ok(())
}
