use rig::prelude::*;
use serde_json::json;

#[tokio::test]
async fn documented_p21_002() -> anyhow::Result<()> {
use rig::test_utils::MockCompletionModel;


let model = MockCompletionModel::text("Hello from a mocked model!");
let agent = AgentBuilder::new(model).preamble("You are friendly.").build();


let reply = agent.prompt("Hi").await?;


assert_eq!(reply.output(), "Hello from a mocked model!");
Ok(())
}

#[tokio::test]
async fn documented_p21_003() -> anyhow::Result<()> {
use rig::test_utils::{MockAddTool, MockCompletionModel, MockTurn};


let model = MockCompletionModel::from_turns([
    // Turn 1: the model asks to call the `add` tool.
    MockTurn::tool_call("call_1", "add", json!({ "x": 2, "y": 3 })),
    // Turn 2: after seeing the tool result, it answers.
    MockTurn::text("2 + 3 = 5"),
]);
let probe = model.clone();


// Attach your real tool here; `MockAddTool` is a ready-made one.
let agent = AgentBuilder::new(model).tool(MockAddTool).build();
let answer = agent.prompt("What is 2 + 3?").max_turns(3).await?;


assert_eq!(answer.output(), "2 + 3 = 5");
assert_eq!(probe.request_count(), 2); // the loop called the model twice
Ok(())
}

#[tokio::test]
async fn documented_p21_004() -> anyhow::Result<()> {
use rig::test_utils::MockCompletionModel;


let model = MockCompletionModel::text("ok");
let probe = model.clone();


let agent = AgentBuilder::new(model).preamble("You are a pirate.").build();
agent.prompt("Ahoy").await?;


let sent = probe.requests();
assert_eq!(sent.len(), 1);
assert_eq!(sent[0].system_instructions(), Some("You are a pirate."));
Ok(())
}

#[tokio::test]
async fn documented_p21_005() -> anyhow::Result<()> {
use rig::test_utils::{sent_documents, MockCompletionModel, MockEmbeddings};


let embeddings = EmbeddingsBuilder::new(MockEmbeddings::model())
    .documents(["a green alien", "an ancient farming tool"])?
    .build()
    .await?;
let index = InMemoryVectorStore::from_documents(embeddings).index(MockEmbeddings::model());


let model = MockCompletionModel::text("ok");
let probe = model.clone();
let agent = AgentBuilder::new(model).dynamic_context(2, index).build();
agent.prompt("What is a flurbo?").await?;


// Both documents were retrieved and sent as context.
assert_eq!(sent_documents(&probe.requests()[0]).len(), 2);
Ok(())
}

mod weather {
use rig::prelude::*;
use rig::tool::ToolExecutionError;


/// Look up the current temperature for a city.
#[rig::rig_tool(params(city = "City name, e.g. `Paris`"))]
async fn get_weather(
    city: String,
    /// Temperature unit: `celsius` (default) or `fahrenheit`.
    unit: Option<String>,
) -> Result<String, ToolExecutionError> {
    let unit = unit.unwrap_or_else(|| "celsius".to_string());
    if city.is_empty() {
        return Err(ToolExecutionError::invalid_args("`city` must not be empty"));
    }
    Ok(format!("It is 21 degrees {unit} in {city}."))
}
#[tokio::test]
async fn real_weather_tool() -> anyhow::Result<()> {
let mut ctx=ToolContext::new();
let output=GetWeather.call(&mut ctx, serde_json::from_value(serde_json::json!({"city":"Paris"}))?).await.unwrap();
assert!(output.contains("21 degrees celsius"));
assert!(GetWeather.call(&mut ctx, serde_json::from_value(serde_json::json!({"city":""}))?).await.is_err());
Ok(())
}
}
