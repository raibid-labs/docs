//! Error types for theme system

use thiserror::Error;

/// Result type for theme operations
pub type ThemeResult<T> = Result<T, ThemeError>;

/// Theme system error types
#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("Theme not found: {0}")]
    NotFound(String),

    #[error("Invalid theme format: {0}")]
    InvalidFormat(String),

    #[error("Invalid color format: {0}")]
    InvalidColor(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Plugin error: {0}")]
    Plugin(#[from] scarab_plugin_api::PluginError),
}
