mod flight_search_tool;

use flight_search_tool::FlightSearchTool;
use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5))
        .preamble("You are a travel assistant. Use the search_flights tool to find flights, then summarize the best options.")
        .tool(FlightSearchTool)
        .default_max_turns(3)
        .build();


    let response = agent
        .prompt("Find me flights from San Antonio (SAT) to Atlanta (ATL) next month.")
        .await?;


    println!("{}", response.output());
    Ok(())
}
