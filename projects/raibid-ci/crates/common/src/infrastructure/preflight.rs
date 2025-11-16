//! Pre-flight Validation Checks
//!
//! This module provides comprehensive pre-flight validation to ensure system
//! requirements are met before installation begins.

#![allow(dead_code)]

use std::path::Path;
use std::process::Command;
use tracing::{debug, info, warn};

use crate::infrastructure::error::{InfraError, InfraResult, ValidationError};

/// System requirements
#[derive(Debug, Clone)]
pub struct SystemRequirements {
    /// Minimum free disk space in GB
    pub min_disk_space_gb: u64,
    /// Minimum free memory in GB
    pub min_memory_gb: u64,
    /// Required commands (binaries that must be in PATH)
    pub required_commands: Vec<String>,
    /// Optional commands (nice to have but not required)
    pub optional_commands: Vec<String>,
    /// Required directories that must exist
    pub required_directories: Vec<String>,
    /// Required network endpoints that must be reachable
    pub required_endpoints: Vec<String>,
}

impl Default for SystemRequirements {
    fn default() -> Self {
        Self {
            min_disk_space_gb: 10,
            min_memory_gb: 2,
            required_commands: vec!["tar".to_string(), "curl".to_string()],
            optional_commands: vec![],
            required_directories: vec![],
            required_endpoints: vec![],
        }
    }
}

/// Pre-flight check result
#[derive(Debug)]
pub struct PreFlightResult {
    pub passed: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

impl Default for PreFlightResult {
    fn default() -> Self {
        Self::new()
    }
}

impl PreFlightResult {
    pub fn new() -> Self {
        Self {
            passed: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.passed = false;
        self.errors.push(ValidationError {
            field: field.into(),
            message: message.into(),
        });
    }

    pub fn add_warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }

    pub fn to_result(&self, component: &str) -> InfraResult<()> {
        if self.passed {
            Ok(())
        } else {
            Err(InfraError::Validation {
                component: component.to_string(),
                errors: self.errors.clone(),
            })
        }
    }
}

/// Pre-flight validator
pub struct PreFlightValidator {
    requirements: SystemRequirements,
}

impl PreFlightValidator {
    pub fn new(requirements: SystemRequirements) -> Self {
        Self { requirements }
    }

    /// Run all pre-flight checks
    pub fn validate(&self, component: &str) -> InfraResult<()> {
        info!("Running pre-flight checks for {}", component);

        let mut result = PreFlightResult::new();

        // Check disk space
        self.check_disk_space(&mut result);

        // Check memory
        self.check_memory(&mut result);

        // Check required commands
        self.check_required_commands(&mut result);

        // Check optional commands
        self.check_optional_commands(&mut result);

        // Check required directories
        self.check_required_directories(&mut result);

        // Check network connectivity
        self.check_network_connectivity(&mut result);

        // Log warnings
        for warning in &result.warnings {
            warn!("Pre-flight warning: {}", warning);
        }

        result.to_result(component)
    }

