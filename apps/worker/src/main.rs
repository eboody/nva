use anyhow::Context;
use pet_resort_worker::runtime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let config = runtime::Config::from_env_defaults();

    tracing::info!(
        agent_runtime_mode = ?config.agent_runtime_mode(),
        side_effect_mode = ?config.side_effect_mode(),
        "pet-resort worker shell started; durable leasing is implemented by downstream data-model/workflow cards"
    );

    tokio::signal::ctrl_c()
        .await
        .context("worker shutdown signal listener failed")
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("pet_resort_worker=info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()
        .init();
}
