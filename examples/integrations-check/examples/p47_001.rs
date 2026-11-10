use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};
use rig::tool::DynamicTool;
use rig::tool::rmcp::McpConnection;

// Connection owns the service; registered tools keep it alive.
async fn mcp_agent(connection: &McpConnection) -> anyhow::Result<Agent> {
    let tools = connection
        .tools()
        .await?
        .into_iter()
        .map(DynamicTool::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5))
        .preamble("You can use the tools of an MCP server.")
        .dynamic_tools(tools)
        .build())
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
use rmcp::ServiceExt;
use rmcp::model::{ClientCapabilities, ClientConfig, Implementation};
use rmcp::transport::StreamableHttpClientTransport;

let client_config = ClientConfig::new(
    ClientCapabilities::default(),
    Implementation::new("my-agent", "0.1.0"),
);
let transport = StreamableHttpClientTransport::from_uri("http://localhost:8080");
let connection = McpConnection::new(client_config.serve(transport).await?);
let agent = mcp_agent(&connection).await?;
let answer = agent.prompt("What is 2 + 5?").await?.output();
println!("{answer}");
connection.close().await;
Ok(())
}
