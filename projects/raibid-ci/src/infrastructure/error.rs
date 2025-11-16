//! Infrastructure Error Types
//!
//! This module provides comprehensive error handling for all infrastructure operations
//! with detailed context, retry capabilities, and recovery suggestions.

use std::fmt;
use std::time::Duration;

/// Result type alias for infrastructure operations
#[allow(dead_code)]
pub type InfraResult<T> = Result<T, InfraError>;

/// Comprehensive error type for infrastructure operations
#[allow(dead_code)]
#[derive(Debug)]
pub enum InfraError {
    /// Download-related errors
    Download {
        component: String,
        url: String,
        reason: String,
        suggestion: String,
    },

    /// Checksum verification failures
    ChecksumMismatch {
        component: String,
        expected: String,
        actual: String,
        file_path: String,
    },

    /// Network connectivity issues
    Network {
        operation: String,
        reason: String,
        suggestion: String,
    },

    /// Installation failures
    Installation {
        component: String,
        phase: InstallPhase,
        reason: String,
        suggestion: String,
    },

    /// Configuration errors
    Configuration {
        component: String,
        field: String,
        reason: String,
        suggestion: String,
    },

    /// Prerequisites not met
    PrerequisiteMissing {
        component: String,
        prerequisite: String,
        suggestion: String,
    },

    /// Command execution failures
    CommandFailed {
        command: String,
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
        suggestion: String,
    },

    /// Kubernetes/cluster errors
    Kubernetes {
        operation: String,
        resource: String,
        reason: String,
        suggestion: String,
    },

    /// Helm-specific errors
    Helm {
        operation: HelmOperation,
        chart: String,
        reason: String,
        suggestion: String,
    },

    /// Timeout errors
    Timeout {
        operation: String,
        duration: Duration,
        suggestion: String,
    },

    /// Health check failures
    HealthCheck {
        component: String,
        check: String,
        reason: String,
        suggestion: String,
    },

    /// Rollback failures
    Rollback {
        component: String,
        reason: String,
        partial_cleanup: Vec<String>,
    },

    /// File system errors
    FileSystem {
        operation: String,
        path: String,
        reason: String,
    },

    /// Validation errors
    Validation {
        component: String,
        errors: Vec<ValidationError>,
    },

    /// Transient errors that can be retried
    Transient {
        operation: String,
        reason: String,
        retry_after: Option<Duration>,
    },

    /// Fatal errors that cannot be recovered
    Fatal {
        component: String,
        reason: String,
        context: Vec<String>,
    },
}

/// Installation phase for detailed error reporting
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallPhase {
    PreFlight,
    Download,
    Verification,
    Installation,
    Configuration,
    Bootstrap,
    Validation,
    PostInstall,
}

impl fmt::Display for InstallPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstallPhase::PreFlight => write!(f, "pre-flight checks"),
            InstallPhase::Download => write!(f, "download"),
            InstallPhase::Verification => write!(f, "verification"),
            InstallPhase::Installation => write!(f, "installation"),
            InstallPhase::Configuration => write!(f, "configuration"),
            InstallPhase::Bootstrap => write!(f, "bootstrap"),
            InstallPhase::Validation => write!(f, "validation"),
            InstallPhase::PostInstall => write!(f, "post-installation"),
        }
    }
}

/// Helm operation types
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelmOperation {
    RepoAdd,
    RepoUpdate,
    Install,
    Upgrade,
    Uninstall,
    List,
    Get,
}

impl fmt::Display for HelmOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HelmOperation::RepoAdd => write!(f, "repository add"),
            HelmOperation::RepoUpdate => write!(f, "repository update"),
            HelmOperation::Install => write!(f, "install"),
            HelmOperation::Upgrade => write!(f, "upgrade"),
            HelmOperation::Uninstall => write!(f, "uninstall"),
            HelmOperation::List => write!(f, "list"),
            HelmOperation::Get => write!(f, "get"),
        }
    }
}

