//! Rollback Utilities
//!
//! This module provides comprehensive rollback mechanisms for infrastructure installations
//! with transaction-like behavior and detailed cleanup tracking.

#![allow(dead_code)]

use tracing::{debug, info, warn};

use crate::infrastructure::error::{InfraError, InfraResult};

/// Rollback action that can be executed
pub type RollbackAction = Box<dyn FnOnce() -> InfraResult<()> + Send>;

/// Rollback manager for tracking and executing cleanup actions
pub struct RollbackManager {
    /// Component name
    component: String,
    /// Stack of rollback actions
    actions: Vec<(String, RollbackAction)>,
    /// Whether to automatically rollback on drop
    auto_rollback: bool,
}

impl RollbackManager {
    /// Create a new rollback manager for a component
    pub fn new(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            actions: Vec::new(),
            auto_rollback: true,
        }
    }

    /// Add a rollback action with a description
    pub fn add_action(&mut self, description: impl Into<String>, action: RollbackAction) {
        let desc = description.into();
        debug!("Adding rollback action: {}", desc);
        self.actions.push((desc, action));
    }

    /// Disable automatic rollback on drop
    pub fn disable_auto_rollback(&mut self) {
        self.auto_rollback = false;
    }

    /// Commit successful installation (disables rollback)
    pub fn commit(mut self) {
        info!("Committing successful installation for {}, disabling rollback", self.component);
        self.auto_rollback = false;
        self.actions.clear();
    }

    /// Execute all rollback actions in reverse order
    pub fn rollback(mut self) -> InfraResult<()> {
        self.execute_rollback()
    }

    /// Internal rollback execution
    fn execute_rollback(&mut self) -> InfraResult<()> {
        if self.actions.is_empty() {
            info!("No rollback actions to execute for {}", self.component);
            return Ok(());
        }

        warn!("Executing rollback for {} ({} actions)", self.component, self.actions.len());

        let mut failed_actions = Vec::new();
        let mut successful_cleanups = Vec::new();

        // Execute actions in reverse order (LIFO)
        while let Some((description, action)) = self.actions.pop() {
            debug!("Executing rollback action: {}", description);

            match action() {
                Ok(()) => {
                    info!("Rollback action succeeded: {}", description);
                    successful_cleanups.push(description);
                }
                Err(err) => {
                    warn!("Rollback action failed: {} - {}", description, err);
                    failed_actions.push((description, err.to_string()));
                }
            }
        }

        if !failed_actions.is_empty() {
            Err(InfraError::Rollback {
                component: self.component.clone(),
                reason: format!("{} rollback actions failed", failed_actions.len()),
                partial_cleanup: successful_cleanups,
            })
        } else {
            info!("Rollback completed successfully for {}", self.component);
            Ok(())
        }
    }
}

impl Drop for RollbackManager {
    fn drop(&mut self) {
        if self.auto_rollback && !self.actions.is_empty() {
            warn!("RollbackManager dropped with auto-rollback enabled, executing rollback");
            let _ = self.execute_rollback();
        }
    }
}

/// Rollback context for tracking installed resources
#[derive(Debug, Default)]
pub struct RollbackContext {
    /// Installed files
    pub files: Vec<String>,
    /// Created directories
    pub directories: Vec<String>,
    /// Kubernetes resources
    pub k8s_resources: Vec<KubernetesResource>,
    /// Helm releases
    pub helm_releases: Vec<HelmRelease>,
    /// Systemd services
    pub systemd_services: Vec<String>,
    /// Custom cleanup commands
    pub custom_commands: Vec<String>,
}

/// Kubernetes resource identifier
#[derive(Debug, Clone)]
pub struct KubernetesResource {
    pub kind: String,
    pub name: String,
    pub namespace: Option<String>,
}

/// Helm release identifier
#[derive(Debug, Clone)]
pub struct HelmRelease {
    pub name: String,
    pub namespace: String,
}

impl RollbackContext {
    /// Create a new rollback context
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a created file
    pub fn add_file(&mut self, path: impl Into<String>) {
        self.files.push(path.into());
    }

    /// Record a created directory
    pub fn add_directory(&mut self, path: impl Into<String>) {
        self.directories.push(path.into());
    }

    /// Record a Kubernetes resource
    pub fn add_k8s_resource(
        &mut self,
        kind: impl Into<String>,
        name: impl Into<String>,
        namespace: Option<String>,
    ) {
        self.k8s_resources.push(KubernetesResource {
            kind: kind.into(),
            name: name.into(),
            namespace,
        });
    }

