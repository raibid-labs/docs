//! Error types for raibid-core

use thiserror::Error;

/// Result type alias using our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for raibid-core operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("GitHub API error: {0}")]
    GitHub(String),

    #[error("Git operation failed: {0}")]
    Git(#[from] git2::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),

    #[error("Uncommitted changes detected in {0}")]
    UncommittedChanges(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid filter expression: {0}")]
    InvalidFilter(String),

    #[error("{0}")]
    Other(String),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Other(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Other(s.to_string())
    }
}
