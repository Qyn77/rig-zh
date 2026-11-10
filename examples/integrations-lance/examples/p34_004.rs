#[tokio::main]
async fn main() -> anyhow::Result<()> {
let db = lancedb::connect("/tmp").execute().await?;
Ok(())
}
