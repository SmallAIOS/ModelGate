use anyhow::Result;

/// MCP transport type.
#[derive(Debug, Clone, PartialEq)]
pub enum Transport {
    Stdio,
    Sse { port: u16 },
    Http { port: u16 },
}

/// MCP server configuration.
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    pub transport: Transport,
    pub workspace_root: Option<std::path::PathBuf>,
}

/// Start the MCP server.
///
/// This is a placeholder — actual MCP integration requires the rmcp SDK
/// which will be added once the crate is available on crates.io with
/// stable API.
pub async fn serve(config: McpServerConfig) -> Result<()> {
    tracing::info!("starting MCP server with {:?} transport", config.transport);

    match config.transport {
        Transport::Stdio => {
            tracing::info!("MCP stdio transport ready");
            // Future: rmcp stdio transport integration
            // let service = SmctlServer::new(workspace).serve(stdio()).await?;
            // service.waiting().await?;
            tokio::signal::ctrl_c().await?;
        }
        Transport::Sse { port } => {
            tracing::info!("MCP SSE transport on port {port}");
            tokio::signal::ctrl_c().await?;
        }
        Transport::Http { port } => {
            tracing::info!("MCP HTTP transport on port {port}");
            tokio::signal::ctrl_c().await?;
        }
    }

    Ok(())
}
