//! Error types for the configuration system

use std::io;
use thiserror::Error;

/// Configuration error type
#[derive(Debug, Error)]
pub enum ConfigError {
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// TOML parsing error
    #[error("TOML parse error: {0}")]
    TomlError(#[from] toml::de::Error),

    /// TOML serialization error
    #[error("TOML serialization error: {0}")]
    TomlSerializeError(#[from] toml::ser::Error),

    /// JSON parsing error
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// File watcher error
    #[error("File watch error: {0}")]
    NotifyError(#[from] notify::Error),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Validation error (alias)
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Resource not found (e.g., plugin not installed)
    #[error("Not found: {0}")]
    NotFound(String),

    /// Security error (checksum mismatch, signature invalid, etc.)
    #[error("Security error: {0}")]
    SecurityError(String),

    /// Network/HTTP error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Invalid configuration value
    #[error("Invalid value for {field}: {message}")]
    InvalidValue { field: String, message: String },

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Watch error
    #[error("File watch error: {0}")]
    WatchError(String),

    /// Invalid color format
    #[error("Invalid color: {0}")]
    InvalidColor(String),

    /// Invalid scrollback value
    #[error("Invalid scrollback: {0}")]
    InvalidScrollback(String),

    /// Invalid shell configuration
    #[error("Invalid shell: {0}")]
    InvalidShell(String),

    /// Invalid font size
    #[error("Invalid font size: {0}")]
    InvalidFontSize(String),

    /// Invalid line height
    #[error("Invalid line height: {0}")]
    InvalidLineHeight(String),

    /// Fusabi compilation error
    #[error("Fusabi compilation error: {0}")]
    FusabiCompileError(String),

    /// Fusabi runtime error
    #[error("Fusabi runtime error: {0}")]
    FusabiRuntimeError(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, ConfigError>;
