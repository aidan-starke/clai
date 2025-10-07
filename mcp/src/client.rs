use common::error::ClaiError;
use rmcp::{
    RoleClient, ServiceExt,
    model::{
        CallToolRequestParam, CallToolResult, ClientCapabilities, ClientInfo, Implementation,
        InitializeRequestParam,
    },
    service::RunningService,
    transport::StreamableHttpClientTransport,
};

pub struct Client {
    client: RunningService<RoleClient, InitializeRequestParam>,
}

impl Client {
    pub async fn new(server_url: &str) -> Self {
        let client_info = ClientInfo {
            protocol_version: Default::default(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "clai-client".to_string(),
                version: "0.0.1".to_string(),
            },
        };

        let client = client_info
            .serve(StreamableHttpClientTransport::from_uri(format!(
                "{}/{}",
                server_url, "mcp"
            )))
            .await
            .expect("Failed to initialize client");

        Self { client }
    }

    pub async fn find(&self, search: &str) -> Result<CallToolResult, ClaiError> {
        self.client
            .call_tool(CallToolRequestParam {
                name: "find".into(),
                arguments: serde_json::json!({ "search": search }).as_object().cloned(),
            })
            .await
            .map_err(ClaiError::from)
    }
}
