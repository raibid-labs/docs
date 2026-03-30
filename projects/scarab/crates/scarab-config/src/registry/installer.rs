//! Plugin installation and management

use super::types::InstalledPlugin;
use crate::error::{ConfigError, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Plugin installer and manager
pub struct PluginInstaller {
    /// Plugin installation directory
    plugin_dir: PathBuf,
    /// Installed plugins index
    index: InstalledPluginsIndex,
}

impl PluginInstaller {
    /// Create new installer
    pub fn new(plugin_dir: PathBuf) -> Result<Self> {
        // Ensure plugin directory exists
        if !plugin_dir.exists() {
            fs::create_dir_all(&plugin_dir)?;
        }

        // Load or create index
        let index_path = plugin_dir.join("installed.json");
        let index = if index_path.exists() {
            let content = fs::read_to_string(&index_path)?;
            serde_json::from_str(&content)?
        } else {
            InstalledPluginsIndex::new()
        };

        Ok(Self { plugin_dir, index })
    }

    /// Install plugin from content
    pub fn install(&mut self, name: &str, version: &str, content: Vec<u8>) -> Result<InstalledPlugin> {
        // Determine file extension based on content
        let extension = if content.starts_with(b"FZB\x00") {
            "fzb"
        } else {
            "fsx"
        };

        // Create plugin directory
        let plugin_path = self.plugin_dir.join(name);
        if !plugin_path.exists() {
            fs::create_dir_all(&plugin_path)?;
        }

        // Write plugin file
        let file_path = plugin_path.join(format!("{}.{}", name, extension));
        fs::write(&file_path, content)?;

        // Create installed plugin entry
        let installed = InstalledPlugin {
            name: name.to_string(),
            version: version.to_string(),
            path: file_path,
            installed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            enabled: true,
            config: HashMap::new(),
        };

        // Update index
        self.index.plugins.insert(name.to_string(), installed.clone());
        self.save_index()?;

        Ok(installed)
    }

    /// Remove installed plugin
    pub fn remove(&mut self, name: &str) -> Result<()> {
        let plugin = self.index.plugins.remove(name).ok_or_else(|| {
            ConfigError::NotFound(format!("Plugin '{}' not installed", name))
        })?;

        // Remove plugin directory
        if let Some(parent) = plugin.path.parent() {
            if parent.exists() {
                fs::remove_dir_all(parent)?;
            }
        }

        self.save_index()?;
        Ok(())
    }

    /// Get installed plugin info
    pub fn get_installed(&self, name: &str) -> Result<InstalledPlugin> {
        self.index
            .plugins
            .get(name)
            .cloned()
            .ok_or_else(|| ConfigError::NotFound(format!("Plugin '{}' not installed", name)))
    }

    /// List all installed plugins
    pub fn list_installed(&self) -> Result<Vec<InstalledPlugin>> {
        Ok(self.index.plugins.values().cloned().collect())
    }

    /// Enable plugin
    pub fn enable(&mut self, name: &str) -> Result<()> {
        let plugin = self.index.plugins.get_mut(name).ok_or_else(|| {
            ConfigError::NotFound(format!("Plugin '{}' not installed", name))
        })?;

        plugin.enabled = true;
        self.save_index()?;
        Ok(())
    }

    /// Disable plugin
    pub fn disable(&mut self, name: &str) -> Result<()> {
        let plugin = self.index.plugins.get_mut(name).ok_or_else(|| {
            ConfigError::NotFound(format!("Plugin '{}' not installed", name))
        })?;

        plugin.enabled = false;
        self.save_index()?;
        Ok(())
    }

    /// Update plugin configuration
    pub fn set_config(
        &mut self,
        name: &str,
        config: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let plugin = self.index.plugins.get_mut(name).ok_or_else(|| {
            ConfigError::NotFound(format!("Plugin '{}' not installed", name))
        })?;

        plugin.config = config;
        self.save_index()?;
        Ok(())
    }

    /// Save index to disk
    fn save_index(&self) -> Result<()> {
        let index_path = self.plugin_dir.join("installed.json");
        let json = serde_json::to_string_pretty(&self.index)?;
        fs::write(index_path, json)?;
        Ok(())
    }

    /// Get plugin directory path
    pub fn plugin_dir(&self) -> &Path {
        &self.plugin_dir
    }
}

/// Index of installed plugins
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct InstalledPluginsIndex {
    /// Schema version
    version: String,
    /// Installed plugins indexed by name
    plugins: HashMap<String, InstalledPlugin>,
}

impl InstalledPluginsIndex {
    fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            plugins: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_install_and_remove() {
        let temp_dir = TempDir::new().unwrap();
        let mut installer = PluginInstaller::new(temp_dir.path().to_path_buf()).unwrap();

        // Install plugin
        let content = b"module TestPlugin\nlet version = 1.0".to_vec();
        let installed = installer.install("test-plugin", "1.0.0", content).unwrap();

        assert_eq!(installed.name, "test-plugin");
        assert_eq!(installed.version, "1.0.0");
        assert!(installed.enabled);

        // Verify plugin file exists
        assert!(installed.path.exists());

        // List installed
        let list = installer.list_installed().unwrap();
        assert_eq!(list.len(), 1);

        // Remove plugin
        installer.remove("test-plugin").unwrap();
        assert!(installer.get_installed("test-plugin").is_err());
    }

    #[test]
    fn test_enable_disable() {
        let temp_dir = TempDir::new().unwrap();
        let mut installer = PluginInstaller::new(temp_dir.path().to_path_buf()).unwrap();

        let content = b"module TestPlugin".to_vec();
        installer.install("test-plugin", "1.0.0", content).unwrap();

        // Disable
        installer.disable("test-plugin").unwrap();
        let plugin = installer.get_installed("test-plugin").unwrap();
        assert!(!plugin.enabled);

        // Enable
        installer.enable("test-plugin").unwrap();
        let plugin = installer.get_installed("test-plugin").unwrap();
        assert!(plugin.enabled);
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().to_path_buf();

        // Install in first instance
        {
            let mut installer = PluginInstaller::new(plugin_dir.clone()).unwrap();
            let content = b"module TestPlugin".to_vec();
            installer.install("test-plugin", "1.0.0", content).unwrap();
        }

        // Verify persistence in new instance
        {
            let installer = PluginInstaller::new(plugin_dir).unwrap();
            let plugin = installer.get_installed("test-plugin").unwrap();
            assert_eq!(plugin.name, "test-plugin");
        }
    }
}
