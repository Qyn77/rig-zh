#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rig::catalog::Catalog;
    let resolved = Catalog::builtin().resolve("anthropic/claude-haiku-4-5")?;
    println!("context window: {:?}", resolved.spec.context_window);
    Ok(())
}
