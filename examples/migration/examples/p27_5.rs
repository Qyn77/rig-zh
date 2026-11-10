#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rig::catalog::Catalog;
    use rig::providers::registry::ConnectOptions;
    let model_name = "anthropic/claude-sonnet-5-5";
    let api_key = std::env::var("ANTHROPIC_API_KEY")?;
    let model = Catalog::builtin().connect_with(
        model_name, ConnectOptions::new().api_key(api_key),
    )?;
    let agent = rig::AgentBuilder::new(model)
        .preamble("You are a helpful assistant.").build();
    println!("{}", agent.prompt("Hello!").await?.output());
    Ok(())
}
