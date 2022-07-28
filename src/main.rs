mod common;
mod config;
mod http_server;
mod meta;
mod metrics;
mod nut_client;
mod openmetrics_builder;

// Set env var RUST_LOG to override
pub const LOG_LEVEL_DEFAULT: &str = "info";

#[tokio::main]
async fn main() {
    // Setup logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(LOG_LEVEL_DEFAULT)).init();

    // Setup config
    let config = config::read_config();
    if config.print_metrics_and_exit {
        metrics::print_metrics();
        return;
    }

    // Run
    http_server::run_server(config).await;
}
