use rig::extractor::ExtractorBuilder;
use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct Person {
    name: String,
    /// The person's age in years, if stated.
    age: Option<u8>,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let model = OpenAI::from_env()?.completion(openai::GPT_5_5);
    let extractor = ExtractorBuilder::<Person>::new(model).build();


    let person = extractor
        .extract("Marie Curie was 66 years old when she passed away.")
        .await?
        .output;


    println!("{person:?}");
    Ok(())
}
