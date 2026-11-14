use rmcp::{ClientServiceExt, model::ClientConfig, transport::StreamableHttpClientTransport};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = std::env::var("MCP_URL").unwrap_or_else(|_| "http://127.0.0.1:8000/mcp".into());
    let transport = StreamableHttpClientTransport::from_uri(url);
    let peer = ClientConfig::default().serve(transport).await?;
    let tools = peer.list_tools(Default::default()).await?;
    println!("{tools:#?}");
    peer.cancel().await?;
    Ok(())
}
