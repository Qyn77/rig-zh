#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rig::handlers::Handlers;
    use rig::cassette::agent::replay::register_all;
    use rig::cassette::effect_log::{EffectLog, EffectLogReplayer};
    use rig::effect::EffectFamily;
    use rig::tool::{RegisteredTool, server::ToolServer};
    let log: EffectLog = serde_json::from_str(&std::fs::read_to_string("frameworks.effects.json")?)?;
    let handlers = Handlers::default();
    register_all(&log, &handlers)?;
    let tools = ToolServer::new().run();
    let mut seen = std::collections::BTreeSet::new();
    for record in log.iter().filter(|r| r.kind.family() == EffectFamily::Tool) {
        if seen.insert(record.key.to_string()) {
            let replayer = EffectLogReplayer::for_key(&log, &record.key)?;
            tools.add_registered_tool(RegisteredTool::from_handler(replayer)?);
        }
    }
    let model_key = log.iter().find(|r| r.kind.family() == EffectFamily::Completion)
        .ok_or_else(|| anyhow::anyhow!("no completion record"))?.key.clone();
    let agent = rig::AgentBuilder::over_handlers(handlers, "replay-example", model_key)
        .preamble("You are a helpful assistant.")
        .tool_server_handle(tools).build();
    println!("{}", agent.prompt("Name three Rust web frameworks.").max_turns(4).await?.output());
    Ok(())
}
