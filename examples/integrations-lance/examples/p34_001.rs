use rig::lancedb::{LanceDbVectorIndex, SearchParams};
use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};


async fn build_agent(db_uri: &str) -> anyhow::Result<Agent> {
    let openai_client = OpenAI::from_env()?;
    let embedding_model = openai_client
        .embedding(openai::TEXT_EMBEDDING_3_SMALL, None)
        .erase();


    // e.g. "s3://rig-montreal-lancedb", "/tmp", or "/mnt/efs"
    let db = lancedb::connect(db_uri).execute().await?;
    let table = db.open_table("montreal_open_data").execute().await?;


    // The query embedding model must match the one the loader used.
    let index =
        LanceDbVectorIndex::new(table, embedding_model, "id", SearchParams::default()).await?;


    let agent = AgentBuilder::new(openai_client.completion(openai::GPT_5_5))
        .preamble("You answer questions about the City of Montreal's open data catalog.")
        .dynamic_context(3, index)
        .build();
    Ok(agent)
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_uri = std::env::var("LANCEDB_URI")?;
    let agent = build_agent(&db_uri).await?;
    println!("{}", agent.prompt("Which Montreal dataset covers public parks?").await?.output());
    Ok(())
}