    /// Check available disk space
    fn check_disk_space(&self, result: &mut PreFlightResult) {
        debug!(
            "Checking disk space (minimum: {} GB)",
            self.requirements.min_disk_space_gb
        );

        // Try to check disk space using df
        let output = Command::new("df").arg("-BG").arg(".").output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.lines().nth(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        if let Ok(available) = parts[3].trim_end_matches('G').parse::<u64>() {
                            if available < self.requirements.min_disk_space_gb {
                                result.add_error(
                                    "disk_space",
                                    format!(
                                        "Insufficient disk space. Required: {}GB, Available: {}GB",
                                        self.requirements.min_disk_space_gb, available
                                    ),
                                );
                            } else {
                                debug!("Disk space check passed: {}GB available", available);
                            }
                        }
                    }
                }
            }
            _ => {
                result.add_warning(
                    "Could not check disk space. Ensure sufficient space is available.",
                );
            }
        }
    }

    /// Check available memory
    fn check_memory(&self, result: &mut PreFlightResult) {
        debug!(
            "Checking memory (minimum: {} GB)",
            self.requirements.min_memory_gb
        );

        // Try to check memory using free
        let output = Command::new("free").arg("-g").output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.lines().nth(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        if let Ok(available) = parts[3].parse::<u64>() {
                            if available < self.requirements.min_memory_gb {
                                result.add_warning(format!(
                                    "Low available memory. Required: {}GB, Available: {}GB",
                                    self.requirements.min_memory_gb, available
                                ));
                            } else {
                                debug!("Memory check passed: {}GB available", available);
                            }
                        }
                    }
                }
            }
            _ => {
                result
                    .add_warning("Could not check memory. Ensure sufficient memory is available.");
            }
        }
    }

    /// Check required commands are available
    fn check_required_commands(&self, result: &mut PreFlightResult) {
        debug!(
            "Checking required commands: {:?}",
            self.requirements.required_commands
        );

        for command in &self.requirements.required_commands {
            if !self.is_command_available(command) {
                result.add_error(
                    "required_command",
                    format!("Required command '{}' not found in PATH", command),
                );
            } else {
                debug!("Required command '{}' is available", command);
            }
        }
    }

    /// Check optional commands
    fn check_optional_commands(&self, result: &mut PreFlightResult) {
        debug!(
            "Checking optional commands: {:?}",
            self.requirements.optional_commands
        );

        for command in &self.requirements.optional_commands {
            if !self.is_command_available(command) {
                result.add_warning(format!(
                    "Optional command '{}' not found. Some features may not be available.",
                    command
                ));
            } else {
                debug!("Optional command '{}' is available", command);
            }
        }
    }

    /// Check if a command is available in PATH
    fn is_command_available(&self, command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Check required directories exist
    fn check_required_directories(&self, result: &mut PreFlightResult) {
        debug!(
            "Checking required directories: {:?}",
            self.requirements.required_directories
        );

        for dir in &self.requirements.required_directories {
            let path = Path::new(dir);
            if !path.exists() {
                result.add_error(
                    "required_directory",
                    format!("Required directory '{}' does not exist", dir),
                );
            } else if !path.is_dir() {
                result.add_error(
                    "required_directory",
                    format!("'{}' exists but is not a directory", dir),
                );
            } else {
                debug!("Required directory '{}' exists", dir);
            }
        }
    }

    /// Check network connectivity to required endpoints
    fn check_network_connectivity(&self, result: &mut PreFlightResult) {
        if self.requirements.required_endpoints.is_empty() {
            return;
        }

        debug!(
            "Checking network connectivity to: {:?}",
            self.requirements.required_endpoints
        );

        for endpoint in &self.requirements.required_endpoints {
            // Try a simple curl check
            let output = Command::new("curl")
                .arg("-s")
                .arg("--connect-timeout")
                .arg("5")
                .arg("-o")
                .arg("/dev/null")
                .arg("-w")
                .arg("%{http_code}")
                .arg(endpoint)
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    let status_code = String::from_utf8_lossy(&output.stdout);
                    if status_code.starts_with('2') || status_code.starts_with('3') {
                        debug!("Network endpoint '{}' is reachable", endpoint);
                    } else {
                        result.add_warning(format!(
                            "Network endpoint '{}' returned HTTP {}",
                            endpoint, status_code
                        ));
                    }
                }
                _ => {
                    result.add_warning(format!(
                        "Could not reach network endpoint '{}'. Check network connectivity.",
                        endpoint
                    ));
                }
            }
        }
    }
}

/// Create system requirements for k3s installation
pub fn k3s_requirements() -> SystemRequirements {
    SystemRequirements {
        min_disk_space_gb: 10,
        min_memory_gb: 2,
        required_commands: vec!["tar".to_string(), "curl".to_string()],
        optional_commands: vec!["sudo".to_string()],
        required_directories: vec![],
        required_endpoints: vec!["https://github.com".to_string()],
    }
}

/// Create system requirements for Gitea installation
pub fn gitea_requirements() -> SystemRequirements {
    SystemRequirements {
        min_disk_space_gb: 15,
        min_memory_gb: 4,
        required_commands: vec!["kubectl".to_string(), "helm".to_string()],
        optional_commands: vec![],
        required_directories: vec![],
        required_endpoints: vec![],
    }
}

