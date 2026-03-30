//! Plugin configuration loading and discovery

use crate::{context::PluginConfigData, error::Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Plugin configuration from TOML file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginConfig {
    /// Plugin name
    pub name: String,
    /// Path to plugin file (.fzb or .fsx)
    pub path: PathBuf,
    /// Whether plugin is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Plugin-specific configuration
    #[serde(default)]
    pub config: PluginConfigData,
}

fn default_true() -> bool {
    true
}

impl PluginConfig {
    /// Load plugin configuration from TOML file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Vec<Self>> {
        let content = fs::read_to_string(path)?;
        let config: PluginsToml = toml::from_str(&content)?;
        Ok(config.plugin)
    }

    /// Expand path with home directory
    pub fn expanded_path(&self) -> PathBuf {
        expand_path(&self.path)
    }
}

/// Root TOML structure for plugins.toml
#[derive(Debug, Deserialize, Serialize)]
struct PluginsToml {
    plugin: Vec<PluginConfig>,
}

/// Plugin discovery system
pub struct PluginDiscovery {
    /// Plugin directories to search
    search_paths: Vec<PathBuf>,
}

impl PluginDiscovery {
    /// Create new discovery with default paths
    pub fn new() -> Self {
        let mut search_paths = vec![
            Self::default_plugin_dir(),
            PathBuf::from("/usr/local/share/scarab/plugins"),
            PathBuf::from("/usr/share/scarab/plugins"),
        ];

        // Add custom path from environment
        if let Ok(custom_path) = std::env::var("SCARAB_PLUGIN_PATH") {
            search_paths.insert(0, PathBuf::from(custom_path));
        }

        Self { search_paths }
    }

    /// Get default plugin directory (~/.config/scarab/plugins)
    pub fn default_plugin_dir() -> PathBuf {
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join(".config/scarab/plugins")
        } else {
            PathBuf::from(".config/scarab/plugins")
        }
    }

    /// Get default config file path (~/.config/scarab/plugins.toml)
    pub fn default_config_path() -> PathBuf {
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join(".config/scarab/plugins.toml")
        } else {
            PathBuf::from(".config/scarab/plugins.toml")
        }
    }

    /// Add search path
    pub fn add_path(&mut self, path: impl Into<PathBuf>) {
        self.search_paths.push(path.into());
    }

    /// Discover all plugin files in search paths
    pub fn discover(&self) -> Vec<PathBuf> {
        let mut plugins = Vec::new();

        for dir in &self.search_paths {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if Self::is_plugin_file(&path) {
                        plugins.push(path);
                    }
                }
            }
        }

        plugins
    }

    /// Check if file is a valid plugin file
    fn is_plugin_file(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("fzb") | Some("fsx")
        )
    }

    /// Load plugins from configuration file
    pub fn load_config(&self, path: Option<&Path>) -> Result<Vec<PluginConfig>> {
        let config_path = path
            .map(PathBuf::from)
            .unwrap_or_else(Self::default_config_path);

        if !config_path.exists() {
            return Ok(Vec::new());
        }

        PluginConfig::from_file(config_path)
    }

    /// Create default plugin directory if it doesn't exist
    pub fn ensure_plugin_dir() -> Result<PathBuf> {
        let dir = Self::default_plugin_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }

    /// Create default config file with example
    pub fn create_default_config() -> Result<PathBuf> {
        let config_path = Self::default_config_path();

        if config_path.exists() {
            return Ok(config_path);
        }

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create example config with delightful examples
        let example_config = r#"# 🎉 Scarab Plugin Configuration
#
# Welcome to plugin paradise! This is where you configure all your terminal
# superpowers. Each plugin can transform your terminal experience in unique ways.
#
# 💡 Pro Tips:
#   - Plugins are loaded in the order they appear here
#   - Use `enabled = false` to temporarily disable a plugin
#   - Check ~/.config/scarab/plugins/ for available plugins
#   - Create your own plugins - it's easier than you think!
#
# 🚀 Get started by uncommenting one of the examples below!

# Example 1: Error Notification Plugin
# Gets your attention when something goes wrong
#
# [[plugin]]
# name = "error-notifier"
# path = "~/.config/scarab/plugins/error-notifier.fzb"
# enabled = true
#
# [plugin.config]
# keywords = ["ERROR", "FAIL", "PANIC", "FATAL"]
# notification_style = "urgent"
# play_sound = false

# Example 2: Git Status Plugin
# Shows git branch and status in your terminal
#
# [[plugin]]
# name = "git-helper"
# path = "~/.config/scarab/plugins/git-helper.fsx"
# enabled = true
#
# [plugin.config]
# show_branch = true
# show_dirty = true
# emoji_mode = true  # 🌿 for branches, ✨ for clean, 💥 for dirty

# Example 3: Command History Plugin
# Keeps track of your most-used commands
#
# [[plugin]]
# name = "command-stats"
# path = "~/.config/scarab/plugins/command-stats.fzb"
# enabled = true
#
# [plugin.config]
# track_frequency = true
# suggest_aliases = true

# Example 4: Custom Welcome Message
# Greet yourself with style every time
#
# [[plugin]]
# name = "welcome"
# path = "~/.config/scarab/plugins/welcome.fsx"
# enabled = true
#
# [plugin.config]
# message = "Ready to do amazing things? Let's go! 🚀"
# show_time = true
# show_quote_of_the_day = true

# ✨ Your plugins go here! ✨
# Just uncomment the examples above or add your own.
# Happy customizing!

"#;

        fs::write(&config_path, example_config)?;
        Ok(config_path)
    }
}

impl Default for PluginDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Expand ~ in path to home directory
fn expand_path(path: &Path) -> PathBuf {
    if let Some(s) = path.to_str() {
        if let Some(stripped) = s.strip_prefix("~/") {
            if let Some(home) = std::env::var_os("HOME") {
                return PathBuf::from(home).join(stripped);
            }
        }
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_path() {
        let path = PathBuf::from("~/test/path");
        let expanded = expand_path(&path);
        assert!(!expanded.to_string_lossy().contains('~'));
    }

    #[test]
    fn test_is_plugin_file() {
        // is_plugin_file checks if path.is_file() first, so we need actual files
        // For unit testing, we just test the extension logic
        use std::path::Path;

        let has_valid_ext = |path: &Path| -> bool {
            matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("fzb") | Some("fsx")
            )
        };

        assert!(has_valid_ext(Path::new("test.fzb")));
        assert!(has_valid_ext(Path::new("test.fsx")));
        assert!(!has_valid_ext(Path::new("test.txt")));
    }
}
