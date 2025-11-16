//! Health Check Utilities
//!
//! This module provides health check functionality with timeouts and detailed status reporting.

#![allow(dead_code)]

use std::process::Command;
use std::time::Duration;
use tracing::{info, warn};

use crate::infrastructure::error::{InfraError, InfraResult};
use crate::infrastructure::retry::{RetryConfig, poll_until};

/// Health check status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "Healthy"),
            HealthStatus::Degraded => write!(f, "Degraded"),
            HealthStatus::Unhealthy => write!(f, "Unhealthy"),
            HealthStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
    pub checks: Vec<CheckResult>,
}

/// Individual check result
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

impl HealthCheckResult {
    pub fn new(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Unknown,
            message: String::new(),
            checks: Vec::new(),
        }
    }

    pub fn add_check(&mut self, name: impl Into<String>, passed: bool, message: impl Into<String>) {
        self.checks.push(CheckResult {
            name: name.into(),
            passed,
            message: message.into(),
        });
    }

    pub fn evaluate_status(&mut self) {
        if self.checks.is_empty() {
            self.status = HealthStatus::Unknown;
            self.message = "No checks performed".to_string();
            return;
        }

        let total = self.checks.len();
        let passed = self.checks.iter().filter(|c| c.passed).count();

        if passed == total {
            self.status = HealthStatus::Healthy;
            self.message = format!("All {} checks passed", total);
        } else if passed > 0 {
            self.status = HealthStatus::Degraded;
            self.message = format!("{}/{} checks passed", passed, total);
        } else {
            self.status = HealthStatus::Unhealthy;
            self.message = format!("All {} checks failed", total);
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.status == HealthStatus::Healthy
    }

    pub fn to_result(&self) -> InfraResult<()> {
        if self.is_healthy() {
            Ok(())
        } else {
            let failed_checks: Vec<String> = self.checks
                .iter()
                .filter(|c| !c.passed)
                .map(|c| format!("{}: {}", c.name, c.message))
                .collect();

            Err(InfraError::HealthCheck {
                component: self.component.clone(),
                check: "overall".to_string(),
                reason: failed_checks.join("; "),
                suggestion: "Check component logs and status for more details".to_string(),
            })
        }
    }
}

/// K3s health checker
pub struct K3sHealthChecker {
    kubeconfig: String,
    timeout: Duration,
}

