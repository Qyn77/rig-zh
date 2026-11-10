#[tokio::main]
async fn main() -> anyhow::Result<()> {
use rig::prelude::*;
use rig::providers::openai::{self, OpenAI};
use rig::tool::rmcp::McpClientHandler;
use rig::tool::server::ToolServer;
use rmcp::model::{ClientCapabilities, ClientConfig, Implementation};


let tool_server = ToolServer::new().run();


let client_config = ClientConfig::new(
    ClientCapabilities::default(),
    Implementation::new("my-agent", "0.1.0"),
);
let handler = McpClientHandler::new(client_config, tool_server.clone());


// Connects, fetches the initial tool list, and keeps it in sync.
let transport = rmcp::transport::StreamableHttpClientTransport::from_uri("http://localhost:8080");
let mcp = handler.connect(transport).await?;


let agent = AgentBuilder::new(OpenAI::from_env()?.completion(openai::GPT_5_5))
    .preamble("You can use the tools of an MCP server.")
    .tool_server_handle(tool_server)
    .build();


let answer = agent.prompt("What is 2 + 5?").max_turns(2).await?.output();
Ok(())
}
