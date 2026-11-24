//! Opt-in local Streamable HTTP transport for a shared application.
use crate::Server;
use axum::{
    Router,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    path::Path,
    sync::Arc,
    time::Duration,
};
use subtle::ConstantTimeEq;
use tokio::sync::oneshot;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
#[derive(Debug, Deserialize, Serialize)]
pub struct Instance {
    pub address: SocketAddr,
    pub application: String,
    pub pid: u32,
    pub session_id: String,
}
impl Instance {
    pub fn endpoint(&self) -> String {
        format!("http://{}/mcp", self.address)
    }
}
pub struct Live {
    pub instance: Instance,
    _descriptor: tempfile::NamedTempFile,
    stop: Option<oneshot::Sender<()>>,
}
impl Drop for Live {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }
}

pub fn start(
    directory: &Path,
    application: &str,
    token: &str,
    server: impl FnOnce(String) -> Server,
) -> Result<Live> {
    if !(32..=128).contains(&token.len())
        || !token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
    {
        return Err("live MCP requires a 32–128 character URL-safe bearer token".into());
    }
    fs::create_dir_all(directory)?;
    let socket = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    socket.set_nonblocking(true)?;
    let address = socket.local_addr()?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let mut descriptor = tempfile::Builder::new()
        .prefix("starter-")
        .suffix(".json")
        .tempfile_in(directory)?;
    let session_id = descriptor
        .path()
        .file_stem()
        .and_then(|id| id.to_str())
        .ok_or("invalid discovery file")?
        .to_owned();
    let instance = Instance {
        address,
        application: application.into(),
        pid: std::process::id(),
        session_id,
    };
    serde_json::to_writer(descriptor.as_file_mut(), &instance)?;
    descriptor.as_file_mut().flush()?;
    let handler = server(instance.session_id.clone());
    let authorization: Arc<str> = format!("Bearer {token}").into();
    let (stop, stopped) = oneshot::channel();
    std::thread::Builder::new()
        .name("starter-live-mcp".into())
        .spawn(move || {
            runtime.block_on(async move {
                let service = StreamableHttpService::new(
                    move || Ok(handler.clone()),
                    Arc::new(LocalSessionManager::default()),
                    StreamableHttpServerConfig::default()
                        .with_allowed_hosts(["127.0.0.1"])
                        .with_json_response(true),
                );
                let app = Router::new()
                    .nest_service("/mcp", service)
                    .layer(middleware::from_fn_with_state(authorization, authorize));
                let listener = match tokio::net::TcpListener::from_std(socket) {
                    Ok(listener) => listener,
                    Err(error) => {
                        eprintln!("Live MCP listener failed: {error}");
                        return;
                    }
                };
                if let Err(error) = axum::serve(listener, app)
                    .with_graceful_shutdown(async move {
                        let _ = stopped.await;
                    })
                    .await
                {
                    eprintln!("Live MCP server failed: {error}");
                }
            });
        })?;
    Ok(Live {
        instance,
        _descriptor: descriptor,
        stop: Some(stop),
    })
}
async fn authorize(State(auth): State<Arc<str>>, request: Request, next: Next) -> Response {
    if request.headers().get(header::ORIGIN).is_some() {
        return StatusCode::FORBIDDEN.into_response();
    }
    if !request
        .headers()
        .get(header::AUTHORIZATION)
        .is_some_and(|value| bool::from(value.as_bytes().ct_eq(auth.as_bytes())))
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    next.run(request).await
}

/// Discovery files are hints. Authenticate and perform MCP initialization
/// against the selected endpoint before trusting the advertised identity.
pub fn discover(directory: &Path) -> Result<Vec<Instance>> {
    let mut instances = Vec::new();
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(instances),
        Err(error) => return Err(error.into()),
    };
    for entry in entries.flatten().take(128) {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.starts_with("starter-")
            || !name.ends_with(".json")
            || !entry.file_type().is_ok_and(|kind| kind.is_file())
        {
            continue;
        }
        let Ok(mut file) = fs::File::open(&path) else {
            continue;
        };
        let mut bytes = Vec::new();
        if Read::by_ref(&mut file)
            .take(4097)
            .read_to_end(&mut bytes)
            .is_err()
            || bytes.len() > 4096
        {
            continue;
        }
        let Ok(instance) = serde_json::from_slice::<Instance>(&bytes) else {
            continue;
        };
        if instance.address.ip().is_loopback()
            && path.file_stem().and_then(|id| id.to_str()) == Some(&instance.session_id)
            && TcpStream::connect_timeout(&instance.address, Duration::from_millis(100)).is_ok()
        {
            instances.push(instance);
        }
    }
    instances.sort_by(|a, b| a.session_id.cmp(&b.session_id));
    Ok(instances)
}
