use rig::extractor::ExtractorBuilder;
use rig::providers::openai::{self, OpenAI};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct Person {
    name: Option<String>,
    age: Option<u8>,
    profession: Option<String>,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let model = OpenAI::from_env()?.completion(openai::GPT_5_5);
    let extractor = ExtractorBuilder::<Person>::new(model).build();


    let person = extractor
        .extract("John Doe is a 30 year old doctor.")
        .await?
        .output;


    println!("{person:?}");
    Ok(())
}
