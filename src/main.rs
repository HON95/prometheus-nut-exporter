mod common;
mod config;
mod http_server;
mod meta;
mod metrics;
mod nut_client;
mod openmetrics_builder;

use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    // Setup logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(config::Config::DEFAULT_LOG_LEVEL)).init();

    // Setup config
    let config = config::read_config();
    if config.print_metrics_and_exit {
        metrics::print_metrics();
        return;
    }

    // Start server
    let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
    let server_task = tokio::spawn(http_server::run_server(config, shutdown_tx.subscribe()));

    // Listen for shutdown signals
    let mut sigint_stream = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm_stream = signal(SignalKind::terminate()).unwrap();
    tokio::select! {
        _ = shutdown_rx.recv() => {
            log::debug!("Received internal shutdown signal.");
        },
        _ = sigint_stream.recv() => {
            log::debug!("Received interrupt signal.");
            shutdown_tx.send(true).unwrap();
        },
        _ = sigterm_stream.recv() => {
            log::debug!("Received termination signal.");
            shutdown_tx.send(true).unwrap();
        },
    }

    // Wait for server
    server_task.await.unwrap();
}
