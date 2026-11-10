#[tokio::main]
async fn main() -> anyhow::Result<()> {
use rig::candle::{CandleModel, ModelData};
use rig::prelude::*;


let model = CandleModel::from_safetensors_async(ModelData {
    config: std::fs::read("./model/config.json")?,
    tokenizer: std::fs::read("./model/tokenizer.json")?,
    weights: std::fs::read("./model/model.safetensors")?,
})
.await?;


let agent = AgentBuilder::new(model.completion())
    .preamble("You are a concise assistant.")
    .build();
println!("{}", agent.prompt("Explain ownership briefly.").await?.output());
Ok(())
}
