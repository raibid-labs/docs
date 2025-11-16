//! Configuration management for raibid-core

use crate::error::{Error, Result};
use crate::types::FilterCriteria;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General settings
    pub general: GeneralConfig,

    /// Sync settings
    pub sync: SyncConfig,

    /// Filter settings
    pub filter: FilterCriteria,

    /// TUI settings
    pub tui: TuiConfig,

    /// Git settings
    pub git: GitConfig,

    /// GitHub settings
    pub github: GitHubConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            sync: SyncConfig::default(),
            filter: FilterCriteria::default(),
            tui: TuiConfig::default(),
            git: GitConfig::default(),
            github: GitHubConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| Error::Config(format!("Failed to parse config: {}", e)))?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get default config path
    pub fn default_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| Error::Config("Could not determine config directory".to_string()))?;
        Ok(config_dir.join("raibid-cli").join("config.toml"))
    }

    /// Load config from default path or create default
    pub fn load_or_default() -> Result<Self> {
        let path = Self::default_path()?;
        if path.exists() {
            Self::load(&path)
        } else {
            Ok(Self::default())
        }
    }
}

/// General configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// GitHub organization name
    pub org: String,

    /// Workspace root directory
    pub workspace_root: PathBuf,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            org: "raibid-labs".to_string(),
            workspace_root: home.join("raibid-labs"),
        }
    }
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Number of concurrent operations
    pub concurrency: usize,

    /// Automatically pull updates
    pub auto_pull: bool,

    /// Check for uncommitted changes before pulling
    pub check_uncommitted: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            concurrency: 5,
            auto_pull: true,
            check_uncommitted: true,
        }
    }
}

/// TUI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    /// Key bindings
    pub keys: KeyBindings,

    /// Show repository descriptions
    pub show_descriptions: bool,

    /// Compact mode (less spacing)
    pub compact_mode: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            keys: KeyBindings::default(),
            show_descriptions: true,
            compact_mode: false,
        }
    }
}

/// Key bindings for TUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub quit: String,
    pub sync: String,
    pub help: String,
    pub up: String,
    pub down: String,
    pub page_up: String,
    pub page_down: String,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            quit: "q".to_string(),
            sync: "s".to_string(),
            help: "?".to_string(),
            up: "k".to_string(),
            down: "j".to_string(),
            page_up: "u".to_string(),
            page_down: "d".to_string(),
        }
    }
}

/// Git configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// Use SSH for authentication
    pub ssh_auth: bool,

    /// Clone depth (0 = full clone)
    pub depth: u32,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            ssh_auth: true,
            depth: 0,
        }
    }
}

/// GitHub configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// GitHub API token (optional, usually from gh cli)
    pub token: Option<String>,

    /// API base URL
    pub api_url: String,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            token: None,
            api_url: "https://api.github.com".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.org, "raibid-labs");
        assert_eq!(config.sync.concurrency, 5);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.general.org, config.general.org);
    }
}
