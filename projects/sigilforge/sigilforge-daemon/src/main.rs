//! Sigilforge Daemon
//!
//! Background service that manages credentials and exposes a local API
//! for Sigilforge clients.
//!
//! # Running
//!
//! ```bash
//! cargo run -p sigilforge-daemon
//! # or after install:
//! sigilforged
//! ```

use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod api;
mod config;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    info!("Starting Sigilforge daemon...");

    let config = config::load_config()?;
    info!("Loaded configuration from {:?}", config.config_path);

    run_daemon(config).await
}

fn init_logging() {
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();
}

async fn run_daemon(config: config::DaemonConfig) -> Result<()> {
    info!("Daemon starting on {:?}", config.socket_path);

    // Create API state
    let state = api::ApiState::new()?;

    // Start the JSON-RPC server
    let server_handle = api::start_server(&config.socket_path, state).await?;

    info!("Daemon running. Press Ctrl+C to stop.");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received, stopping server...");

    // Stop the server gracefully
    server_handle.stop().await?;
    server_handle.stopped().await;

    // Clean up socket file
    if config.socket_path.exists() {
        std::fs::remove_file(&config.socket_path)?;
        info!("Socket file removed");
    }

    info!("Daemon stopped");
    Ok(())
}
