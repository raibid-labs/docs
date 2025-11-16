//! Error types for the agent

use thiserror::Error;

/// Agent error types
#[derive(Debug, Error)]
pub enum AgentError {
    /// Redis connection error
    #[error("Redis connection error: {0}")]
    RedisConnection(#[from] redis::RedisError),

    /// Job parsing error
    #[error("Job parsing error: {0}")]
    JobParsing(String),

    /// Git operation error
    #[error("Git operation error: {0}")]
    Git(#[from] git2::Error),

    /// Build execution error
    #[error("Build execution error: {0}")]
    BuildExecution(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Agent result type
pub type AgentResult<T> = Result<T, AgentError>;
