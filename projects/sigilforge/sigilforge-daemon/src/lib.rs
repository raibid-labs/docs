//! Sigilforge Daemon Library
//!
//! This library exposes the daemon's API and configuration for testing
//! and potential embedding in other applications.

pub mod api;
pub mod config;

pub use api::{start_server, ApiState, ServerConfig};
pub use config::{load_config, DaemonConfig};
