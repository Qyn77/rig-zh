use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};
use serde::Deserialize;
use serde_json::json;


#[derive(Deserialize)]
struct AddArgs {
    a: i32,
    b: i32,
}


#[derive(Debug, thiserror::Error)]
#[error("math error")]
struct MathError;


struct Adder;


impl Tool for Adder {
    const NAME: &'static str = "add";
    type Error = MathError;
    type Args = AddArgs;
    type Output = i32;


    fn description(&self) -> String {
        "Add two integers together".to_string()
    }


    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": { "a": { "type": "number" }, "b": { "type": "number" } },
            "required": ["a", "b"]
        })
    }


    async fn call(&self, _ctx: &mut ToolContext, args: Self::Args) -> Result<i32, MathError> {
        Ok(args.a + args.b)
    }
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5))
        .preamble("Use the add tool for arithmetic.")
        .tool(Adder)
        .default_max_turns(2) // one turn to call the tool, one to answer
        .build();


    let answer = agent.prompt("What is 21 + 21?").await?.output();
    println!("{answer}");
    Ok(())
}
