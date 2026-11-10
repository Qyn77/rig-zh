#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rig::handlers::Handlers;
    use rig::cassette::agent::replay::register_all;
    use rig::cassette::effect_log::EffectLog;
    use rig::effect::EffectFamily;
    let log: EffectLog = serde_json::from_str(&std::fs::read_to_string("frameworks.effects.json")?)?;
    let handlers = Handlers::default();
    register_all(&log, &handlers)?;
    let model_key = log.iter().find(|r| r.kind.family() == EffectFamily::Completion)
        .ok_or_else(|| anyhow::anyhow!("no completion record"))?.key.clone();
    let agent = rig::AgentBuilder::over_handlers(handlers, "replay-example", model_key)
        .preamble("You are a helpful assistant.").build();
    let response = agent.prompt("Name three Rust web frameworks.").await?;
    println!("{}", response.output());
    Ok(())
}
