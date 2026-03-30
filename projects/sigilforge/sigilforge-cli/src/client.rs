//! Daemon client for communicating with sigilforged.
//!
//! This module provides a client for connecting to the Sigilforge daemon
//! over a Unix socket (or named pipe on Windows) using JSON-RPC.

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, warn};

/// Response containing a fresh access token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTokenResponse {
    pub token: String,
    pub expires_at: Option<String>,
}

/// Information about a configured account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub service: String,
    pub account: String,
    pub scopes: Vec<String>,
    pub created_at: String,
    pub last_used: Option<String>,
}

/// Response containing a list of accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAccountsResponse {
    pub accounts: Vec<AccountInfo>,
}

/// Response after adding an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAccountResponse {
    pub message: String,
}

/// Response containing a resolved value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveResponse {
    pub value: String,
}

/// Client for communicating with the Sigilforge daemon.
pub struct DaemonClient {
    stream: Option<UnixStream>,
    #[allow(dead_code)]
    socket_path: PathBuf,
    next_id: u64,
}

impl DaemonClient {
    /// Attempt to connect to the daemon at the given socket path.
    pub async fn connect(socket_path: &Path) -> Result<Self> {
        debug!("Attempting to connect to daemon at {:?}", socket_path);

        // Check if socket exists
        if !socket_path.exists() {
            debug!("Socket does not exist at {:?}", socket_path);
            return Ok(Self {
                stream: None,
                socket_path: socket_path.to_path_buf(),
                next_id: 1,
            });
        }

        // Try to connect to the Unix socket
        match UnixStream::connect(socket_path).await {
            Ok(stream) => {
                debug!("Successfully connected to daemon");
                Ok(Self {
                    stream: Some(stream),
                    socket_path: socket_path.to_path_buf(),
                    next_id: 1,
                })
            }
            Err(e) => {
                warn!("Failed to connect to daemon: {}", e);
                Ok(Self {
                    stream: None,
                    socket_path: socket_path.to_path_buf(),
                    next_id: 1,
                })
            }
        }
    }

    /// Connect to daemon using default socket path.
    pub async fn connect_default() -> Result<Self> {
        let socket_path = default_socket_path();
        Self::connect(&socket_path).await
    }

    /// Check if the client is connected to the daemon.
    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    /// Send a JSON-RPC request and receive a response.
    async fn send_request<T: for<'de> Deserialize<'de>>(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Not connected to daemon"))?;

        let id = self.next_id;
        self.next_id += 1;

        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id,
        });

        let request_str = serde_json::to_string(&request)?;
        debug!("Sending request: {}", request_str);

        stream.write_all(request_str.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        let mut reader = BufReader::new(stream);
        let mut response_str = String::new();
        reader.read_line(&mut response_str).await?;

        debug!("Received response: {}", response_str);

        let response: serde_json::Value = serde_json::from_str(&response_str)?;

        if let Some(error) = response.get("error") {
            anyhow::bail!("RPC error: {}", error);
        }

        let result = response
            .get("result")
            .ok_or_else(|| anyhow::anyhow!("No result in response"))?;

        Ok(serde_json::from_value(result.clone())?)
    }

    /// Get a fresh access token for the specified account.
    pub async fn get_token(&mut self, service: &str, account: &str) -> Result<GetTokenResponse> {
        self.send_request("get_token", json!([service, account]))
            .await
    }

    /// List all configured accounts, optionally filtered by service.
    pub async fn list_accounts(
        &mut self,
        service: Option<&str>,
    ) -> Result<ListAccountsResponse> {
        self.send_request("list_accounts", json!([service]))
            .await
    }

    /// Add a new account with the specified scopes.
    pub async fn add_account(
        &mut self,
        service: &str,
        account: &str,
        scopes: Vec<String>,
    ) -> Result<AddAccountResponse> {
        self.send_request("add_account", json!([service, account, scopes]))
            .await
    }

    /// Resolve a credential reference to its actual value.
    pub async fn resolve(&mut self, reference: &str) -> Result<ResolveResponse> {
        self.send_request("resolve", json!([reference])).await
    }
}

/// Get the default socket path for the daemon.
pub fn default_socket_path() -> PathBuf {
    let dirs = ProjectDirs::from("com", "raibid-labs", "sigilforge");

    if cfg!(unix) {
        dirs.as_ref()
            .map(|d| d.runtime_dir().unwrap_or(d.data_dir()).join("sigilforge.sock"))
            .unwrap_or_else(|| PathBuf::from("/tmp/sigilforge.sock"))
    } else {
        PathBuf::from(r"\\.\pipe\sigilforge")
    }
}
