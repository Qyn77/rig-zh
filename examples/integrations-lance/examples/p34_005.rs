#[tokio::main]
async fn main() -> anyhow::Result<()> {
let db = lancedb::connect("/mnt/efs").execute().await?;
Ok(())
}
