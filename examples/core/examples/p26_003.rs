use anyhow::Result;
use rig::extractor::ExtractorBuilder;
use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize, JsonSchema)]
enum Sentiment {
    Positive,
    Negative,
    Neutral,
}


#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct SentimentClassification {
    sentiment: Sentiment,
    confidence: f32,
}


#[tokio::main]
async fn main() -> Result<()> {
    let model = OpenAI::from_env()?.completion(openai::GPT_5_5);


    let classifier = ExtractorBuilder::<SentimentClassification>::new(model)
        .append_preamble("Classify the sentiment of the given text.")
        .build();


    let text = "I absolutely loved the new restaurant. The food was amazing!";
    let result = classifier.extract(text).await?.output;


    println!("Sentiment: {:?} ({:.2})", result.sentiment, result.confidence);
    Ok(())
}
