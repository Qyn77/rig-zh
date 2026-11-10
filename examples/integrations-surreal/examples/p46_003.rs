#[tokio::main]
async fn main() -> anyhow::Result<()> {
use rig::providers::openai::{self, OpenAI};
use rig::surrealdb::{Mem, SurrealDistanceFunction, SurrealVectorStore};
use surrealdb::Surreal;


let model = OpenAI::from_env()?.embedding(openai::TEXT_EMBEDDING_3_SMALL, None);
let surreal = Surreal::new::<Mem>(()).await?;
surreal.use_ns("example").use_db("example").await?;


let store = SurrealVectorStore::new(
    model,
    surreal,
    Some("word_definitions".into()),
    SurrealDistanceFunction::Cosine,
);
Ok(())
}
