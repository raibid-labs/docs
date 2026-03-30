//! Integration tests for the provider registry.
//!
//! These tests verify that the provider registry can load and discover providers,
//! and that providers can be accessed through the registry.

use provider_dummy::DummyProvider;
use scryforge_daemon::registry::ProviderRegistry;

#[test]
fn test_registry_discovery() {
    let mut registry = ProviderRegistry::new();

    // Initially empty
    assert_eq!(registry.count(), 0);
    assert!(registry.list().is_empty());

    // Register dummy provider
    registry.register(DummyProvider::new());

    // Verify registration
    assert_eq!(registry.count(), 1);
    assert!(registry.contains("dummy"));

    let provider_ids = registry.list();
    assert_eq!(provider_ids.len(), 1);
    assert_eq!(provider_ids[0], "dummy");
}

#[test]
fn test_registry_provider_access() {
    let mut registry = ProviderRegistry::new();
    registry.register(DummyProvider::new());

    // Get the provider
    let provider = registry.get("dummy");
    assert!(provider.is_some());

    let provider = provider.unwrap();
    assert_eq!(provider.id(), "dummy");
    assert_eq!(provider.name(), "Dummy Provider");
}

#[tokio::test]
async fn test_registry_provider_health_check() {
    let mut registry = ProviderRegistry::new();
    registry.register(DummyProvider::new());

    let provider = registry.get("dummy").unwrap();

    // Health check should succeed
    let health = provider.health_check().await;
    assert!(health.is_ok());

    let health = health.unwrap();
    assert!(health.is_healthy);
    assert_eq!(health.error_count, 0);
}

#[tokio::test]
async fn test_registry_provider_capabilities() {
    let mut registry = ProviderRegistry::new();
    registry.register(DummyProvider::new());

    let provider = registry.get("dummy").unwrap();

    // Check capabilities
    let caps = provider.capabilities();
    assert!(caps.has_feeds);
    assert!(!caps.has_collections);
    assert!(!caps.has_saved_items);
    assert!(!caps.has_communities);
}

#[tokio::test]
async fn test_registry_provider_sync() {
    let mut registry = ProviderRegistry::new();
    registry.register(DummyProvider::new());

    let provider = registry.get("dummy").unwrap();

    // Sync should succeed
    let result = provider.sync().await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.success);
    assert_eq!(result.errors.len(), 0);
}

#[test]
fn test_registry_multiple_providers() {
    let mut registry = ProviderRegistry::new();

    // Register multiple instances (in a real scenario, these would be different providers)
    registry.register(DummyProvider::new());

    // Try to get a non-existent provider
    let nonexistent = registry.get("nonexistent");
    assert!(nonexistent.is_none());

    // Verify dummy is still accessible
    let dummy = registry.get("dummy");
    assert!(dummy.is_some());
}

#[test]
fn test_registry_remove_provider() {
    let mut registry = ProviderRegistry::new();
    registry.register(DummyProvider::new());

    assert_eq!(registry.count(), 1);

    // Remove the provider
    let removed = registry.remove("dummy");
    assert!(removed.is_some());

    // Verify removal
    assert_eq!(registry.count(), 0);
    assert!(!registry.contains("dummy"));
}

#[test]
fn test_registry_clear() {
    let mut registry = ProviderRegistry::new();
    registry.register(DummyProvider::new());

    assert_eq!(registry.count(), 1);

    // Clear all providers
    registry.clear();

    // Verify cleared
    assert_eq!(registry.count(), 0);
    assert!(registry.list().is_empty());
}
