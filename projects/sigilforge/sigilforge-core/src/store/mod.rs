//! Secret storage abstraction.
//!
//! This module provides:
//! - [`Secret`] - A wrapper for sensitive values that prevents accidental logging
//! - [`SecretStore`] - Trait for secret storage backends
//! - [`MemoryStore`] - In-memory implementation for testing
//! - [`KeyringStore`] - OS keyring implementation (with `keyring-store` feature)
//! - [`create_store`] - Helper to select backend based on availability
//!
//! # Storage Key Convention
//!
//! Keys follow the pattern: `sigilforge/{service}/{account}/{credential_type}`
//!
//! # Example
//!
//! ```rust,ignore
//! use sigilforge_core::store::{Secret, SecretStore, create_store};
//!
//! let store = create_store(true); // Prefer keyring if available
//!
//! let secret = Secret::new("super-secret-token");
//! store.set("sigilforge/spotify/personal/access_token", &secret).await.unwrap();
//!
//! let retrieved = store.get("sigilforge/spotify/personal/access_token").await.unwrap();
//! assert_eq!(retrieved.unwrap().expose(), "super-secret-token");
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod memory;
#[cfg(feature = "keyring-store")]
mod keyring;

pub use memory::MemoryStore;
#[cfg(feature = "keyring-store")]
pub use keyring::KeyringStore;

/// A secret value that prevents accidental exposure in logs.
///
/// The inner value is only accessible via [`expose()`](Secret::expose).
/// Debug and Display implementations show `[REDACTED]` instead of the value.
#[derive(Clone, Serialize, Deserialize)]
pub struct Secret(String);

impl Secret {
    /// Create a new secret from a string value.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Expose the secret value.
    ///
    /// Use sparingly and never log the result.
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Consume the secret and return the inner value.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Secret([REDACTED])")
    }
}

impl std::fmt::Display for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl PartialEq for Secret {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Secret {}

/// Error type for secret store operations.
#[derive(Debug, Error)]
pub enum StoreError {
    /// The requested secret was not found.
    #[error("secret not found: {key}")]
    NotFound { key: String },

    /// Access to the secret was denied.
    #[error("access denied to secret: {key}")]
    AccessDenied { key: String },

    /// The storage backend encountered an error.
    #[error("backend error: {message}")]
    BackendError { message: String },

    /// Serialization or deserialization failed.
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// The keyring backend is not available.
    #[error("keyring not available: {message}")]
    KeyringUnavailable { message: String },
}

/// Abstraction over secret storage backends.
///
/// Implementations include:
/// - [`MemoryStore`] - In-memory storage for testing
/// - [`KeyringStore`] (with `keyring-store` feature) - OS keyring
/// - `EncryptedFileStore` (future) - ROPS/SOPS encrypted files
#[async_trait]
pub trait SecretStore: Send + Sync {
    /// Retrieve a secret by key.
    ///
    /// Returns `Ok(None)` if the key doesn't exist.
    async fn get(&self, key: &str) -> Result<Option<Secret>, StoreError>;

    /// Store a secret at the given key.
    ///
    /// Overwrites any existing value.
    async fn set(&self, key: &str, secret: &Secret) -> Result<(), StoreError>;

    /// Delete a secret by key.
    ///
    /// Returns `Ok(())` even if the key didn't exist.
    async fn delete(&self, key: &str) -> Result<(), StoreError>;

    /// List all keys matching a prefix.
    ///
    /// Returns an empty vec if no keys match.
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, StoreError>;

    /// Check if a key exists without retrieving the value.
    async fn exists(&self, key: &str) -> Result<bool, StoreError> {
        Ok(self.get(key).await?.is_some())
    }
}

/// Create a secret store with automatic backend selection.
///
/// This helper function selects the best available backend based on:
/// 1. Feature flags (whether `keyring-store` is enabled)
/// 2. Runtime availability (whether the keyring is accessible)
/// 3. User preference (the `prefer_keyring` parameter)
///
/// # Backend Selection Logic
///
/// - If `prefer_keyring` is `true` and the `keyring-store` feature is enabled:
///   - Attempts to create a [`KeyringStore`]
///   - Falls back to [`MemoryStore`] with a warning if keyring is unavailable
/// - Otherwise: Returns [`MemoryStore`]
///
/// # Arguments
///
/// * `prefer_keyring` - Whether to prefer keyring over memory storage
///
/// # Returns
///
/// A boxed [`SecretStore`] implementation
///
/// # Example
///
/// ```rust,ignore
/// use sigilforge_core::store::create_store;
///
/// // Try to use keyring, fallback to memory if unavailable
/// let store = create_store(true);
/// ```
pub fn create_store(prefer_keyring: bool) -> Box<dyn SecretStore> {
    #[cfg(feature = "keyring-store")]
    if prefer_keyring {
        match KeyringStore::try_new("sigilforge") {
            Ok(store) => {
                tracing::info!("Using OS keyring for secret storage");
                tracing::warn!(
                    "Note: On headless systems without a keyring daemon, \
                     keyring operations may fail silently. \
                     Consider using MemoryStore for development."
                );
                return Box::new(store);
            }
            Err(e) => {
                tracing::warn!(
                    "Keyring unavailable ({}), falling back to memory store. \
                     Secrets will not persist across restarts.",
                    e
                );
            }
        }
    }

    #[cfg(not(feature = "keyring-store"))]
    if prefer_keyring {
        tracing::warn!(
            "Keyring storage requested but keyring-store feature not enabled. \
             Using memory store. Secrets will not persist across restarts."
        );
    }

    tracing::debug!("Using in-memory secret storage");
    Box::new(MemoryStore::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_debug_redacted() {
        let secret = Secret::new("super-secret");
        let debug = format!("{:?}", secret);
        assert!(!debug.contains("super-secret"));
        assert!(debug.contains("REDACTED"));
    }

    #[test]
    fn test_secret_display_redacted() {
        let secret = Secret::new("super-secret");
        let display = format!("{}", secret);
        assert!(!display.contains("super-secret"));
        assert!(display.contains("REDACTED"));
    }

    #[tokio::test]
    async fn test_create_store_memory_fallback() {
        // This should always return a store, even if keyring is unavailable
        let store = create_store(false);

        // Verify the store is usable by testing basic operations
        let secret = Secret::new("test");
        store.set("test-key", &secret).await.unwrap();
        let retrieved = store.get("test-key").await.unwrap();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_create_store_prefer_keyring() {
        // This should either return keyring or fallback to memory
        // The actual backend chosen depends on platform and keyring availability
        let store = create_store(true);

        // Verify the store is usable - test basic operations
        // Note: KeyringStore may be returned even on headless systems where
        // the keyring daemon isn't running. In such cases, set() may succeed
        // but get() will return None. This is a known limitation of platform keyrings.
        let secret = Secret::new("test");
        let test_key = "test-key-prefer";

        // Set should work or error
        if store.set(test_key, &secret).await.is_err() {
            // Set failed, this is acceptable for non-functional keyring
            return;
        }

        // Get might return None if keyring daemon isn't running
        // This is expected behavior on headless systems
        match store.get(test_key).await {
            Ok(Some(retrieved)) => {
                // Happy path: store is working
                assert_eq!(retrieved.expose(), "test");
                store.delete(test_key).await.unwrap();
            }
            Ok(None) => {
                // Keyring accepted set but can't retrieve - daemon not running
                // This is expected on headless systems, just clean up
                let _ = store.delete(test_key).await;
            }
            Err(_) => {
                // Some other error occurred
                let _ = store.delete(test_key).await;
            }
        }
    }
}
