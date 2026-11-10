#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rig::catalog::Catalog;
    use rig::providers::registry::ConnectOptions;
    let catalog = Catalog::builtin();
    let resolved = catalog.resolve("openai/gpt-5.5")?;
    let spec = resolved.spec;
    println!("{}: {:?} token context, tools: {}",
        spec.display_name, spec.context_window, spec.tools);
    if let Some(pricing) = &spec.pricing {
        println!("${} / ${} per million input/output tokens", pricing.input, pricing.output);
    }
    let model = catalog.connect_with("openai/gpt-5.5",
        ConnectOptions::new().api_key(std::env::var("OPENAI_API_KEY")?))?;
    let _agent = rig::AgentBuilder::new(model).build();
    Ok(())
}
