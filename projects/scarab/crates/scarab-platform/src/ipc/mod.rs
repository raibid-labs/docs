//! Cross-platform IPC implementation
//!
//! This module provides a unified interface for Inter-Process Communication
//! using Unix domain sockets on Unix-like systems and Named Pipes on Windows.

use anyhow::Result;
use std::io::{Read, Write};

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use unix::{IpcClient, IpcListener, IpcStream};

#[cfg(windows)]
pub use windows::{IpcClient, IpcListener, IpcStream};

/// Common IPC traits
pub trait IpcConnection: Read + Write + Send {
    /// Check if the connection is still active
    fn is_connected(&self) -> bool;

    /// Get the connection ID
    fn id(&self) -> String;

    /// Shutdown the connection
    fn shutdown(&mut self) -> Result<()>;
}

/// IPC server trait
pub trait IpcServer {
    type Stream: IpcConnection;

    /// Accept a new connection
    fn accept(&self) -> Result<Self::Stream>;

    /// Get the server address/name
    fn address(&self) -> String;

    /// Shutdown the server
    fn shutdown(&mut self) -> Result<()>;
}

/// IPC configuration
#[derive(Debug, Clone)]
pub struct IpcConfig {
    /// Buffer size for read/write operations
    pub buffer_size: usize,
    /// Timeout for connection attempts (in milliseconds)
    pub connect_timeout: u64,
    /// Timeout for read operations (in milliseconds)
    pub read_timeout: Option<u64>,
    /// Timeout for write operations (in milliseconds)
    pub write_timeout: Option<u64>,
    /// Maximum number of pending connections
    pub max_connections: u32,
}

impl Default for IpcConfig {
    fn default() -> Self {
        Self {
            buffer_size: 8192,
            connect_timeout: 5000,
            read_timeout: None,
            write_timeout: None,
            max_connections: 128,
        }
    }
}

/// Create platform-specific IPC server
pub fn create_server(name: &str, config: &IpcConfig) -> Result<impl IpcServer> {
    #[cfg(unix)]
    {
        unix::IpcListener::new(name, config)
    }
    #[cfg(windows)]
    {
        windows::IpcListener::new(name, config)
    }
}

/// Create platform-specific IPC client
pub fn create_client(name: &str, config: &IpcConfig) -> Result<impl IpcConnection> {
    #[cfg(unix)]
    {
        unix::IpcClient::connect(name, config)
    }
    #[cfg(windows)]
    {
        windows::IpcClient::connect(name, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = IpcConfig::default();
        assert_eq!(config.buffer_size, 8192);
        assert_eq!(config.connect_timeout, 5000);
        assert_eq!(config.max_connections, 128);
    }
}
