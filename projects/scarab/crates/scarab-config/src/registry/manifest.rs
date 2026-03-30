//! Registry manifest handling

use super::types::PluginEntry;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Registry manifest containing all available plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryManifest {
    /// Schema version
    pub version: String,
    /// Registry metadata
    pub metadata: RegistryMetadata,
    /// All available plugins indexed by name
    pub plugins: HashMap<String, PluginEntry>,
    /// Last update timestamp
    pub updated_at: u64,
}

impl RegistryManifest {
    /// Create new empty manifest
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            metadata: RegistryMetadata::default(),
            plugins: HashMap::new(),
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Load manifest from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Serialize manifest to JSON
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Get plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&PluginEntry> {
        self.plugins.get(name)
    }

    /// Add or update plugin entry
    pub fn upsert_plugin(&mut self, plugin: PluginEntry) {
        self.plugins.insert(plugin.name.clone(), plugin);
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Remove plugin by name
    pub fn remove_plugin(&mut self, name: &str) -> Option<PluginEntry> {
        let result = self.plugins.remove(name);
        if result.is_some() {
            self.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        result
    }

    /// Get all plugins as sorted vector
    pub fn all_plugins(&self) -> Vec<&PluginEntry> {
        let mut plugins: Vec<_> = self.plugins.values().collect();
        plugins.sort_by(|a, b| a.name.cmp(&b.name));
        plugins
    }

    /// Get total plugin count
    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}

impl Default for RegistryManifest {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMetadata {
    /// Registry name
    pub name: String,
    /// Registry description
    pub description: String,
    /// Registry URL
    pub url: String,
    /// Maintainer contact
    pub maintainer: Option<String>,
}

impl Default for RegistryMetadata {
    fn default() -> Self {
        Self {
            name: "Scarab Official Registry".to_string(),
            description: "Official plugin registry for Scarab terminal emulator".to_string(),
            url: "https://registry.scarab.dev".to_string(),
            maintainer: Some("registry@scarab.dev".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::types::{PluginStats, PluginVersion};

    #[test]
    fn test_manifest_creation() {
        let manifest = RegistryManifest::new();
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.plugins.len(), 0);
    }

    #[test]
    fn test_manifest_json_roundtrip() {
        let mut manifest = RegistryManifest::new();

        let plugin = PluginEntry {
            name: "test-plugin".to_string(),
            description: "Test plugin".to_string(),
            readme: None,
            author: "Test Author".to_string(),
            author_email: None,
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            latest_version: "1.0.0".to_string(),
            versions: vec![],
            tags: vec!["test".to_string()],
            stats: PluginStats::default(),
            created_at: 0,
            updated_at: 0,
        };

        manifest.upsert_plugin(plugin);

        let json = manifest.to_json().unwrap();
        let decoded = RegistryManifest::from_json(&json).unwrap();

        assert_eq!(decoded.plugins.len(), 1);
        assert!(decoded.get_plugin("test-plugin").is_some());
    }
}
