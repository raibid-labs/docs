//! OAuth provider configuration and registry.
//!
//! This module provides:
//! - [`ProviderConfig`] - Configuration for an OAuth provider
//! - [`ProviderRegistry`] - Registry of configured OAuth providers
//!
//! The registry comes pre-configured with common providers (GitHub, Spotify, Google)
//! and can be extended with custom providers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for an OAuth provider.
///
/// Defines the endpoints and capabilities of an OAuth provider.
///
/// # Example
///
/// ```
/// use sigilforge_core::provider::ProviderConfig;
///
/// let github = ProviderConfig {
///     id: "github".to_string(),
///     name: "GitHub".to_string(),
///     auth_url: "https://github.com/login/oauth/authorize".to_string(),
///     token_url: "https://github.com/login/oauth/access_token".to_string(),
///     revoke_url: None,
///     default_scopes: vec!["repo".to_string(), "user".to_string()],
///     supports_pkce: true,
///     supports_device_code: true,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderConfig {
    /// Unique identifier for the provider (e.g., "github", "spotify").
    pub id: String,

    /// Human-readable name (e.g., "GitHub", "Spotify").
    pub name: String,

    /// OAuth authorization endpoint URL.
    pub auth_url: String,

    /// OAuth token endpoint URL.
    pub token_url: String,

    /// Optional token revocation endpoint URL.
    pub revoke_url: Option<String>,

    /// Default OAuth scopes to request.
    pub default_scopes: Vec<String>,

    /// Whether this provider supports PKCE (Proof Key for Code Exchange).
    pub supports_pkce: bool,

    /// Whether this provider supports the device code flow.
    pub supports_device_code: bool,
}

impl ProviderConfig {
    /// Create a new provider configuration.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            auth_url: String::new(),
            token_url: String::new(),
            revoke_url: None,
            default_scopes: Vec::new(),
            supports_pkce: false,
            supports_device_code: false,
        }
    }

    /// Set the authorization URL.
    pub fn with_auth_url(mut self, url: impl Into<String>) -> Self {
        self.auth_url = url.into();
        self
    }

    /// Set the token URL.
    pub fn with_token_url(mut self, url: impl Into<String>) -> Self {
        self.token_url = url.into();
        self
    }

    /// Set the revocation URL.
    pub fn with_revoke_url(mut self, url: impl Into<String>) -> Self {
        self.revoke_url = Some(url.into());
        self
    }

    /// Set the default scopes.
    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.default_scopes = scopes;
        self
    }

    /// Enable PKCE support.
    pub fn with_pkce(mut self, enabled: bool) -> Self {
        self.supports_pkce = enabled;
        self
    }

    /// Enable device code flow support.
    pub fn with_device_code(mut self, enabled: bool) -> Self {
        self.supports_device_code = enabled;
        self
    }
}

/// Registry of OAuth provider configurations.
///
/// Maintains a mapping of provider IDs to their configurations.
/// Can be initialized with default providers or built from scratch.
///
/// # Example
///
/// ```
/// use sigilforge_core::provider::ProviderRegistry;
///
/// // Start with default providers
/// let registry = ProviderRegistry::with_defaults();
///
/// // Get a provider configuration
/// let github = registry.get("github").unwrap();
/// assert_eq!(github.name, "GitHub");
/// ```
#[derive(Debug, Clone)]
pub struct ProviderRegistry {
    providers: HashMap<String, ProviderConfig>,
}

impl ProviderRegistry {
    /// Create a new empty provider registry.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Create a provider registry with default providers pre-registered.
    ///
    /// Default providers include:
    /// - GitHub
    /// - Spotify
    /// - Google
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // GitHub configuration
        registry.register(ProviderConfig {
            id: "github".to_string(),
            name: "GitHub".to_string(),
            auth_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            revoke_url: Some("https://api.github.com/applications/{client_id}/token".to_string()),
            default_scopes: vec!["repo".to_string(), "user".to_string()],
            supports_pkce: true,
            supports_device_code: true,
        });

