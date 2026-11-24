#![forbid(unsafe_code)]
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ServerCapabilities, ServerConfig},
    tool, tool_handler, tool_router,
};
use serde_json::{Value, json};
use starter_core::{Application, Error, Request, Response, Session};
use std::sync::{Arc, Mutex};

trait Service: Send + Sync {
    fn capabilities(&self) -> Result<Value, Error>;
    fn execute(&self, request: Request) -> Result<Response, Error>;
}
struct Adapter<S> {
    app: Arc<Mutex<Application<S>>>,
    session: Session,
}
impl<S: Clone + Send + 'static> Service for Adapter<S> {
    fn capabilities(&self) -> Result<Value, Error> {
        let app = self
            .app
            .lock()
            .map_err(|_| Error::new("internal", "application unavailable"))?;
        let operations: Vec<_> = app.discover(&self.session, true).collect();
        Ok(json!({"session_id":self.session.id,"revision":app.revision(),"operations":operations}))
    }
    fn execute(&self, request: Request) -> Result<Response, Error> {
        self.app
            .lock()
            .map_err(|_| Error::new("internal", "application unavailable"))?
            .invoke(&self.session, true, request)
    }
}
#[derive(Clone)]
pub struct Server {
    tool_router: ToolRouter<Self>,
    adapter: Arc<dyn Service>,
}
#[derive(serde::Deserialize, schemars::JsonSchema)]
struct Batch {
    requests: Vec<Request>,
}
#[tool_router]
impl Server {
    pub fn new<S: Clone + Send + 'static>(
        app: Arc<Mutex<Application<S>>>,
        session: Session,
    ) -> Self {
        Self {
            tool_router: Self::tool_router(),
            adapter: Arc::new(Adapter { app, session }),
        }
    }
    #[tool(description = "Discover the application's public operations, schemas and session")]
    fn capabilities(&self) -> CallToolResult {
        match self.adapter.capabilities() {
            Ok(v) => CallToolResult::structured(v),
            Err(e) => CallToolResult::structured_error(json!(e)),
        }
    }
    #[tool(description = "Invoke a discovered operation")]
    fn execute(&self, Parameters(request): Parameters<Request>) -> CallToolResult {
        match self.adapter.execute(request) {
            Ok(v) => CallToolResult::structured(json!(v)),
            Err(e) => CallToolResult::structured_error(json!(e)),
        }
    }
    #[tool(description = "Execute up to 32 requests sequentially, committing each independently")]
    fn execute_batch(&self, Parameters(batch): Parameters<Batch>) -> CallToolResult {
        if !(1..=32).contains(&batch.requests.len()) {
            return CallToolResult::structured_error(json!(Error::new(
                "invalid_input",
                "batch requires 1–32 requests"
            )));
        }
        let results: Vec<_> = batch
            .requests
            .into_iter()
            .map(|req| match self.adapter.execute(req) {
                Ok(v) => json!({"ok":v}),
                Err(e) => json!({"error":e}),
            })
            .collect();
        CallToolResult::structured(json!({"atomic":false,"items":results}))
    }
}
#[tool_handler(router = self.tool_router)]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
    }
}
pub fn stdio(server: Server) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            server
                .serve(rmcp::transport::stdio())
                .await?
                .waiting()
                .await?;
            Ok(())
        })
}
