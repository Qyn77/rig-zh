#[tokio::main]
async fn main() -> anyhow::Result<()> {
// Local, on-disk store.
let db = lancedb::connect("data/lancedb-store").execute().await?;


// Object storage on S3 (see LanceDB's storage guide for credentials).
let db = lancedb::connect("s3://my-lancedb-bucket").execute().await?;
Ok(())
}
