use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    tool, tool_handler, tool_router,
    transport::{
        StreamableHttpServerConfig, StreamableHttpService,
        streamable_http_server::session::local::LocalSessionManager,
    },
};
use std::{borrow::Cow, env, fs, io, path::Path};

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct FindArgs {
    search: String,
}

#[derive(Clone)]
pub struct Server {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Server {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    pub fn get_server() -> StreamableHttpService<Self> {
        StreamableHttpService::new(
            || Ok(Self::new()),
            LocalSessionManager::default().into(),
            StreamableHttpServerConfig {
                stateful_mode: true,
                sse_keep_alive: None,
            },
        )
    }

    #[tool(description = "Search for files in the current directory")]
    async fn find(
        &self,
        Parameters(FindArgs { search }): Parameters<FindArgs>,
    ) -> Result<CallToolResult, McpError> {
        println!("Searching for: {}", search);

        let current_dir = env::current_dir().map_err(|e| McpError {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: Cow::Owned(format!("Failed to get current directory: {}", e)),
            data: None,
        })?;

        let mut results = Vec::new();

        // Recursively search for files
        if let Err(e) = search_directory(&current_dir, &search, &mut results) {
            return Err(McpError {
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                message: Cow::Owned(format!("Error searching directory: {}", e)),
                data: None,
            });
        }

        if results.is_empty() {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "No files found matching '{}'",
                search
            ))]))
        } else {
            let result_text = format!(
                "Found {} file(s) matching '{}':\n{}",
                results.len(),
                search,
                results.join("\n")
            );
            Ok(CallToolResult::success(vec![Content::text(result_text)]))
        }
    }
}

#[tool_handler]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("File search server".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

fn search_directory(dir: &Path, search_term: &str, results: &mut Vec<String>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name
                    .to_lowercase()
                    .contains(&search_term.to_lowercase())
                {
                    if let Some(path_str) = path.to_str() {
                        results.push(path_str.to_string());
                    }
                }
            }
        } else if path.is_dir() {
            // Skip hidden directories and common build/cache directories
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if !dir_name.starts_with('.')
                    && dir_name != "target"
                    && dir_name != "node_modules"
                    && dir_name != "__pycache__"
                {
                    search_directory(&path, search_term, results)?;
                }
            }
        }
    }
    Ok(())
}
