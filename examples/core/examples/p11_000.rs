use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};
use rig::tool::{ToolContext, ToolExecutionError};
use serde::Deserialize;
use serde_json::json;


#[derive(Deserialize)]
struct AddArgs {
    x: i64,
    y: i64,
}


// A tool implemented by hand against the `Tool` trait.
struct Add;


impl Tool for Add {
    const NAME: &'static str = "add";
    type Args = AddArgs;
    type Output = i64;
    type Error = ToolExecutionError;


    fn description(&self) -> String {
        "Add x and y together".to_string()
    }


    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "x": { "type": "integer", "description": "The first number" },
                "y": { "type": "integer", "description": "The second number" }
            },
            "required": ["x", "y"]
        })
    }


    async fn call(&self, _ctx: &mut ToolContext, args: AddArgs) -> Result<i64, ToolExecutionError> {
        Ok(args.x + args.y)
    }
}


// A tool generated from a function. The macro creates a unit struct named
// after the function in PascalCase (`subtract` -> `Subtract`).
#[rig::rig_tool(description = "Subtract y from x")]
fn subtract(x: i64, y: i64) -> Result<i64, ToolExecutionError> {
    Ok(x - y)
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let calculator = AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5))
        .preamble("You are a calculator. Use the provided tools to answer arithmetic questions.")
        .tool(Add)
        .tool(Subtract)
        .build();


    let answer = calculator
        .prompt("What is (5 - 2) + 10?")
        .max_turns(4)
        .await?
        .output();
    println!("{answer}");


    Ok(())
}