impl K3sHealthChecker {
    pub fn new(kubeconfig: impl Into<String>) -> Self {
        Self {
            kubeconfig: kubeconfig.into(),
            timeout: Duration::from_secs(300),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn check(&self) -> InfraResult<HealthCheckResult> {
        info!("Checking k3s health");
        let mut result = HealthCheckResult::new("k3s");

        // Check if kubectl works
        let kubectl_check = self.check_kubectl();
        result.add_check("kubectl", kubectl_check.is_ok(),
            kubectl_check.as_ref().map(|s| s.clone()).unwrap_or_else(|e| e.to_string()));

        // Check if nodes are ready
        let nodes_check = self.check_nodes_ready();
        result.add_check("nodes_ready", nodes_check.is_ok(),
            nodes_check.as_ref().map(|s| s.clone()).unwrap_or_else(|e| e.to_string()));

        // Check if system pods are running
        let pods_check = self.check_system_pods();
        result.add_check("system_pods", pods_check.is_ok(),
            pods_check.as_ref().map(|s| s.clone()).unwrap_or_else(|e| e.to_string()));

        result.evaluate_status();
        Ok(result)
    }

    fn check_kubectl(&self) -> Result<String, String> {
        let output = Command::new("kubectl")
            .arg("version")
            .arg("--short")
            .env("KUBECONFIG", &self.kubeconfig)
            .output()
            .map_err(|e| format!("kubectl not accessible: {}", e))?;

        if output.status.success() {
            Ok("kubectl is accessible".to_string())
        } else {
            Err("kubectl command failed".to_string())
        }
    }

    fn check_nodes_ready(&self) -> Result<String, String> {
        let output = Command::new("kubectl")
            .arg("get")
            .arg("nodes")
            .arg("-o")
            .arg("jsonpath={.items[*].status.conditions[?(@.type=='Ready')].status}")
            .env("KUBECONFIG", &self.kubeconfig)
            .output()
            .map_err(|e| format!("Failed to get node status: {}", e))?;

        if !output.status.success() {
            return Err("kubectl get nodes failed".to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("True") {
            Ok("Node is Ready".to_string())
        } else {
            Err(format!("Node not ready: {}", stdout))
        }
    }

    fn check_system_pods(&self) -> Result<String, String> {
        let output = Command::new("kubectl")
            .arg("get")
            .arg("pods")
            .arg("--all-namespaces")
            .arg("-o")
            .arg("jsonpath={.items[*].status.phase}")
            .env("KUBECONFIG", &self.kubeconfig)
            .output()
            .map_err(|e| format!("Failed to get pod status: {}", e))?;

        if !output.status.success() {
            return Err("kubectl get pods failed".to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let phases: Vec<&str> = stdout.split_whitespace().collect();

        let running = phases.iter().filter(|&&p| p == "Running").count();
        let total = phases.len();

        if total == 0 {
            Err("No pods found".to_string())
        } else if running == total {
            Ok(format!("All {} pods are Running", total))
        } else {
            Err(format!("Only {}/{} pods are Running", running, total))
        }
    }

    pub fn wait_until_healthy(&self) -> InfraResult<()> {
        info!("Waiting for k3s to become healthy (timeout: {:?})", self.timeout);

        let config = RetryConfig::slow();

        poll_until(
            &config,
            self.timeout,
            "k3s health check",
            || {
                match self.check() {
                    Ok(result) => Ok(result.is_healthy()),
                    Err(e) => {
                        warn!("Health check error: {}", e);
                        Ok(false)
                    }
                }
            },
        )
    }
}

/// Helm release health checker
pub struct HelmHealthChecker {
    kubeconfig: String,
    namespace: String,
    release_name: String,
    timeout: Duration,
}

impl HelmHealthChecker {
    pub fn new(
        kubeconfig: impl Into<String>,
        namespace: impl Into<String>,
        release_name: impl Into<String>,
    ) -> Self {
        Self {
            kubeconfig: kubeconfig.into(),
            namespace: namespace.into(),
            release_name: release_name.into(),
            timeout: Duration::from_secs(600),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn check(&self) -> InfraResult<HealthCheckResult> {
        info!("Checking Helm release: {}/{}", self.namespace, self.release_name);
        let mut result = HealthCheckResult::new(format!("helm/{}", self.release_name));

        // Check if release exists
        let release_check = self.check_release_exists();
        result.add_check("release_exists", release_check.is_ok(),
            release_check.as_ref().map(|s| s.clone()).unwrap_or_else(|e| e.to_string()));

        // Check if release is deployed
        let status_check = self.check_release_status();
        result.add_check("release_deployed", status_check.is_ok(),
            status_check.as_ref().map(|s| s.clone()).unwrap_or_else(|e| e.to_string()));

        // Check if pods are ready
        let pods_check = self.check_pods_ready();
        result.add_check("pods_ready", pods_check.is_ok(),
            pods_check.as_ref().map(|s| s.clone()).unwrap_or_else(|e| e.to_string()));

        result.evaluate_status();
        Ok(result)
    }

    fn check_release_exists(&self) -> Result<String, String> {
        let output = Command::new("helm")
            .arg("list")
            .arg("--namespace")
            .arg(&self.namespace)
            .arg("--filter")
            .arg(&self.release_name)
            .arg("-o")
            .arg("json")
            .env("KUBECONFIG", &self.kubeconfig)
            .output()
            .map_err(|e| format!("helm list failed: {}", e))?;

        if !output.status.success() {
            return Err("helm list command failed".to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains(&self.release_name) {
            Ok(format!("Release '{}' exists", self.release_name))
        } else {
            Err(format!("Release '{}' not found", self.release_name))
        }
    }

    fn check_release_status(&self) -> Result<String, String> {
        let output = Command::new("helm")
            .arg("status")
            .arg(&self.release_name)
            .arg("--namespace")
            .arg(&self.namespace)
            .env("KUBECONFIG", &self.kubeconfig)
            .output()
            .map_err(|e| format!("helm status failed: {}", e))?;

        if !output.status.success() {
            return Err("helm status command failed".to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("STATUS: deployed") {
            Ok("Release is deployed".to_string())
        } else {
            Err(format!("Release status: {}", stdout))
        }
    }

    fn check_pods_ready(&self) -> Result<String, String> {
        let output = Command::new("kubectl")
            .arg("wait")
            .arg("--for=condition=ready")
            .arg("pod")
            .arg("--all")
            .arg("--namespace")
            .arg(&self.namespace)
            .arg("--timeout=10s")
            .env("KUBECONFIG", &self.kubeconfig)
            .output()
            .map_err(|e| format!("kubectl wait failed: {}", e))?;

        if output.status.success() {
            Ok("All pods are ready".to_string())
        } else {
            Err("Some pods are not ready".to_string())
        }
    }

    pub fn wait_until_healthy(&self) -> InfraResult<()> {
        info!("Waiting for Helm release to become healthy (timeout: {:?})", self.timeout);

        let config = RetryConfig::slow();

        poll_until(
            &config,
            self.timeout,
            &format!("helm release {} health check", self.release_name),
            || {
                match self.check() {
                    Ok(result) => Ok(result.is_healthy()),
                    Err(e) => {
                        warn!("Health check error: {}", e);
                        Ok(false)
                    }
                }
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "Healthy");
        assert_eq!(HealthStatus::Degraded.to_string(), "Degraded");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "Unhealthy");
        assert_eq!(HealthStatus::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_health_check_result_new() {
        let result = HealthCheckResult::new("test");
        assert_eq!(result.component, "test");
        assert_eq!(result.status, HealthStatus::Unknown);
        assert!(result.checks.is_empty());
    }

    #[test]
    fn test_health_check_result_add_check() {
        let mut result = HealthCheckResult::new("test");
        result.add_check("test1", true, "passed");
        assert_eq!(result.checks.len(), 1);
        assert_eq!(result.checks[0].name, "test1");
        assert!(result.checks[0].passed);
    }

    #[test]
    fn test_health_check_result_evaluate_all_passed() {
        let mut result = HealthCheckResult::new("test");
        result.add_check("check1", true, "ok");
        result.add_check("check2", true, "ok");
        result.evaluate_status();

        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.is_healthy());
    }

    #[test]
    fn test_health_check_result_evaluate_some_passed() {
        let mut result = HealthCheckResult::new("test");
        result.add_check("check1", true, "ok");
        result.add_check("check2", false, "failed");
        result.evaluate_status();

        assert_eq!(result.status, HealthStatus::Degraded);
        assert!(!result.is_healthy());
    }

    #[test]
    fn test_health_check_result_evaluate_all_failed() {
        let mut result = HealthCheckResult::new("test");
        result.add_check("check1", false, "failed");
        result.add_check("check2", false, "failed");
        result.evaluate_status();

        assert_eq!(result.status, HealthStatus::Unhealthy);
        assert!(!result.is_healthy());
    }

    #[test]
    fn test_health_check_result_to_result_healthy() {
        let mut result = HealthCheckResult::new("test");
        result.add_check("check1", true, "ok");
        result.evaluate_status();

        assert!(result.to_result().is_ok());
    }

    #[test]
    fn test_health_check_result_to_result_unhealthy() {
        let mut result = HealthCheckResult::new("test");
        result.add_check("check1", false, "failed");
        result.evaluate_status();

        let res = result.to_result();
        assert!(res.is_err());
        if let Err(InfraError::HealthCheck { component, .. }) = res {
            assert_eq!(component, "test");
        } else {
            panic!("Expected HealthCheck error");
        }
    }

    #[test]
    fn test_k3s_health_checker_creation() {
        let checker = K3sHealthChecker::new("/tmp/kubeconfig");
        assert_eq!(checker.kubeconfig, "/tmp/kubeconfig");
        assert_eq!(checker.timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_k3s_health_checker_with_timeout() {
        let checker = K3sHealthChecker::new("/tmp/kubeconfig")
            .with_timeout(Duration::from_secs(600));
        assert_eq!(checker.timeout, Duration::from_secs(600));
    }

    #[test]
    fn test_helm_health_checker_creation() {
        let checker = HelmHealthChecker::new("/tmp/kubeconfig", "default", "my-release");
        assert_eq!(checker.kubeconfig, "/tmp/kubeconfig");
        assert_eq!(checker.namespace, "default");
        assert_eq!(checker.release_name, "my-release");
        assert_eq!(checker.timeout, Duration::from_secs(600));
    }
}
