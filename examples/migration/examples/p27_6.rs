#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rig::catalog::Catalog;
    // Persist the reference text, not the removed ProviderRef type.
    let reference = "deepseek/deepseek-v4-flash";
    let stored = serde_json::to_string(reference)?;
    let restored: String = serde_json::from_str(&stored)?;
    let model = Catalog::builtin().connect(&restored)?;
    let _agent = rig::AgentBuilder::new(model).build();
    assert_eq!(restored, reference);
    Ok(())
}
