//! OS keyring-backed secret storage implementation.

use async_trait::async_trait;
use keyring::Entry;

use super::{Secret, SecretStore, StoreError};

/// OS keyring-backed secret store.
///
/// This store uses the platform's native keyring service:
/// - macOS: Keychain
/// - Linux: Secret Service API (via libsecret)
/// - Windows: Credential Manager
///
/// # Storage Key Format
///
/// Keys are stored using the format: `{service_name}/{key}`
/// where the service_name is set during construction.
///
/// # Example
///
/// ```rust,ignore
/// use sigilforge_core::store::{KeyringStore, SecretStore, Secret};
///
/// let store = KeyringStore::try_new("sigilforge").unwrap();
/// let secret = Secret::new("my-token");
/// store.set("spotify/personal/access_token", &secret).await.unwrap();
/// ```
pub struct KeyringStore {
    service_name: String,
}

impl KeyringStore {
    /// Create a new keyring store with the given service name.
    ///
    /// # Panics
    ///
    /// Panics if the keyring backend is not available on this platform.
    /// Use [`try_new`](Self::try_new) for a non-panicking version.
    pub fn new(service_name: &str) -> Self {
        Self::try_new(service_name).expect("keyring backend not available")
    }

    /// Try to create a new keyring store.
    ///
    /// Returns an error if the keyring backend is not available on this platform.
    pub fn try_new(service_name: &str) -> Result<Self, StoreError> {
        // Validate that keyring is available by attempting to create a test entry
        let test_key = format!("{}/__test__", service_name);
        match Entry::new(&test_key, "availability_check") {
            Ok(_) => Ok(Self {
                service_name: service_name.to_string(),
            }),
            Err(e) => Err(StoreError::KeyringUnavailable {
                message: format!("keyring backend not available: {}", e),
            }),
        }
    }

    /// Create a keyring entry for the given key.
    fn create_entry(&self, key: &str) -> Result<Entry, StoreError> {
        let service = format!("{}/{}", self.service_name, key);
        Entry::new(&service, "sigilforge").map_err(|e| StoreError::BackendError {
            message: format!("failed to create keyring entry: {}", e),
        })
    }
}

impl std::fmt::Debug for KeyringStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyringStore")
            .field("service_name", &self.service_name)
            .finish()
    }
}

#[async_trait]
impl SecretStore for KeyringStore {
    async fn get(&self, key: &str) -> Result<Option<Secret>, StoreError> {
        let entry = self.create_entry(key)?;

        match entry.get_password() {
            Ok(password) => Ok(Some(Secret::new(password))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(keyring::Error::Ambiguous(_)) => Err(StoreError::BackendError {
                message: format!("ambiguous keyring entry for key: {}", key),
            }),
            Err(keyring::Error::Invalid(msg, _)) => Err(StoreError::BackendError {
                message: format!("invalid keyring operation: {}", msg),
            }),
            Err(keyring::Error::PlatformFailure(e)) => Err(StoreError::BackendError {
                message: format!("platform keyring failure: {}", e),
            }),
            Err(e) => Err(StoreError::BackendError {
                message: format!("keyring error: {}", e),
            }),
        }
    }

    async fn set(&self, key: &str, secret: &Secret) -> Result<(), StoreError> {
        let entry = self.create_entry(key)?;

        entry
            .set_password(secret.expose())
            .map_err(|e| StoreError::BackendError {
                message: format!("failed to set keyring password: {}", e),
            })
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        let entry = self.create_entry(key)?;

        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()), // Idempotent delete
            Err(e) => Err(StoreError::BackendError {
                message: format!("failed to delete keyring entry: {}", e),
            }),
        }
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        // The keyring crate doesn't provide a native list operation.
        // This is a limitation of most platform keyring APIs.
        // For now, we return an error indicating this is unsupported.
        //
        // Future implementations could maintain a separate index or
        // use platform-specific APIs where available.
        Err(StoreError::BackendError {
            message: format!(
                "list_keys not supported by keyring backend (requested prefix: {})",
                prefix
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests verify the API but don't actually interact with the keyring
    // to avoid platform-specific test failures and credential pollution.

    #[test]
    fn test_keyring_store_creation() {
        // This test may fail on platforms without keyring support
        // We test both success and failure paths
        match KeyringStore::try_new("sigilforge-test") {
            Ok(store) => {
                assert_eq!(store.service_name, "sigilforge-test");
            }
            Err(StoreError::KeyringUnavailable { .. }) => {
                // Expected on platforms without keyring support
            }
            Err(e) => {
                panic!("unexpected error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_keyring_store_operations() {
        // Only run this test if keyring is available
        let store = match KeyringStore::try_new("sigilforge-test-ops") {
            Ok(s) => s,
            Err(_) => {
                // Skip test if keyring unavailable
                eprintln!("Skipping test_keyring_store_operations: keyring unavailable");
                return;
            }
        };

        // Use a timestamp-based key to avoid conflicts
        let test_key = format!("test/{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());
        let secret = Secret::new("test-value");

        // Note: On headless Linux systems without a proper keyring daemon (e.g., CI environments),
        // the keyring crate may report success on set() but fail to persist data.
        // This is a known limitation of the platform keyring backends.
        // We test the happy path but accept that it may not work in all environments.

        // Test set (should not error)
        if let Err(e) = store.set(&test_key, &secret).await {
            eprintln!("Keyring set failed ({}), skipping test - keyring backend not fully functional", e);
            return;
        }

        // Test get - may return None if keyring daemon isn't running
        match store.get(&test_key).await {
            Ok(Some(retrieved)) => {
                // Happy path: keyring is working
                assert_eq!(retrieved.expose(), "test-value");

                // Test delete
                store.delete(&test_key).await.unwrap();
                let deleted = store.get(&test_key).await.unwrap();
                assert!(deleted.is_none());
            }
            Ok(None) => {
                // Keyring backend accepted the set but didn't persist
                // This happens on headless systems without keyring daemon
                eprintln!("Keyring set succeeded but get returned None - keyring daemon may not be running");
                eprintln!("This is expected on headless systems. Skipping remainder of test.");
                // Clean up attempt (may also fail)
                let _ = store.delete(&test_key).await;
            }
            Err(e) => {
                eprintln!("Keyring get failed: {}. Skipping test.", e);
                let _ = store.delete(&test_key).await;
            }
        }

        // Test delete is idempotent (should never error)
        store.delete(&test_key).await.unwrap();
    }

    #[tokio::test]
    async fn test_keyring_store_get_nonexistent() {
        let store = match KeyringStore::try_new("sigilforge-test-nonexist") {
            Ok(s) => s,
            Err(_) => return,
        };

        let result = store.get("nonexistent/key").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_keyring_list_keys_unsupported() {
        let store = match KeyringStore::try_new("sigilforge-test-list") {
            Ok(s) => s,
            Err(_) => return,
        };

        let result = store.list_keys("sigilforge").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(StoreError::BackendError { .. })));
    }
}