/// Create system requirements for Redis installation
pub fn redis_requirements() -> SystemRequirements {
    SystemRequirements {
        min_disk_space_gb: 10,
        min_memory_gb: 2,
        required_commands: vec!["kubectl".to_string(), "helm".to_string()],
        optional_commands: vec![],
        required_directories: vec![],
        required_endpoints: vec![],
    }
}

/// Create system requirements for KEDA installation
pub fn keda_requirements() -> SystemRequirements {
    SystemRequirements {
        min_disk_space_gb: 5,
        min_memory_gb: 1,
        required_commands: vec!["kubectl".to_string(), "helm".to_string()],
        optional_commands: vec![],
        required_directories: vec![],
        required_endpoints: vec![],
    }
}

/// Create system requirements for Flux installation
pub fn flux_requirements() -> SystemRequirements {
    SystemRequirements {
        min_disk_space_gb: 5,
        min_memory_gb: 1,
        required_commands: vec!["kubectl".to_string(), "tar".to_string(), "curl".to_string()],
        optional_commands: vec!["sudo".to_string()],
        required_directories: vec![],
        required_endpoints: vec!["https://github.com".to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preflight_result_new() {
        let result = PreFlightResult::new();
        assert!(result.passed);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_preflight_result_add_error() {
        let mut result = PreFlightResult::new();
        result.add_error("test_field", "test error");

        assert!(!result.passed);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].field, "test_field");
        assert_eq!(result.errors[0].message, "test error");
    }

    #[test]
    fn test_preflight_result_add_warning() {
        let mut result = PreFlightResult::new();
        result.add_warning("test warning");

        assert!(result.passed);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0], "test warning");
    }

    #[test]
    fn test_preflight_result_to_result_success() {
        let result = PreFlightResult::new();
        assert!(result.to_result("test").is_ok());
    }

    #[test]
    fn test_preflight_result_to_result_failure() {
        let mut result = PreFlightResult::new();
        result.add_error("field", "error");

        let res = result.to_result("test");
        assert!(res.is_err());
        if let Err(InfraError::Validation { component, errors }) = res {
            assert_eq!(component, "test");
            assert_eq!(errors.len(), 1);
        } else {
            panic!("Expected validation error");
        }
    }

    #[test]
    fn test_k3s_requirements() {
        let req = k3s_requirements();
        assert!(req.required_commands.contains(&"tar".to_string()));
        assert!(req.required_commands.contains(&"curl".to_string()));
        assert_eq!(req.min_disk_space_gb, 10);
    }

    #[test]
    fn test_gitea_requirements() {
        let req = gitea_requirements();
        assert!(req.required_commands.contains(&"kubectl".to_string()));
        assert!(req.required_commands.contains(&"helm".to_string()));
        assert_eq!(req.min_disk_space_gb, 15);
    }

    #[test]
    fn test_validator_check_required_commands() {
        let req = SystemRequirements {
            required_commands: vec!["sh".to_string()], // sh should exist on Unix
            ..Default::default()
        };
        let validator = PreFlightValidator::new(req);
        let mut result = PreFlightResult::new();

        validator.check_required_commands(&mut result);

        // sh should be available on Unix systems
        #[cfg(unix)]
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validator_check_required_commands_missing() {
        let req = SystemRequirements {
            required_commands: vec!["nonexistent_command_12345".to_string()],
            ..Default::default()
        };
        let validator = PreFlightValidator::new(req);
        let mut result = PreFlightResult::new();

        validator.check_required_commands(&mut result);

        assert!(!result.errors.is_empty());
        assert!(result.errors[0]
            .message
            .contains("nonexistent_command_12345"));
    }

    #[test]
    fn test_validator_check_optional_commands() {
        let req = SystemRequirements {
            optional_commands: vec!["nonexistent_optional_12345".to_string()],
            ..Default::default()
        };
        let validator = PreFlightValidator::new(req);
        let mut result = PreFlightResult::new();

        validator.check_optional_commands(&mut result);

        // Optional commands should only produce warnings
        assert!(result.passed);
        assert!(!result.warnings.is_empty());
    }
}
