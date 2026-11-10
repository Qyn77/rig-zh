#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rig::cassette::agent::AgentReplayExt;
    use rig::cassette::effect_log::EffectLogRecorder;
    use rig::providers::openai::{self, OpenAI};
    let recorder = EffectLogRecorder::new();
    let agent = rig::AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5))
        .owner("replay-example")
        .preamble("You are a helpful assistant.")
        .record_to(recorder.clone()).build();
    let response = agent.prompt("Name three Rust web frameworks.").await?;
    println!("{}", response.output());
    let log = agent.stamp(recorder.take());
    std::fs::write("frameworks.effects.json", serde_json::to_string_pretty(&log)?)?;
    Ok(())
}
