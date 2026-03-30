//! # scryforge-sigilforge-client
//!
//! Client library for communicating with the Sigilforge auth daemon.
//!
//! This crate provides:
//! - [`SigilforgeClient`] - Client for fetching tokens from Sigilforge daemon
//! - [`TokenFetcher`] - Trait for components that need auth tokens
//! - [`MockTokenFetcher`] - Mock implementation for testing
//!
//! ## Example
//!
//! ```no_run
//! use scryforge_sigilforge_client::{SigilforgeClient, TokenFetcher};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = SigilforgeClient::new(PathBuf::from("/tmp/sigilforge.sock"));
//!
//! if client.is_available() {
//!     let token = client.fetch_token("spotify", "personal").await?;
//!     println!("Got token: {}", token);
//! } else {
//!     eprintln!("Sigilforge daemon not available");
//! }
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::debug;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Error)]
pub enum SigilforgeError {
    #[error("Sigilforge daemon not available: {0}")]
    Unavailable(String),

    #[error("Token not found for {service}/{account}")]
    TokenNotFound { service: String, account: String },

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

pub type Result<T> = std::result::Result<T, SigilforgeError>;

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetTokenResponse {
    token: String,
    expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResolveResponse {
    value: String,
}

// ============================================================================
// Client Implementation
// ============================================================================

/// Client for communicating with Sigilforge daemon over Unix socket.
///
/// The client maintains a connection to the Sigilforge daemon and provides
/// methods to fetch OAuth tokens and resolve credential references.
#[derive(Debug)]
pub struct SigilforgeClient {
    socket_path: PathBuf,
}

impl SigilforgeClient {
    /// Create a new Sigilforge client with the specified socket path.
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Path to the Sigilforge daemon Unix socket
    ///
    /// # Example
    ///
    /// ```
    /// use scryforge_sigilforge_client::SigilforgeClient;
    /// use std::path::PathBuf;
    ///
    /// let client = SigilforgeClient::new(PathBuf::from("/tmp/sigilforge.sock"));
    /// ```
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// Create a new client using the default socket path.
    ///
    /// The default path is determined by the platform:
    /// - Unix: `$XDG_RUNTIME_DIR/sigilforge.sock` or `/tmp/sigilforge.sock`
    /// - Windows: `\\.\pipe\sigilforge`
    pub fn with_default_path() -> Self {
        Self {
            socket_path: default_socket_path(),
        }
    }

    /// Check if the Sigilforge daemon is available.
    ///
    /// Returns `true` if the socket file exists, `false` otherwise.
    /// Note: This doesn't guarantee the daemon is actually running.
    pub fn is_available(&self) -> bool {
        self.socket_path.exists()
    }

    /// Get the socket path being used by this client.
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Get a fresh access token for the specified service and account.
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "spotify", "github")
    /// * `account` - Account identifier (e.g., "personal", "work")
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The daemon is not available
    /// - The connection fails
    /// - The account is not found
    /// - The RPC call fails
    pub async fn get_token(&self, service: &str, account: &str) -> Result<String> {
        if !self.is_available() {
            return Err(SigilforgeError::Unavailable(format!(
                "Socket not found at {:?}",
                self.socket_path
            )));
        }

        let response: GetTokenResponse = self
            .send_request("get_token", json!([service, account]))
            .await
            .map_err(|e| {
                if e.to_string().contains("not found") {
                    SigilforgeError::TokenNotFound {
                        service: service.to_string(),
                        account: account.to_string(),
                    }
                } else {
                    e
                }
            })?;

        Ok(response.token)
    }

    /// Resolve a credential reference to its actual value.
    ///
    /// # Arguments
    ///
    /// * `reference` - Credential reference (e.g., "auth://spotify/personal/token")
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The daemon is not available
    /// - The reference is invalid
    /// - The credential is not found
    pub async fn resolve(&self, reference: &str) -> Result<String> {
        if !self.is_available() {
            return Err(SigilforgeError::Unavailable(format!(
                "Socket not found at {:?}",
                self.socket_path
            )));
        }

        let response: ResolveResponse = self.send_request("resolve", json!([reference])).await?;
        Ok(response.value)
    }

    /// Send a JSON-RPC request and receive a typed response.
    async fn send_request<T>(&self, method: &str, params: serde_json::Value) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Connect to the daemon
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| {
                SigilforgeError::Connection(format!(
                    "Failed to connect to {:?}: {}",
                    self.socket_path, e
                ))
            })?;

        debug!("Connected to Sigilforge daemon at {:?}", self.socket_path);

        // Prepare JSON-RPC request
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1,
        });

        let request_str = serde_json::to_string(&request)?;
        debug!("Sending request: {}", request_str);

        // Send request
        stream.write_all(request_str.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        // Read response
        let mut reader = BufReader::new(&mut stream);
        let mut response_str = String::new();
        reader.read_line(&mut response_str).await?;

        debug!("Received response: {}", response_str.trim());

        // Parse response
        let response: serde_json::Value = serde_json::from_str(&response_str)?;

        // Check for errors
        if let Some(error) = response.get("error") {
            let error_msg = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(SigilforgeError::Rpc(error_msg.to_string()));
        }

        // Extract result
        let result = response
            .get("result")
            .ok_or_else(|| SigilforgeError::InvalidResponse("No result field".to_string()))?;

        Ok(serde_json::from_value(result.clone())?)
    }
}

