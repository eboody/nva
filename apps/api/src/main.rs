use std::net::SocketAddr;

use anyhow::Context;
use pet_resort_api::{
    http,
    observability::{ObservabilityConfig, ObservabilityRuntime},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let observability = ObservabilityRuntime::new(
        ObservabilityConfig::from_process_env().context("invalid observability configuration")?,
    );
    let _tracing_guard = observability
        .install_tracing()
        .context("failed to initialize structured tracing")?;
    let state = http::VaccineDocumentState::default().with_observability(observability);

    let addr: SocketAddr = std::env::var("PET_RESORT_API_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3001".to_owned())
        .parse()
        .context("PET_RESORT_API_ADDR must be a socket address")?;

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind pet-resort API at {addr}"))?;

    tracing::info!(%addr, "pet-resort API listening");
    axum::serve(listener, http::router_with_state(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("pet-resort API server failed")
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
