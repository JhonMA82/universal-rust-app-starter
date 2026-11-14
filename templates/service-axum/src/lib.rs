use axum::{Json, Router, routing::get};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Health {
    pub status: &'static str,
}

async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}

pub fn router() -> Router {
    Router::new().route("/health", get(health))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_succeeds() {
        let response = router()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
