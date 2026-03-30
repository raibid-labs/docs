//! Error types for the scripting system

use std::fmt;

/// Result type for scripting operations
pub type ScriptResult<T> = Result<T, ScriptError>;

/// Errors that can occur during script operations
#[derive(Debug, Clone)]
pub enum ScriptError {
    /// Failed to load script file
    LoadError {
        path: String,
        reason: String,
    },

    /// Failed to parse script
    ParseError {
        script: String,
        line: usize,
        column: usize,
        message: String,
    },

    /// Failed to compile script
    CompileError {
        script: String,
        message: String,
    },

    /// Runtime error during script execution
    RuntimeError {
        script: String,
        message: String,
    },

    /// API error - invalid API call from script
    ApiError {
        function: String,
        message: String,
    },

    /// Resource not found
    ResourceNotFound {
        resource_type: String,
        name: String,
    },

    /// Type error
    TypeError {
        expected: String,
        found: String,
    },

    /// IO error
    IoError(String),

    /// Watcher error
    WatcherError(String),
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptError::LoadError { path, reason } => {
                write!(f, "Failed to load script '{}': {}", path, reason)
            }
            ScriptError::ParseError {
                script,
                line,
                column,
                message,
            } => {
                write!(
                    f,
                    "Parse error in '{}' at {}:{}: {}",
                    script, line, column, message
                )
            }
            ScriptError::CompileError { script, message } => {
                write!(f, "Compilation error in '{}': {}", script, message)
            }
            ScriptError::RuntimeError { script, message } => {
                write!(f, "Runtime error in '{}': {}", script, message)
            }
            ScriptError::ApiError { function, message } => {
                write!(f, "API error in '{}': {}", function, message)
            }
            ScriptError::ResourceNotFound {
                resource_type,
                name,
            } => {
                write!(f, "{} '{}' not found", resource_type, name)
            }
            ScriptError::TypeError { expected, found } => {
                write!(f, "Type error: expected {}, found {}", expected, found)
            }
            ScriptError::IoError(msg) => write!(f, "IO error: {}", msg),
            ScriptError::WatcherError(msg) => write!(f, "Watcher error: {}", msg),
        }
    }
}

impl std::error::Error for ScriptError {}

impl From<std::io::Error> for ScriptError {
    fn from(err: std::io::Error) -> Self {
        ScriptError::IoError(err.to_string())
    }
}
