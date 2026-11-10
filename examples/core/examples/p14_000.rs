use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Requires the OPENAI_API_KEY environment variable.
    let model = OpenAI::from_env()?.embedding(openai::TEXT_EMBEDDING_3_SMALL, None);


    let embeddings = EmbeddingsBuilder::new(model)
        .document("Some text")?
        .document("More text")?
        .build()
        .await?;


    println!("Generated {} embeddings", embeddings.len());
    Ok(())
}
