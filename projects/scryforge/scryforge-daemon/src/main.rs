//! # scryforge-daemon
//!
//! The Scryforge hub daemon.
//!
//! This daemon is responsible for:
//! - Loading and managing provider plugins
//! - Periodic sync and caching of stream data
//! - Token retrieval from Sigilforge for OAuth providers
//! - Exposing the daemon API over Unix socket for TUI and other clients
//! - Managing local state (SQLite cache)
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    scryforge-daemon                          │
//! │                                                              │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
//! │  │   Provider   │  │   Provider   │  │   Provider   │      │
//! │  │   Registry   │  │   Sync Loop  │  │   Cache      │      │
//! │  └──────────────┘  └──────────────┘  └──────────────┘      │
//! │                                                              │
//! │  ┌────────────────────────────────────────────────────┐    │
//! │  │              JSON-RPC API Server                    │    │
//! │  │           (Unix socket / TCP)                       │    │
//! │  └────────────────────────────────────────────────────┘    │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!           ┌──────────────────┼──────────────────┐
//!           ▼                  ▼                  ▼
//!      ┌─────────┐      ┌─────────────┐    ┌──────────┐
//!      │   TUI   │      │   Scarab    │    │  Other   │
//!      │ Client  │      │   (future)  │    │ Clients  │
//!      └─────────┘      └─────────────┘    └──────────┘
//! ```
//!
//! ## Configuration
//!
//! The daemon reads configuration from `$XDG_CONFIG_HOME/scryforge/config.toml`.
//! See `docs/ARCHITECTURE.md` for configuration options.
//!
//! ## Running
//!
//! ```bash
//! # Start the daemon
//! cargo run --bin scryforge-daemon
//!
//! # With debug logging
//! RUST_LOG=debug cargo run --bin scryforge-daemon
//! ```

pub mod registry;

use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod api;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(true)
        .init();

    info!("Starting scryforge-daemon v{}", env!("CARGO_PKG_VERSION"));

    // TODO: Load configuration from config.toml
    // let config = load_config()?;

    // Initialize provider registry
    let mut registry = registry::ProviderRegistry::new();

    // Load dummy provider for testing
    info!("Loading dummy provider...");
    registry.register(provider_dummy::DummyProvider::new());

    // Display registered providers
    let provider_ids = registry.list();
    info!("Registered {} provider(s): {:?}", provider_ids.len(), provider_ids);

    // Verify dummy provider is accessible
    if let Some(provider) = registry.get("dummy") {
        info!("Dummy provider loaded: {}", provider.name());

        // Perform health check
        match provider.health_check().await {
            Ok(health) => {
                info!("Provider health check: healthy={}, message={:?}",
                      health.is_healthy, health.message);
            }
            Err(e) => {
                info!("Provider health check failed: {}", e);
            }
        }
    }

    // TODO: Initialize cache (SQLite)
    // let cache = Cache::open(&config.cache_path)?;

    // TODO: Connect to Sigilforge for auth
    // let sigilforge = SigilforgeClient::connect(&config.sigilforge_socket)?;

    // TODO: Start sync loop
    // let sync_handle = tokio::spawn(sync_loop(registry.clone(), cache.clone()));

    // Start the JSON-RPC API server
    let (server_handle, addr) = api::start_server().await?;

    info!("Daemon startup complete");
    info!("Listening on: {}", addr);
    info!("Press Ctrl+C to stop");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;

    info!("Shutting down...");

    // Stop the server gracefully
    server_handle.stop()?;

    info!("Daemon stopped");
    Ok(())
}

// ============================================================================
// TODO: Module stubs for future implementation
// ============================================================================

// mod config {
//     //! Configuration loading and management
// }

// mod cache {
//     //! SQLite-based caching for items and streams
// }

// mod sync {
//     //! Background sync loop for fetching new data
// }

// mod sigilforge {
//     //! Client for communicating with Sigilforge auth daemon
// }
