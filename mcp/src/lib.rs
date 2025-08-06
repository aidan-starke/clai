use rmcp::{
    ErrorData as McpError, ServiceExt, handler::server::router::tool::ToolRouter, model::*, tool,
    tool_handler, tool_router,
};
use std::sync::Arc;
use tokio::sync::Mutex;

fn stdio() -> (tokio::io::Stdin, tokio::io::Stdout) {
    (tokio::io::stdin(), tokio::io::stdout())
}

#[derive(Clone)]
pub struct Counter {
    counter: Arc<Mutex<i32>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Counter {
    fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Increment the counter by 1")]
    async fn increment(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }

    #[tool(description = "Get the current counter value")]
    async fn get(&self) -> Result<CallToolResult, McpError> {
        let counter = self.counter.lock().await;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }
}

#[tool_handler]
impl rmcp::ServerHandler for Counter {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A simple calculator".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let service = Counter::new().serve(stdio()).await.inspect_err(|e| {
        println!("Error starting server: {}", e);
    })?;
    service.waiting().await?;

    Ok(())
}
