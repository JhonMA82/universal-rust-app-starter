#![forbid(unsafe_code)]
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode, header},
    routing::{get, post},
};
use serde_json::{Value, json};
use starter_core::{Application, Request, Session};
use std::sync::{Arc, Mutex};
use subtle::ConstantTimeEq;

struct Adapter<S> {
    app: Arc<Mutex<Application<S>>>,
    session: Session,
    authorization: String,
}
type Reply = Result<Json<Value>, (StatusCode, Json<Value>)>;
fn error(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (status, Json(json!({"code":code,"message":message})))
}
fn unavailable() -> (StatusCode, Json<Value>) {
    error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal",
        "application unavailable",
    )
}
impl<S> Adapter<S> {
    fn authorize(&self, headers: &HeaderMap) -> Result<(), (StatusCode, Json<Value>)> {
        if headers
            .get(header::AUTHORIZATION)
            .is_some_and(|v| bool::from(v.as_bytes().ct_eq(self.authorization.as_bytes())))
        {
            Ok(())
        } else {
            Err(error(
                StatusCode::UNAUTHORIZED,
                "forbidden",
                "invalid authorization",
            ))
        }
    }
}
pub fn router<S: Send + 'static>(
    app: Arc<Mutex<Application<S>>>,
    session: Session,
    bearer: &str,
) -> Result<Router, &'static str> {
    if !(32..=128).contains(&bearer.len())
        || !bearer
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
    {
        return Err("bearer token requires 32–128 URL-safe characters");
    }
    Ok(Router::new()
        .route("/operations", get(discover::<S>))
        .route("/invoke", post(invoke::<S>))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024 + 1024))
        .with_state(Arc::new(Adapter {
            app,
            session,
            authorization: format!("Bearer {bearer}"),
        })))
}
async fn discover<S: Send + 'static>(State(s): State<Arc<Adapter<S>>>, h: HeaderMap) -> Reply {
    s.authorize(&h)?;
    let app = s.app.lock().map_err(|_| unavailable())?;
    Ok(Json(
        json!({"session_id":s.session.id,"revision":app.revision(),"operations":app.discover(&s.session,true).collect::<Vec<_>>()}),
    ))
}
async fn invoke<S: Send + 'static>(
    State(s): State<Arc<Adapter<S>>>,
    h: HeaderMap,
    Json(req): Json<Request>,
) -> Reply {
    s.authorize(&h)?;
    let mut app = s.app.lock().map_err(|_| unavailable())?;
    app.invoke(&s.session, true, req)
        .map(|v| Json(json!(v)))
        .map_err(|e| {
            let status = match e.code {
                "forbidden" => StatusCode::FORBIDDEN,
                "unknown_operation" => StatusCode::NOT_FOUND,
                "conflict" | "request_id_reused" => StatusCode::CONFLICT,
                "internal" => return unavailable(),
                _ => StatusCode::UNPROCESSABLE_ENTITY,
            };
            error(status, e.code, &e.message)
        })
}