// ============================================================================
// TokenFetcher Trait
// ============================================================================

/// Trait for components that need to fetch authentication tokens.
///
/// This trait provides an abstraction for token fetching, allowing
/// providers to request credentials without being coupled to the
/// Sigilforge implementation.
#[async_trait]
pub trait TokenFetcher: Send + Sync {
    /// Fetch an authentication token for the specified service and account.
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "spotify", "github")
    /// * `account` - Account identifier (e.g., "personal", "work")
    ///
    /// # Errors
    ///
    /// Returns an error if the token cannot be fetched.
    async fn fetch_token(&self, service: &str, account: &str) -> Result<String>;
}

#[async_trait]
impl TokenFetcher for SigilforgeClient {
    async fn fetch_token(&self, service: &str, account: &str) -> Result<String> {
        self.get_token(service, account).await
    }
}

// ============================================================================
// Mock Implementation for Testing
// ============================================================================

/// Mock token fetcher for testing purposes.
///
/// This implementation returns pre-configured tokens without
/// connecting to an actual Sigilforge daemon.
///
/// # Example
///
/// ```
/// use scryforge_sigilforge_client::{MockTokenFetcher, TokenFetcher};
/// use std::collections::HashMap;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut tokens = HashMap::new();
/// tokens.insert(
///     ("spotify".to_string(), "personal".to_string()),
///     "test_token_123".to_string()
/// );
///
/// let fetcher = MockTokenFetcher::new(tokens);
/// let token = fetcher.fetch_token("spotify", "personal").await?;
/// assert_eq!(token, "test_token_123");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct MockTokenFetcher {
    tokens: HashMap<(String, String), String>,
}

impl MockTokenFetcher {
    /// Create a new mock token fetcher with the given tokens.
    ///
    /// # Arguments
    ///
    /// * `tokens` - Map of (service, account) to token value
    pub fn new(tokens: HashMap<(String, String), String>) -> Self {
        Self { tokens }
    }

    /// Create an empty mock token fetcher.
    pub fn empty() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }

    /// Add a token to the mock fetcher.
    pub fn with_token(mut self, service: String, account: String, token: String) -> Self {
        self.tokens.insert((service, account), token);
        self
    }
}

#[async_trait]
impl TokenFetcher for MockTokenFetcher {
    async fn fetch_token(&self, service: &str, account: &str) -> Result<String> {
        self.tokens
            .get(&(service.to_string(), account.to_string()))
            .cloned()
            .ok_or_else(|| SigilforgeError::TokenNotFound {
                service: service.to_string(),
                account: account.to_string(),
            })
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Get the default socket path for the Sigilforge daemon.
///
/// The path is platform-dependent:
/// - Unix: `$XDG_RUNTIME_DIR/sigilforge.sock` or `/tmp/sigilforge.sock`
/// - Windows: `\\.\pipe\sigilforge`
pub fn default_socket_path() -> PathBuf {
    let dirs = ProjectDirs::from("com", "raibid-labs", "sigilforge");

    if cfg!(unix) {
        dirs.as_ref()
            .and_then(|d| d.runtime_dir())
            .map(|d| d.join("sigilforge.sock"))
            .unwrap_or_else(|| PathBuf::from("/tmp/sigilforge.sock"))
    } else {
        PathBuf::from(r"\\.\pipe\sigilforge")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_socket_path() {
        let path = default_socket_path();
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn test_client_creation() {
        let client = SigilforgeClient::new(PathBuf::from("/tmp/test.sock"));
        assert_eq!(client.socket_path(), Path::new("/tmp/test.sock"));
    }

    #[test]
    fn test_client_with_default_path() {
        let client = SigilforgeClient::with_default_path();
        assert!(!client.socket_path().as_os_str().is_empty());
    }

    #[tokio::test]
    async fn test_mock_token_fetcher() {
        let mut tokens = HashMap::new();
        tokens.insert(
            ("spotify".to_string(), "personal".to_string()),
            "test_token_123".to_string(),
        );

        let fetcher = MockTokenFetcher::new(tokens);
        let token = fetcher.fetch_token("spotify", "personal").await.unwrap();
        assert_eq!(token, "test_token_123");

        let result = fetcher.fetch_token("github", "work").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_token_fetcher_builder() {
        let fetcher = MockTokenFetcher::empty()
            .with_token(
                "spotify".to_string(),
                "personal".to_string(),
                "token1".to_string(),
            )
            .with_token(
                "github".to_string(),
                "work".to_string(),
                "token2".to_string(),
            );

        let token1 = fetcher.fetch_token("spotify", "personal").await.unwrap();
        assert_eq!(token1, "token1");

        let token2 = fetcher.fetch_token("github", "work").await.unwrap();
        assert_eq!(token2, "token2");
    }

    #[tokio::test]
    async fn test_client_unavailable() {
        let client = SigilforgeClient::new(PathBuf::from("/nonexistent/socket.sock"));
        assert!(!client.is_available());

        let result = client.get_token("spotify", "personal").await;
        assert!(matches!(result, Err(SigilforgeError::Unavailable(_))));
    }
}
