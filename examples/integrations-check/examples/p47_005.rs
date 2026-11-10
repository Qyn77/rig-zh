use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, ErrorData, ServerCapabilities, ServerConfig},
    schemars, tool, tool_handler, tool_router,
};


#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SumRequest {
    a: i32,
    b: i32,
}


#[derive(Clone)]
struct Calculator {
    tool_router: ToolRouter<Calculator>,
}


#[tool_router]
impl Calculator {
    fn new() -> Self {
        Self { tool_router: Self::tool_router() }
    }


    #[tool(description = "Calculate the sum of two numbers")]
    fn sum(&self, Parameters(SumRequest { a, b }): Parameters<SumRequest>) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![ContentBlock::text((a + b).to_string())]))
    }
}


#[tool_handler(router = self.tool_router)]
impl ServerHandler for Calculator {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rmcp::ServiceExt;
    use rig::tool::{DynamicTool, ToolContext};
    use rig::tool::rmcp::McpConnection;
    use rig::tool::RegisteredTool;
    let (c2s, sfc) = tokio::io::duplex(8192);
    let (s2c, cfs) = tokio::io::duplex(8192);
    let server_task = tokio::spawn(async move {
        let running = Calculator::new().serve((sfc, s2c)).await.unwrap();
        running.waiting().await.unwrap();
    });
    let client = rmcp::model::ClientConfig::default().serve((cfs, c2s)).await?;
    let connection = McpConnection::new(client);
    let tools = connection.tools().await?;
    assert_eq!(tools.len(), 1);
    let sum = DynamicTool::try_from(tools.into_iter().next().unwrap())?;
    let result = RegisteredTool::from_dynamic(sum)
        .execute(r#"{"a":2,"b":5}"#.to_owned(), &mut ToolContext::new())
        .await.into_result()?;
    assert_eq!(result.render(), "7");
    connection.close().await;
    server_task.await?;
    println!("MCP offline transport: 2 + 5 = {}", result.render());
    Ok(())
}
