use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // All diagnostics on stderr. stdout is exclusively MCP JSON-RPC.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
    mcp_tools::EchoServer::new()
        .serve(stdio())
        .await?
        .waiting()
        .await?;
    Ok(())
}