        // Spotify configuration
        registry.register(ProviderConfig {
            id: "spotify".to_string(),
            name: "Spotify".to_string(),
            auth_url: "https://accounts.spotify.com/authorize".to_string(),
            token_url: "https://accounts.spotify.com/api/token".to_string(),
            revoke_url: None,
            default_scopes: vec![
                "user-read-private".to_string(),
                "user-read-email".to_string(),
            ],
            supports_pkce: true,
            supports_device_code: false,
        });

        // Google configuration
        registry.register(ProviderConfig {
            id: "google".to_string(),
            name: "Google".to_string(),
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            revoke_url: Some("https://oauth2.googleapis.com/revoke".to_string()),
            default_scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
            supports_pkce: true,
            supports_device_code: true,
        });

        registry
    }

    /// Register a new provider configuration.
    ///
    /// If a provider with the same ID already exists, it will be replaced.
    pub fn register(&mut self, config: ProviderConfig) {
        self.providers.insert(config.id.clone(), config);
    }

    /// Get a provider configuration by ID.
    ///
    /// Returns `None` if the provider is not registered.
    pub fn get(&self, id: &str) -> Option<&ProviderConfig> {
        self.providers.get(id)
    }

    /// Get a mutable reference to a provider configuration by ID.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut ProviderConfig> {
        self.providers.get_mut(id)
    }

    /// Check if a provider is registered.
    pub fn contains(&self, id: &str) -> bool {
        self.providers.contains_key(id)
    }

    /// List all registered provider IDs.
    pub fn list_ids(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    /// Remove a provider from the registry.
    ///
    /// Returns the removed provider configuration, or `None` if it didn't exist.
    pub fn remove(&mut self, id: &str) -> Option<ProviderConfig> {
        self.providers.remove(id)
    }

    /// Get the number of registered providers.
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_builder() {
        let config = ProviderConfig::new("test", "Test Provider")
            .with_auth_url("https://example.com/auth")
            .with_token_url("https://example.com/token")
            .with_scopes(vec!["read".to_string()])
            .with_pkce(true);

        assert_eq!(config.id, "test");
        assert_eq!(config.name, "Test Provider");
        assert_eq!(config.auth_url, "https://example.com/auth");
        assert_eq!(config.token_url, "https://example.com/token");
        assert_eq!(config.default_scopes, vec!["read"]);
        assert!(config.supports_pkce);
    }

    #[test]
    fn test_provider_registry_new() {
        let registry = ProviderRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_provider_registry_with_defaults() {
        let registry = ProviderRegistry::with_defaults();

        assert!(!registry.is_empty());
        assert!(registry.contains("github"));
        assert!(registry.contains("spotify"));
        assert!(registry.contains("google"));

        let github = registry.get("github").unwrap();
        assert_eq!(github.name, "GitHub");
        assert!(github.supports_pkce);
        assert!(github.supports_device_code);
    }

    #[test]
    fn test_provider_registry_register_and_get() {
        let mut registry = ProviderRegistry::new();

        let config = ProviderConfig::new("test", "Test");
        registry.register(config.clone());

        let retrieved = registry.get("test").unwrap();
        assert_eq!(retrieved.id, "test");
        assert_eq!(retrieved.name, "Test");
    }

    #[test]
    fn test_provider_registry_replace() {
        let mut registry = ProviderRegistry::new();

        registry.register(ProviderConfig::new("test", "Test 1"));
        registry.register(ProviderConfig::new("test", "Test 2"));

        let config = registry.get("test").unwrap();
        assert_eq!(config.name, "Test 2");
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_provider_registry_remove() {
        let mut registry = ProviderRegistry::with_defaults();

        let removed = registry.remove("github");
        assert!(removed.is_some());
        assert!(!registry.contains("github"));

        let not_found = registry.remove("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_provider_registry_list_ids() {
        let registry = ProviderRegistry::with_defaults();
        let ids = registry.list_ids();

        assert!(ids.contains(&"github"));
        assert!(ids.contains(&"spotify"));
        assert!(ids.contains(&"google"));
    }
}
