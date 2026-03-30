//! JSON-RPC API module for the scryforge-daemon.
//!
//! This module exposes the daemon's functionality to clients (TUI, web, etc.)
//! over a JSON-RPC interface via TCP.

pub mod handlers;
pub mod server;

pub use server::start_server;
