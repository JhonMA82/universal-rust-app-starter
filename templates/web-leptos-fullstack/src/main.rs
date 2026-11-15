use axum::{Router, response::Html, routing::get};
use leptos::prelude::*;

async fn index() -> Html<String> {
    let html = view! {
        <main><h1>"{{project-name}}"</h1><p>"Rendered on the Rust server"</p></main>
    }
    .to_html();
    Html(format!(
        concat!(
            "<!doctype html><html lang=\"en\"><head>",
            "<meta charset=\"utf-8\"><title>{{project-name}}</title></head>",
            "<body>{html}</body></html>"
        ),
        html = html
    ))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router = Router::new().route("/", get(index));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
