use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let addr = std::env::var("MCP_BIND").unwrap_or_else(|_| "127.0.0.1:8000".into());
    let service = StreamableHttpService::new(
        || Ok(mcp_tools::EchoServer::new()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default().with_json_response(true),
    );
    let app = axum::Router::new().nest_service("/mcp", service);
    let tcp = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "MCP Streamable HTTP server");
    axum::serve(tcp, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
