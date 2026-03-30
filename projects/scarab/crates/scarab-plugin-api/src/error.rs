//! Error types for plugin operations

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin failed to load: {0}")]
    LoadError(String),

    #[error("Plugin version incompatible: required {required}, got {actual}")]
    VersionIncompatible { required: String, actual: String },

    #[error("Plugin hook execution timed out after {0}ms")]
    Timeout(u64),

    #[error("Plugin panic: {0}")]
    Panic(String),

    #[error("Plugin disabled due to repeated failures")]
    Disabled,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("Invalid plugin metadata: {0}")]
    InvalidMetadata(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

impl PluginError {
    /// Get friendly suggestions for fixing this error
    pub fn suggestions(&self) -> Vec<&'static str> {
        match self {
            PluginError::LoadError(_) => vec![
                "Check that the plugin file exists and has the correct extension (.fzb or .fsx)",
                "Verify the plugin file isn't corrupted",
                "Make sure you have read permissions for the plugin file",
                "Try loading the plugin with the full absolute path",
            ],
            PluginError::VersionIncompatible { .. } => vec![
                "Update the plugin to match the current API version",
                "Check if a newer version of the plugin is available",
                "Update Scarab to support the plugin's API version",
                "Contact the plugin author about compatibility",
            ],
            PluginError::Timeout(_) => vec![
                "The plugin might be doing too much work in its hooks",
                "Try optimizing the plugin's hook implementations",
                "Check if the plugin is waiting on external resources",
                "Increase the timeout with --timeout flag if needed",
            ],
            PluginError::ConfigError(_) => vec![
                "Check your plugins.toml file for syntax errors",
                "Verify all required configuration fields are present",
                "Make sure the configuration values have the correct types",
                "Look at example configurations in the docs",
            ],
            PluginError::NotFound(_) => vec![
                "Double-check the file path in your configuration",
                "Make sure the plugin file is in one of the search paths",
                "Use an absolute path instead of a relative one",
                "Check if ~ expansion is working correctly",
            ],
            PluginError::Disabled => vec![
                "Check the plugin logs for recurring errors",
                "The plugin exceeded the maximum failure count",
                "Try fixing the plugin and restarting Scarab",
                "You can re-enable it in plugins.toml once fixed",
            ],
            _ => vec!["Check the plugin documentation for more help"],
        }
    }

    /// Format error with friendly prefix and suggestions
    pub fn friendly_message(&self) -> String {
        use crate::delight;
        delight::friendly_error(&self.to_string(), self.suggestions())
    }
}

pub type Result<T> = std::result::Result<T, PluginError>;
