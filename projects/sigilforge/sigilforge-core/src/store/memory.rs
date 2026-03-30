//! In-memory secret storage implementation.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

use super::{Secret, SecretStore, StoreError};

/// In-memory secret store for testing and development.
///
/// This store is not persistent; data is lost when the process exits.
///
/// # Thread Safety
///
/// This implementation uses interior mutability via `RwLock` and is
/// safe to share across threads.
pub struct MemoryStore {
    data: RwLock<HashMap<String, Secret>>,
}

impl MemoryStore {
    /// Create a new empty memory store.
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }

    /// Create a memory store with initial data.
    pub fn with_data(data: HashMap<String, Secret>) -> Self {
        Self {
            data: RwLock::new(data),
        }
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for MemoryStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let count = self.data.read().map(|d| d.len()).unwrap_or(0);
        f.debug_struct("MemoryStore")
            .field("keys_count", &count)
            .finish()
    }
}

#[async_trait]
impl SecretStore for MemoryStore {
    async fn get(&self, key: &str) -> Result<Option<Secret>, StoreError> {
        let data = self.data.read().map_err(|e| StoreError::BackendError {
            message: format!("lock poisoned: {}", e),
        })?;
        Ok(data.get(key).cloned())
    }

    async fn set(&self, key: &str, secret: &Secret) -> Result<(), StoreError> {
        let mut data = self.data.write().map_err(|e| StoreError::BackendError {
            message: format!("lock poisoned: {}", e),
        })?;
        data.insert(key.to_string(), secret.clone());
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        let mut data = self.data.write().map_err(|e| StoreError::BackendError {
            message: format!("lock poisoned: {}", e),
        })?;
        data.remove(key);
        Ok(())
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        let data = self.data.read().map_err(|e| StoreError::BackendError {
            message: format!("lock poisoned: {}", e),
        })?;
        let keys: Vec<String> = data
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_store_set_get() {
        let store = MemoryStore::new();
        let secret = Secret::new("test-value");

        store.set("test-key", &secret).await.unwrap();
        let retrieved = store.get("test-key").await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().expose(), "test-value");
    }

    #[tokio::test]
    async fn test_memory_store_get_nonexistent() {
        let store = MemoryStore::new();
        let result = store.get("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_memory_store_delete() {
        let store = MemoryStore::new();
        let secret = Secret::new("test-value");

        store.set("test-key", &secret).await.unwrap();
        store.delete("test-key").await.unwrap();

        let result = store.get("test-key").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_memory_store_list_keys() {
        let store = MemoryStore::new();

        store
            .set("sigilforge/spotify/personal/token", &Secret::new("t1"))
            .await
            .unwrap();
        store
            .set("sigilforge/spotify/work/token", &Secret::new("t2"))
            .await
            .unwrap();
        store
            .set("sigilforge/github/main/token", &Secret::new("t3"))
            .await
            .unwrap();

        let spotify_keys = store.list_keys("sigilforge/spotify").await.unwrap();
        assert_eq!(spotify_keys.len(), 2);

        let all_keys = store.list_keys("sigilforge").await.unwrap();
        assert_eq!(all_keys.len(), 3);
    }

    #[tokio::test]
    async fn test_memory_store_exists() {
        let store = MemoryStore::new();
        let secret = Secret::new("test-value");

        assert!(!store.exists("test-key").await.unwrap());

        store.set("test-key", &secret).await.unwrap();

        assert!(store.exists("test-key").await.unwrap());
    }
}
