use anyhow::{Context, Result};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    response::Html,
    routing::{get, post},
};
use giong_core::Application;
use tokio::net::TcpListener;

mod webhook;

pub async fn serve(application: &Application) -> Result<()> {
    let (webhook_state, webhook_callback) = webhook::setup(application);

    let router = Router::new()
        .route("/", get(|| async { Html(include_str!("index.html")) }))
        .route("/webhooks/github", post(webhook::on_github_event))
        .layer(DefaultBodyLimit::max(25 * 1024 * 1024))
        .with_state(webhook_state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    eprintln!("HTTP server listening on {}", listener.local_addr()?);

    tokio::try_join!(
        async {
            axum::serve(listener, router)
                .await
                .context("HTTP server failed")
        },
        webhook_callback
    )?;

    Ok(())
}
