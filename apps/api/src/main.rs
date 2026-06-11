use std::net::SocketAddr;

use anyhow::Context;
use pet_resort_api::http;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let addr: SocketAddr = std::env::var("PET_RESORT_API_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3001".to_owned())
        .parse()
        .context("PET_RESORT_API_ADDR must be a socket address")?;

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind pet-resort API at {addr}"))?;

    tracing::info!(%addr, "pet-resort API listening");
    axum::serve(listener, http::router())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("pet-resort API server failed")
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        tracing_subscriber::EnvFilter::new("pet_resort_api=info,tower_http=info")
    });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()
        .init();
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
