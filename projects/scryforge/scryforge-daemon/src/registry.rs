//! # Provider Registry
//!
//! Manages the collection of loaded providers and provides lookup functionality.
//!
//! The registry stores providers by their ID and allows retrieval by ID or listing
//! all available providers. Providers are stored as trait objects to enable runtime
//! polymorphism.

use fusabi_streams_core::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry for managing loaded providers.
///
/// The registry maintains a collection of providers that have been loaded
/// and initialized. Each provider is identified by its unique ID.
///
/// # Example
///
/// ```no_run
/// use scryforge_daemon::registry::ProviderRegistry;
/// use provider_dummy::DummyProvider;
///
/// let mut registry = ProviderRegistry::new();
/// registry.register(DummyProvider::new());
///
/// let provider = registry.get("dummy").unwrap();
/// println!("Loaded provider: {}", provider.name());
/// ```
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn Provider>>,
}

impl ProviderRegistry {
    /// Create a new empty provider registry.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a provider with the registry.
    ///
    /// The provider's ID will be used as the key. If a provider with the same ID
    /// already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider instance to register
    pub fn register<P>(&mut self, provider: P)
    where
        P: Provider + 'static,
    {
        let id = provider.id().to_string();
        self.providers.insert(id, Arc::new(provider));
    }

    /// Get a provider by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the provider
    ///
    /// # Returns
    ///
    /// An `Option` containing an Arc to the provider if found, or `None` if no
    /// provider with the given ID is registered.
    pub fn get(&self, id: &str) -> Option<Arc<dyn Provider>> {
        self.providers.get(id).cloned()
    }

    /// List all registered provider IDs.
    ///
    /// # Returns
    ///
    /// A vector of string slices containing the IDs of all registered providers.
    pub fn list(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    /// Get the number of registered providers.
    pub fn count(&self) -> usize {
        self.providers.len()
    }

    /// Check if a provider with the given ID is registered.
    pub fn contains(&self, id: &str) -> bool {
        self.providers.contains_key(id)
    }

    /// Remove a provider from the registry.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the provider to remove
    ///
    /// # Returns
    ///
    /// An `Option` containing the removed provider if it existed, or `None` if no
    /// provider with the given ID was registered.
    pub fn remove(&mut self, id: &str) -> Option<Arc<dyn Provider>> {
        self.providers.remove(id)
    }

    /// Clear all providers from the registry.
    pub fn clear(&mut self) {
        self.providers.clear();
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    // Mock provider for testing
    struct MockProvider {
        id: String,
    }

    impl MockProvider {
        fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
            }
        }
    }

    #[async_trait]
    impl Provider for MockProvider {
        fn id(&self) -> &'static str {
            Box::leak(self.id.clone().into_boxed_str())
        }

        fn name(&self) -> &'static str {
            "Mock Provider"
        }

        async fn health_check(&self) -> Result<ProviderHealth> {
            Ok(ProviderHealth {
                is_healthy: true,
                message: None,
                last_sync: None,
                error_count: 0,
            })
        }

        async fn sync(&self) -> Result<SyncResult> {
            Ok(SyncResult {
                success: true,
                items_added: 0,
                items_updated: 0,
                items_removed: 0,
                errors: vec![],
                duration_ms: 0,
            })
        }

        fn capabilities(&self) -> ProviderCapabilities {
            ProviderCapabilities::default()
        }

        async fn available_actions(&self, _item: &Item) -> Result<Vec<Action>> {
            Ok(vec![])
        }

        async fn execute_action(&self, _item: &Item, _action: &Action) -> Result<ActionResult> {
            Ok(ActionResult {
                success: true,
                message: None,
                data: None,
            })
        }
    }

    #[test]
    fn test_new_registry() {
        let registry = ProviderRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_register_provider() {
        let mut registry = ProviderRegistry::new();
        registry.register(MockProvider::new("test"));

        assert_eq!(registry.count(), 1);
        assert!(registry.contains("test"));
    }

    #[test]
    fn test_get_provider() {
        let mut registry = ProviderRegistry::new();
        registry.register(MockProvider::new("test"));

        let provider = registry.get("test");
        assert!(provider.is_some());
        assert_eq!(provider.unwrap().id(), "test");
    }

    #[test]
    fn test_get_nonexistent_provider() {
        let registry = ProviderRegistry::new();
        let provider = registry.get("nonexistent");
        assert!(provider.is_none());
    }

    #[test]
    fn test_list_providers() {
        let mut registry = ProviderRegistry::new();
        registry.register(MockProvider::new("test1"));
        registry.register(MockProvider::new("test2"));

        let mut ids = registry.list();
        ids.sort();

        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], "test1");
        assert_eq!(ids[1], "test2");
    }

    #[test]
    fn test_remove_provider() {
        let mut registry = ProviderRegistry::new();
        registry.register(MockProvider::new("test"));

        assert_eq!(registry.count(), 1);

        let removed = registry.remove("test");
        assert!(removed.is_some());
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_clear_registry() {
        let mut registry = ProviderRegistry::new();
        registry.register(MockProvider::new("test1"));
        registry.register(MockProvider::new("test2"));

        assert_eq!(registry.count(), 2);

        registry.clear();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_replace_provider() {
        let mut registry = ProviderRegistry::new();
        registry.register(MockProvider::new("test"));

        assert_eq!(registry.count(), 1);

        // Register another provider with the same ID
        registry.register(MockProvider::new("test"));

        // Should still have only one provider
        assert_eq!(registry.count(), 1);
    }
}