/// Validation error details
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for InfraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InfraError::Download { component, url, reason, suggestion } => {
                write!(
                    f,
                    "Failed to download {} from {}\nReason: {}\nSuggestion: {}",
                    component, url, reason, suggestion
                )
            }
            InfraError::ChecksumMismatch { component, expected, actual, file_path } => {
                write!(
                    f,
                    "Checksum verification failed for {}\nFile: {}\nExpected: {}\nActual: {}\nSuggestion: The downloaded file may be corrupted. Please retry the installation.",
                    component, file_path, expected, actual
                )
            }
            InfraError::Network { operation, reason, suggestion } => {
                write!(
                    f,
                    "Network error during {}\nReason: {}\nSuggestion: {}",
                    operation, reason, suggestion
                )
            }
            InfraError::Installation { component, phase, reason, suggestion } => {
                write!(
                    f,
                    "Installation failed for {} during {}\nReason: {}\nSuggestion: {}",
                    component, phase, reason, suggestion
                )
            }
            InfraError::Configuration { component, field, reason, suggestion } => {
                write!(
                    f,
                    "Configuration error for {}\nField: {}\nReason: {}\nSuggestion: {}",
                    component, field, reason, suggestion
                )
            }
            InfraError::PrerequisiteMissing { component, prerequisite, suggestion } => {
                write!(
                    f,
                    "Missing prerequisite for {}: {}\nSuggestion: {}",
                    component, prerequisite, suggestion
                )
            }
            InfraError::CommandFailed { command, exit_code, stdout, stderr, suggestion } => {
                write!(
                    f,
                    "Command failed: {}\nExit code: {}\nStdout: {}\nStderr: {}\nSuggestion: {}",
                    command,
                    exit_code.map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string()),
                    stdout,
                    stderr,
                    suggestion
                )
            }
            InfraError::Kubernetes { operation, resource, reason, suggestion } => {
                write!(
                    f,
                    "Kubernetes {} operation failed for {}\nReason: {}\nSuggestion: {}",
                    operation, resource, reason, suggestion
                )
            }
            InfraError::Helm { operation, chart, reason, suggestion } => {
                write!(
                    f,
                    "Helm {} operation failed for chart '{}'\nReason: {}\nSuggestion: {}",
                    operation, chart, reason, suggestion
                )
            }
            InfraError::Timeout { operation, duration, suggestion } => {
                write!(
                    f,
                    "Operation timed out: {}\nDuration: {:?}\nSuggestion: {}",
                    operation, duration, suggestion
                )
            }
            InfraError::HealthCheck { component, check, reason, suggestion } => {
                write!(
                    f,
                    "Health check failed for {}: {}\nReason: {}\nSuggestion: {}",
                    component, check, reason, suggestion
                )
            }
            InfraError::Rollback { component, reason, partial_cleanup } => {
                write!(
                    f,
                    "Rollback failed for {}\nReason: {}\nPartially cleaned up: {:?}",
                    component, reason, partial_cleanup
                )
            }
            InfraError::FileSystem { operation, path, reason } => {
                write!(
                    f,
                    "File system error during {}\nPath: {}\nReason: {}",
                    operation, path, reason
                )
            }
            InfraError::Validation { component, errors } => {
                write!(f, "Validation failed for {}:\n", component)?;
                for err in errors {
                    write!(f, "  - {}: {}\n", err.field, err.message)?;
                }
                Ok(())
            }
            InfraError::Transient { operation, reason, retry_after } => {
                write!(
                    f,
                    "Transient error during {}\nReason: {}\n{}",
                    operation,
                    reason,
                    if let Some(duration) = retry_after {
                        format!("Retry after: {:?}", duration)
                    } else {
                        "This error may be retried".to_string()
                    }
                )
            }
            InfraError::Fatal { component, reason, context } => {
                write!(f, "Fatal error in {}\nReason: {}\nContext:\n", component, reason)?;
                for ctx in context {
                    write!(f, "  - {}\n", ctx)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for InfraError {}

/// Convert from anyhow::Error with context
impl InfraError {
    /// Create a download error
    #[allow(dead_code)]
    pub fn download(component: impl Into<String>, url: impl Into<String>, reason: impl Into<String>) -> Self {
        let component = component.into();
        let url = url.into();
        let reason = reason.into();

        let suggestion = if reason.contains("404") || reason.contains("not found") {
            "Verify the URL is correct and the resource exists.".to_string()
        } else if reason.contains("timeout") || reason.contains("timed out") {
            "Check your network connection and try again. Consider increasing timeout settings.".to_string()
        } else if reason.contains("DNS") || reason.contains("name resolution") {
            "Check your DNS settings and network connectivity.".to_string()
        } else {
            "Check your network connection and verify the download URL is accessible.".to_string()
        };

        InfraError::Download {
            component,
            url,
            reason,
            suggestion,
        }
    }

    /// Create a network error
    #[allow(dead_code)]
    pub fn network(operation: impl Into<String>, reason: impl Into<String>) -> Self {
        let operation = operation.into();
        let reason = reason.into();

        let suggestion = if reason.contains("timeout") {
            "Increase network timeout settings or check for network congestion.".to_string()
        } else if reason.contains("refused") {
            "Check if the target service is running and accessible.".to_string()
        } else {
            "Verify network connectivity and firewall settings.".to_string()
        };

        InfraError::Network {
            operation,
            reason,
            suggestion,
        }
    }

    /// Create an installation error
    #[allow(dead_code)]
    pub fn installation(
        component: impl Into<String>,
        phase: InstallPhase,
        reason: impl Into<String>,
    ) -> Self {
        let component = component.into();
        let reason = reason.into();

        let suggestion = match phase {
            InstallPhase::PreFlight => "Ensure all prerequisites are installed and system requirements are met.".to_string(),
            InstallPhase::Download => "Check network connectivity and disk space.".to_string(),
            InstallPhase::Verification => "The downloaded file may be corrupted. Try downloading again.".to_string(),
            InstallPhase::Installation => "Check for sufficient permissions and disk space.".to_string(),
            InstallPhase::Configuration => "Review the configuration parameters and correct any invalid values.".to_string(),
            InstallPhase::Bootstrap => "Ensure the cluster is accessible and healthy.".to_string(),
            InstallPhase::Validation => "Check component logs for detailed error information.".to_string(),
            InstallPhase::PostInstall => "Review post-installation requirements and dependencies.".to_string(),
        };

        InfraError::Installation {
            component,
            phase,
            reason,
            suggestion,
        }
    }

    /// Check if error is transient and can be retried
    #[allow(dead_code)]
    pub fn is_transient(&self) -> bool {
        matches!(self, InfraError::Transient { .. })
            || matches!(self, InfraError::Network { .. })
            || matches!(self, InfraError::Timeout { .. })
    }

    /// Check if error is fatal
    #[allow(dead_code)]
    pub fn is_fatal(&self) -> bool {
        matches!(self, InfraError::Fatal { .. })
    }

    /// Get suggested retry delay if applicable
    #[allow(dead_code)]
    pub fn retry_delay(&self) -> Option<Duration> {
        match self {
            InfraError::Transient { retry_after, .. } => *retry_after,
            InfraError::Network { .. } => Some(Duration::from_secs(5)),
            InfraError::Timeout { .. } => Some(Duration::from_secs(10)),
            _ => None,
        }
    }
}

/// Helper trait for converting errors with context
#[allow(dead_code)]
pub trait ErrorContext<T> {
    fn infra_context(self, component: &str, phase: InstallPhase) -> InfraResult<T>;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: std::error::Error,
{
    fn infra_context(self, component: &str, phase: InstallPhase) -> InfraResult<T> {
        self.map_err(|e| {
            InfraError::installation(
                component,
                phase,
                e.to_string(),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_error_creation() {
        let err = InfraError::download("k3s", "https://example.com/k3s", "HTTP 404");
        assert!(err.to_string().contains("k3s"));
        assert!(err.to_string().contains("404"));
    }

    #[test]
    fn test_transient_error_detection() {
        let err = InfraError::Transient {
            operation: "download".to_string(),
            reason: "timeout".to_string(),
            retry_after: Some(Duration::from_secs(5)),
        };
        assert!(err.is_transient());
        assert_eq!(err.retry_delay(), Some(Duration::from_secs(5)));
    }

    #[test]
    fn test_fatal_error_detection() {
        let err = InfraError::Fatal {
            component: "k3s".to_string(),
            reason: "unsupported platform".to_string(),
            context: vec!["ARM64 required".to_string()],
        };
        assert!(err.is_fatal());
    }

    #[test]
    fn test_installation_error_phases() {
        let err = InfraError::installation("gitea", InstallPhase::PreFlight, "kubectl not found");
        assert!(err.to_string().contains("pre-flight checks"));
        assert!(err.to_string().contains("prerequisites"));
    }

    #[test]
    fn test_network_error_suggestions() {
        let err = InfraError::network("download", "connection timeout");
        assert!(err.to_string().contains("timeout"));
        assert!(err.to_string().contains("network"));
    }

    #[test]
    fn test_validation_error() {
        let errors = vec![
            ValidationError {
                field: "password".to_string(),
                message: "cannot be empty".to_string(),
            },
            ValidationError {
                field: "namespace".to_string(),
                message: "invalid format".to_string(),
            },
        ];
        let err = InfraError::Validation {
            component: "gitea".to_string(),
            errors,
        };
        let msg = err.to_string();
        assert!(msg.contains("password"));
        assert!(msg.contains("namespace"));
    }

    #[test]
    fn test_helm_error_display() {
        let err = InfraError::Helm {
            operation: HelmOperation::Install,
            chart: "gitea-charts/gitea".to_string(),
            reason: "chart not found".to_string(),
            suggestion: "Run 'helm repo update'".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("install"));
        assert!(msg.contains("gitea-charts/gitea"));
    }
}