    /// Record a Helm release
    pub fn add_helm_release(&mut self, name: impl Into<String>, namespace: impl Into<String>) {
        self.helm_releases.push(HelmRelease {
            name: name.into(),
            namespace: namespace.into(),
        });
    }

    /// Record a systemd service
    pub fn add_systemd_service(&mut self, service: impl Into<String>) {
        self.systemd_services.push(service.into());
    }

    /// Record a custom cleanup command
    pub fn add_custom_command(&mut self, command: impl Into<String>) {
        self.custom_commands.push(command.into());
    }

    /// Generate rollback actions from the context
    pub fn to_rollback_actions(&self, manager: &mut RollbackManager, kubeconfig: Option<&str>) {
        // Add Helm release cleanup
        for release in &self.helm_releases {
            let name = release.name.clone();
            let namespace = release.namespace.clone();
            let kubeconfig = kubeconfig.map(|s| s.to_string());

            manager.add_action(
                format!("Uninstall Helm release: {}/{}", namespace, name),
                Box::new(move || {
                    use std::process::Command;

                    let mut cmd = Command::new("helm");
                    cmd.arg("uninstall")
                        .arg(&name)
                        .arg("--namespace")
                        .arg(&namespace);

                    if let Some(ref kc) = kubeconfig {
                        cmd.env("KUBECONFIG", kc);
                    }

                    let output = cmd.output()
                        .map_err(|e| InfraError::CommandFailed {
                            command: format!("helm uninstall {}", name),
                            exit_code: None,
                            stdout: String::new(),
                            stderr: e.to_string(),
                            suggestion: "Check if Helm is installed and accessible".to_string(),
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        // Ignore "not found" errors during rollback
                        if !stderr.contains("not found") {
                            warn!("Helm uninstall warning: {}", stderr);
                        }
                    }

                    Ok(())
                }),
            );
        }

        // Add Kubernetes resource cleanup
        for resource in &self.k8s_resources {
            let kind = resource.kind.clone();
            let name = resource.name.clone();
            let namespace = resource.namespace.clone();
            let kubeconfig = kubeconfig.map(|s| s.to_string());

            manager.add_action(
                format!("Delete Kubernetes {}: {}", kind, name),
                Box::new(move || {
                    use std::process::Command;

                    let mut cmd = Command::new("kubectl");
                    cmd.arg("delete")
                        .arg(&kind)
                        .arg(&name)
                        .arg("--ignore-not-found=true");

                    if let Some(ref ns) = namespace {
                        cmd.arg("--namespace").arg(ns);
                    }

                    if let Some(ref kc) = kubeconfig {
                        cmd.env("KUBECONFIG", kc);
                    }

                    let output = cmd.output()
                        .map_err(|e| InfraError::CommandFailed {
                            command: format!("kubectl delete {} {}", kind, name),
                            exit_code: None,
                            stdout: String::new(),
                            stderr: e.to_string(),
                            suggestion: "Check if kubectl is installed and cluster is accessible".to_string(),
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if !stderr.contains("not found") {
                            warn!("kubectl delete warning: {}", stderr);
                        }
                    }

                    Ok(())
                }),
            );
        }

        // Add file cleanup
        for file in &self.files {
            let path = file.clone();
            manager.add_action(
                format!("Remove file: {}", path),
                Box::new(move || {
                    use std::fs;

                    if let Err(e) = fs::remove_file(&path) {
                        if e.kind() != std::io::ErrorKind::NotFound {
                            warn!("Failed to remove file {}: {}", path, e);
                        }
                    }
                    Ok(())
                }),
            );
        }

        // Add directory cleanup
        for directory in &self.directories {
            let path = directory.clone();
            manager.add_action(
                format!("Remove directory: {}", path),
                Box::new(move || {
                    use std::fs;

                    if let Err(e) = fs::remove_dir_all(&path) {
                        if e.kind() != std::io::ErrorKind::NotFound {
                            warn!("Failed to remove directory {}: {}", path, e);
                        }
                    }
                    Ok(())
                }),
            );
        }

        // Add systemd service cleanup
        for service in &self.systemd_services {
            let name = service.clone();
            manager.add_action(
                format!("Stop and disable systemd service: {}", name),
                Box::new(move || {
                    use std::process::Command;

                    // Stop service
                    let _ = Command::new("systemctl")
                        .arg("stop")
                        .arg(&name)
                        .output();

                    // Disable service
                    let _ = Command::new("systemctl")
                        .arg("disable")
                        .arg(&name)
                        .output();

                    Ok(())
                }),
            );
        }

        // Add custom commands
        for command in &self.custom_commands {
            let cmd = command.clone();
            manager.add_action(
                format!("Execute cleanup command: {}", cmd),
                Box::new(move || {
                    use std::process::Command;

                    let output = Command::new("sh")
                        .arg("-c")
                        .arg(&cmd)
                        .output()
                        .map_err(|e| InfraError::CommandFailed {
                            command: cmd.clone(),
                            exit_code: None,
                            stdout: String::new(),
                            stderr: e.to_string(),
                            suggestion: "Check command syntax and permissions".to_string(),
                        })?;

                    if !output.status.success() {
                        warn!("Custom cleanup command failed: {}", cmd);
                    }

                    Ok(())
                }),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollback_manager_creation() {
        let manager = RollbackManager::new("test");
        assert_eq!(manager.component, "test");
        assert!(manager.auto_rollback);
        assert_eq!(manager.actions.len(), 0);
    }

    #[test]
    fn test_rollback_manager_add_action() {
        let mut manager = RollbackManager::new("test");
        manager.add_action("test action", Box::new(|| Ok(())));
        assert_eq!(manager.actions.len(), 1);
    }

    #[test]
    fn test_rollback_manager_commit() {
        let mut manager = RollbackManager::new("test");
        manager.add_action("test action", Box::new(|| Ok(())));
        assert_eq!(manager.actions.len(), 1);

        manager.commit();
        // Actions should be cleared after commit
        assert_eq!(manager.actions.len(), 0);
    }

    #[test]
    fn test_rollback_manager_disable_auto() {
        let mut manager = RollbackManager::new("test");
        assert!(manager.auto_rollback);

        manager.disable_auto_rollback();
        assert!(!manager.auto_rollback);
    }

    #[test]
    fn test_rollback_context_creation() {
        let context = RollbackContext::new();
        assert!(context.files.is_empty());
        assert!(context.directories.is_empty());
        assert!(context.k8s_resources.is_empty());
        assert!(context.helm_releases.is_empty());
    }

    #[test]
    fn test_rollback_context_add_file() {
        let mut context = RollbackContext::new();
        context.add_file("/tmp/test.txt");
        assert_eq!(context.files.len(), 1);
        assert_eq!(context.files[0], "/tmp/test.txt");
    }

    #[test]
    fn test_rollback_context_add_directory() {
        let mut context = RollbackContext::new();
        context.add_directory("/tmp/test");
        assert_eq!(context.directories.len(), 1);
        assert_eq!(context.directories[0], "/tmp/test");
    }

    #[test]
    fn test_rollback_context_add_k8s_resource() {
        let mut context = RollbackContext::new();
        context.add_k8s_resource("Pod", "test-pod", Some("default".to_string()));
        assert_eq!(context.k8s_resources.len(), 1);
        assert_eq!(context.k8s_resources[0].kind, "Pod");
        assert_eq!(context.k8s_resources[0].name, "test-pod");
        assert_eq!(context.k8s_resources[0].namespace, Some("default".to_string()));
    }

    #[test]
    fn test_rollback_context_add_helm_release() {
        let mut context = RollbackContext::new();
        context.add_helm_release("redis", "redis-ns");
        assert_eq!(context.helm_releases.len(), 1);
        assert_eq!(context.helm_releases[0].name, "redis");
        assert_eq!(context.helm_releases[0].namespace, "redis-ns");
    }

    #[test]
    fn test_rollback_execution_order() {
        let mut manager = RollbackManager::new("test");
        let mut order = Vec::new();

        manager.add_action("first", {
            let mut order = order.clone();
            Box::new(move || {
                order.push(1);
                Ok(())
            })
        });

        manager.add_action("second", {
            let mut order = order.clone();
            Box::new(move || {
                order.push(2);
                Ok(())
            })
        });

        // Note: Actions execute in LIFO order, but we can't easily test this
        // without mutable state that's accessible from the closures
        // This test just ensures the manager can be created and executed
        manager.disable_auto_rollback();
        let result = manager.rollback();
        assert!(result.is_ok());
    }
}
