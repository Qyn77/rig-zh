#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rig::catalog::Catalog;
    // Credentials are read from DEEPSEEK_API_KEY; constructing does not call the provider.
    let model = Catalog::builtin().connect("deepseek/deepseek-v4-flash")?;
    let _agent = rig::AgentBuilder::new(model).build();
    Ok(())
}
