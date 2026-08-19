use std::{future::Future, net::SocketAddr};

use anyhow::Context;
use pet_resort_api::{
    http,
    observability::{ObservabilityConfig, ObservabilityRuntime},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run().await
}

async fn run() -> anyhow::Result<()> {
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
    serve(listener, state, shutdown_signal()).await
}

async fn serve(
    listener: tokio::net::TcpListener,
    state: http::VaccineDocumentState,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    axum::serve(listener, http::router_with_state(state))
        .with_graceful_shutdown(shutdown)
        .await
        .context("pet-resort API server failed")
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_entrypoint_serves_until_graceful_shutdown() {
        if std::env::var("NVA_API_MAIN_TEST_CHILD").as_deref() == Ok("1") {
            let process_id = std::process::id().to_string();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(500));
                assert!(
                    std::process::Command::new("kill")
                        .args(["-INT", &process_id])
                        .status()
                        .unwrap()
                        .success()
                );
            });
            main().expect("the API binary should stop cleanly after SIGINT");
            return;
        }

        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "tests::binary_entrypoint_serves_until_graceful_shutdown",
                "--nocapture",
            ])
            .env("NVA_API_MAIN_TEST_CHILD", "1")
            .env("PET_RESORT_TELEMETRY_MODE", "local")
            .env("PET_RESORT_API_ADDR", "127.0.0.1:0")
            .status()
            .unwrap();
        assert!(status.success());
    }

    #[tokio::test]
    async fn startup_initializes_runtime_before_rejecting_an_invalid_address() {
        if std::env::var("NVA_API_STARTUP_TEST_CHILD").as_deref() == Ok("1") {
            let error = run().await.unwrap_err();
            assert!(error.to_string().contains("must be a socket address"));
            return;
        }

        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "tests::startup_initializes_runtime_before_rejecting_an_invalid_address",
                "--nocapture",
            ])
            .env("NVA_API_STARTUP_TEST_CHILD", "1")
            .env("PET_RESORT_TELEMETRY_MODE", "local")
            .env("PET_RESORT_API_ADDR", "not-a-socket-address")
            .status()
            .unwrap();
        assert!(status.success());
    }

    #[tokio::test]
    async fn server_honors_immediate_graceful_shutdown() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let state = http::VaccineDocumentState::default();

        serve(listener, state, async {}).await.unwrap();
    }
}
