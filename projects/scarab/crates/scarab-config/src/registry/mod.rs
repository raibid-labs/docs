//! Plugin Registry and Marketplace System
//!
//! Provides a complete plugin marketplace with:
//! - Remote registry synchronization
//! - Local caching (~/.config/scarab/registry/)
//! - Plugin discovery and installation
//! - Version management and updates
//! - Security verification (SHA256, GPG signatures)
//! - Plugin ratings and metadata

pub mod cache;
pub mod client;
pub mod installer;
pub mod manifest;
pub mod security;
pub mod types;

pub use cache::RegistryCache;
pub use client::RegistryClient;
pub use installer::PluginInstaller;
pub use manifest::RegistryManifest;
pub use security::PluginVerifier;
pub use types::{
    InstalledPlugin, PluginEntry, PluginFilter, PluginRating, PluginStats, RegistryConfig,
    SecurityConfig,
};

use crate::error::Result;
use std::path::PathBuf;

/// Main registry manager combining all functionality
pub struct RegistryManager {
    pub client: RegistryClient,
    pub cache: RegistryCache,
    pub installer: PluginInstaller,
    verifier: PluginVerifier,
}

impl RegistryManager {
    /// Create new registry manager with default configuration
    pub fn new() -> Result<Self> {
        let config = RegistryConfig::default();
        Self::with_config(config)
    }

    /// Create registry manager with custom configuration
    pub fn with_config(config: RegistryConfig) -> Result<Self> {
        let cache = RegistryCache::new(config.cache_dir.clone())?;
        let client = RegistryClient::new(config.registry_url.clone());
        let installer = PluginInstaller::new(config.plugin_dir.clone())?;
        let verifier = PluginVerifier::new(config.security.clone());

        Ok(Self {
            client,
            cache,
            installer,
            verifier,
        })
    }

    /// Synchronize with remote registry
    pub async fn sync(&mut self) -> Result<()> {
        let manifest = self.client.fetch_manifest().await?;
        self.cache.update_manifest(manifest)?;
        Ok(())
    }

    /// Search for plugins matching filter
    pub fn search(&self, filter: &PluginFilter) -> Result<Vec<PluginEntry>> {
        self.cache.search(filter)
    }

    /// Get plugin details by name
    pub fn get_plugin(&self, name: &str) -> Result<Option<PluginEntry>> {
        self.cache.get_plugin(name)
    }

    /// Install plugin from registry
    pub async fn install(
        &mut self,
        name: &str,
        version: Option<&str>,
    ) -> Result<InstalledPlugin> {
        // Get plugin entry
        let entry = self
            .cache
            .get_plugin(name)?
            .ok_or_else(|| crate::error::ConfigError::NotFound(name.to_string()))?;

        // Determine version to install
        let version = version.unwrap_or(&entry.latest_version);

        // Download plugin
        let download = self.client.download_plugin(name, version).await?;

        // Verify signature and checksum
        self.verifier
            .verify(&download.content, &entry, version)?;

        // Install plugin
        let installed = self.installer.install(name, version, download.content)?;

        Ok(installed)
    }

    /// Update installed plugin to latest version
    pub async fn update(&mut self, name: &str) -> Result<InstalledPlugin> {
        let installed = self.installer.get_installed(name)?;
        let entry = self
            .cache
            .get_plugin(name)?
            .ok_or_else(|| crate::error::ConfigError::NotFound(name.to_string()))?;

        // Check if update available
        if installed.version == entry.latest_version {
            return Ok(installed);
        }

        // Install latest version
        self.install(name, Some(&entry.latest_version)).await
    }

    /// Remove installed plugin
    pub fn remove(&mut self, name: &str) -> Result<()> {
        self.installer.remove(name)
    }

    /// List all installed plugins
    pub fn list_installed(&self) -> Result<Vec<InstalledPlugin>> {
        self.installer.list_installed()
    }

    /// Check for updates to installed plugins
    pub fn check_updates(&self) -> Result<Vec<(String, String, String)>> {
        let installed = self.installer.list_installed()?;
        let mut updates = Vec::new();

        for plugin in installed {
            if let Ok(Some(entry)) = self.cache.get_plugin(&plugin.name) {
                if plugin.version != entry.latest_version {
                    updates.push((
                        plugin.name.clone(),
                        plugin.version.clone(),
                        entry.latest_version.clone(),
                    ));
                }
            }
        }

        Ok(updates)
    }

    /// Get plugin statistics
    pub fn get_stats(&self, name: &str) -> Result<Option<PluginStats>> {
        Ok(self.cache.get_plugin(name)?.map(|e| e.stats))
    }

    /// Get default cache directory
    pub fn default_cache_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join("scarab")
            .join("registry")
    }

    /// Get default plugin directory
    pub fn default_plugin_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join("scarab")
            .join("plugins")
    }
}

impl Default for RegistryManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default registry manager")
    }
}
