//! raibid-server
//!
//! API server for job dispatching and TUI communication.
//! This crate handles:
//! - Job queue management
//! - Agent registration and health checks
//! - Real-time status updates for TUI
//! - WebSocket connections for live monitoring
//!
//! ## Architecture
//!
//! The server is built with Axum and follows a modular architecture:
//! - `config`: Configuration management
//! - `state`: Shared application state
//! - `routes`: HTTP route handlers
//! - `middleware`: Custom middleware (logging, auth, etc.)
//! - `error`: Error types and handling
//!
//! ## Example
//!
//! ```no_run
//! use raibid_server::{Server, ServerConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = ServerConfig::default();
//!     let server = Server::new(config);
//!     server.run().await
//! }
//! ```

pub mod config;
pub mod error;
pub mod middleware;
pub mod routes;
pub mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::info;

pub use config::ServerConfig;
pub use error::{ServerError, ServerResult};
pub use state::AppState;

/// Main server struct
pub struct Server {
    config: ServerConfig,
    state: Arc<AppState>,
}

impl Server {
    /// Create a new server instance
    pub fn new(config: ServerConfig) -> Self {
        let state = Arc::new(AppState::new());
        Self { config, state }
    }

    /// Create a new server instance with custom state
    pub fn with_state(config: ServerConfig, state: AppState) -> Self {
        Self {
            config,
            state: Arc::new(state),
        }
    }

    /// Build the Axum router with all routes and middleware
    fn build_router(&self) -> Router {
        Router::new()
            .merge(routes::health::routes())
            .merge(routes::jobs::routes())
            .merge(routes::webhooks::routes())
            .layer(TraceLayer::new_for_http())
            .layer(middleware::request_id::RequestIdLayer)
            .with_state(self.state.clone())
    }

    /// Run the server
    pub async fn run(self) -> anyhow::Result<()> {
        // Initialize tracing (ignore error if already initialized)
        let _ = self.init_tracing();

        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

        info!("Starting raibid-server on {}", addr);

        let app = self.build_router();

        let listener = tokio::net::TcpListener::bind(addr).await?;
        info!("Server listening on {}", addr);

        // Run server with graceful shutdown
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        info!("Server shutdown complete");

        Ok(())
    }

    /// Initialize tracing/logging
    fn init_tracing(&self) -> anyhow::Result<()> {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "raibid_server=debug,tower_http=debug,axum=trace".into());

        let result = if self.config.log_format == "json" {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().json())
                .try_init()
        } else {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer())
                .try_init()
        };

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                // If tracing is already initialized, that's okay in tests
                if cfg!(test) {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Failed to initialize tracing: {}", e))
                }
            }
        }
    }
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            info!("Received terminate signal");
        },
    }
}
