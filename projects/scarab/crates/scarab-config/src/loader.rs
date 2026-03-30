//! Configuration file loading and discovery

use crate::{error::Result, ConfigError, ScarabConfig};
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use tracing::{debug, info};

/// Configuration loader with discovery
pub struct ConfigLoader {
    global_path: PathBuf,
}

impl ConfigLoader {
    /// Create a new loader with default paths
    pub fn new() -> Self {
        Self {
            global_path: Self::default_config_path(),
        }
    }

    /// Create a loader with custom global path (for testing)
    pub fn with_path(path: PathBuf) -> Self {
        Self { global_path: path }
    }

    /// Get default global config path (~/.config/scarab/config.toml)
    pub fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("scarab")
            .join("config.toml")
    }

    /// Load configuration with global + local merging
    pub fn load(&self) -> Result<ScarabConfig> {
        let mut config = self.load_global()?;

        if let Some(local) = self.load_local()? {
            debug!("Found local config, merging with global");
            config.merge(local);
        }

        Ok(config)
    }

    /// Load global configuration
    fn load_global(&self) -> Result<ScarabConfig> {
        if self.global_path.exists() {
            info!("Loading global config from: {}", self.global_path.display());
            Self::from_file(&self.global_path)
        } else {
            debug!("No global config found, using defaults");
            Ok(ScarabConfig::default())
        }
    }

    /// Load local configuration by walking up from cwd
    fn load_local(&self) -> Result<Option<ScarabConfig>> {
        let mut current = env::current_dir()?;

        loop {
            let config_path = current.join(".scarab.toml");

            if config_path.exists() {
                info!("Found local config at: {}", config_path.display());
                return Ok(Some(Self::from_file(&config_path)?));
            }

            // Try to go up one directory
            if !current.pop() {
                break;
            }
        }

        debug!("No local config found in directory tree");
        Ok(None)
    }

    /// Load config from a specific file
    pub fn from_file(path: &Path) -> Result<ScarabConfig> {
        let content =
            fs::read_to_string(path).map_err(|_| ConfigError::FileNotFound(path.display().to_string()))?;

        let config: ScarabConfig = toml::from_str(&content)?;
        debug!("Loaded config from: {}", path.display());
        Ok(config)
    }

    /// Save config to global config file
    pub fn save_global(&self, config: &ScarabConfig) -> Result<()> {
        self.save_to(&self.global_path, config)
    }

    /// Save config to a specific file
    pub fn save_to(&self, path: &Path, config: &ScarabConfig) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml = toml::to_string_pretty(config)?;
        fs::write(path, toml)?;
        info!("Saved config to: {}", path.display());
        Ok(())
    }

    /// Create default config file if it doesn't exist
    pub fn ensure_default_config(&self) -> Result<PathBuf> {
        if self.global_path.exists() {
            debug!("Global config already exists");
            return Ok(self.global_path.clone());
        }

        info!("Creating default config at: {}", self.global_path.display());

        // Ensure directory exists
        if let Some(parent) = self.global_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let default_config = ScarabConfig::default();
        self.save_global(&default_config)?;

        Ok(self.global_path.clone())
    }

    /// Get all config locations (for debugging)
    pub fn config_locations(&self) -> Vec<(String, PathBuf, bool)> {
        let mut locations = vec![(
            "Global".to_string(),
            self.global_path.clone(),
            self.global_path.exists(),
        )];

        // Walk up from cwd
        if let Ok(mut current) = env::current_dir() {
            loop {
                let local_path = current.join(".scarab.toml");
                if local_path.exists() {
                    locations.push((format!("Local ({})", current.display()), local_path, true));
                }

                if !current.pop() {
                    break;
                }
            }
        }

        locations
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config_path() {
        let path = ConfigLoader::default_config_path();
        assert!(path.to_string_lossy().contains("scarab"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }

    #[test]
    fn test_load_default_config() {
        let loader = ConfigLoader::new();
        // Should not fail even if file doesn't exist
        let config = loader.load_global().unwrap();
        assert_eq!(config.font.size, 14.0);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut loader = ConfigLoader::new();
        loader.global_path = config_path.clone();

        let mut config = ScarabConfig::default();
        config.font.size = 18.0;

        loader.save_global(&config).unwrap();
        assert!(config_path.exists());

        let loaded = ConfigLoader::from_file(&config_path).unwrap();
        assert_eq!(loaded.font.size, 18.0);
    }

    #[test]
    fn test_ensure_default_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("scarab/config.toml");

        let mut loader = ConfigLoader::new();
        loader.global_path = config_path.clone();

        let path = loader.ensure_default_config().unwrap();
        assert!(path.exists());
        assert_eq!(path, config_path);
    }
}
