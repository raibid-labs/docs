//! KEDA Installation Module
//!
//! This module handles deploying KEDA (Kubernetes Event-Driven Autoscaling) with Helm
//! to k3s cluster and configuring ScaledObject for Redis Streams-based autoscaling.

use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info, warn};

/// KEDA Helm chart information
const KEDA_HELM_REPO: &str = "https://kedacore.github.io/charts";
const KEDA_HELM_REPO_NAME: &str = "kedacore";
const KEDA_CHART_NAME: &str = "kedacore/keda";
const KEDA_RELEASE_NAME: &str = "raibid-keda";
const KEDA_NAMESPACE: &str = "keda";

/// ScaledObject configuration for Redis Streams
#[derive(Debug, Clone)]
pub struct ScaledObjectConfig {
    /// Name of the ScaledObject resource
    pub name: String,
    /// Namespace where ScaledObject will be created
    pub namespace: String,
    /// Redis stream name to monitor
    pub stream_name: String,
    /// Redis consumer group name
    pub consumer_group: String,
    /// Redis service address
    pub redis_address: String,
    /// Pending entries count threshold for scaling
    pub pending_entries_count: String,
    /// Minimum replica count (0 for scale-to-zero)
    pub min_replica_count: i32,
    /// Maximum replica count
    pub max_replica_count: i32,
    /// Polling interval in seconds
    pub polling_interval: i32,
    /// Target deployment/job name
    pub target_name: String,
    /// Target resource type (Deployment or Job)
    pub target_kind: TargetKind,
}

/// Target resource kind for scaling
#[derive(Debug, Clone, PartialEq)]
pub enum TargetKind {
    Deployment,
    #[allow(dead_code)]
    Job,
}

impl Default for ScaledObjectConfig {
    fn default() -> Self {
        Self {
            name: "raibid-ci-agent-scaler".to_string(),
            namespace: "raibid-ci".to_string(),
            stream_name: "raibid:jobs".to_string(),
            consumer_group: "raibid-workers".to_string(),
            redis_address: "raibid-redis-master.raibid-redis.svc.cluster.local:6379".to_string(),
            pending_entries_count: "1".to_string(),
            min_replica_count: 0, // Scale to zero
            max_replica_count: 10,
            polling_interval: 10, // 10 seconds
            target_name: "raibid-ci-agent".to_string(),
            target_kind: TargetKind::Deployment,
        }
    }
}

/// KEDA installation configuration
#[derive(Debug, Clone)]
pub struct KedaConfig {
    /// KEDA Helm chart version
    pub chart_version: Option<String>,
    /// Kubernetes namespace for KEDA
    pub namespace: String,
    /// Helm release name
    pub release_name: String,
    /// KEDA operator log level
    pub log_level: String,
    /// Enable metrics server
    pub metrics_server_enabled: bool,
    /// ScaledObject configuration
    pub scaled_object: Option<ScaledObjectConfig>,
    /// Kubeconfig path
    pub kubeconfig_path: PathBuf,
}

impl Default for KedaConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/root"));
        Self {
            chart_version: None, // Use latest
            namespace: KEDA_NAMESPACE.to_string(),
            release_name: KEDA_RELEASE_NAME.to_string(),
            log_level: "info".to_string(),
            metrics_server_enabled: true,
            scaled_object: Some(ScaledObjectConfig::default()),
            kubeconfig_path: home.join(".kube").join("config"),
        }
    }
}

/// KEDA installer
pub struct KedaInstaller {
    config: KedaConfig,
}

impl KedaInstaller {
    /// Create a new KEDA installer with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(KedaConfig::default())
    }

    /// Create a new KEDA installer with custom configuration
    pub fn with_config(config: KedaConfig) -> Result<Self> {
        Ok(Self { config })
    }

    /// Check if kubectl is available
    #[allow(dead_code)]
    fn check_kubectl(&self) -> Result<()> {
        let output = Command::new("kubectl")
            .arg("version")
            .arg("--client")
            .output();

        match output {
            Ok(out) if out.status.success() => {
                debug!("kubectl is available");
                Ok(())
            }
            _ => Err(anyhow!(
                "kubectl not found. Please ensure k3s is installed and kubectl is in PATH."
            )),
        }
    }

    /// Check if Helm is available
    pub fn check_helm(&self) -> Result<()> {
        let output = Command::new("helm")
            .arg("version")
            .output();

        match output {
            Ok(out) if out.status.success() => {
                debug!("helm is available");
                Ok(())
            }
            _ => Err(anyhow!(
                "helm not found. Please install Helm 3.x"
            )),
        }
    }

    /// Add KEDA Helm repository
    pub fn add_helm_repo(&self) -> Result<()> {
        info!("Adding KEDA Helm repository");

        let output = Command::new("helm")
            .arg("repo")
            .arg("add")
            .arg(KEDA_HELM_REPO_NAME)
            .arg(KEDA_HELM_REPO)
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to add Helm repository")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Ignore error if repo already exists
            if !stderr.contains("already exists") {
                return Err(anyhow!("Failed to add Helm repository: {}", stderr));
            }
            debug!("Helm repository already exists");
        }

        // Update repository
        let output = Command::new("helm")
            .arg("repo")
            .arg("update")
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to update Helm repository")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to update Helm repository: {}", stderr));
        }

        info!("Helm repository added and updated");
        Ok(())
    }

    /// Create namespace for KEDA
    pub fn create_namespace(&self) -> Result<()> {
        info!("Creating namespace: {}", self.config.namespace);

        let output = Command::new("kubectl")
            .arg("create")
            .arg("namespace")
            .arg(&self.config.namespace)
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to create namespace")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Ignore error if namespace already exists
            if !stderr.contains("already exists") {
                return Err(anyhow!("Failed to create namespace: {}", stderr));
            }
            debug!("Namespace already exists");
        }

        Ok(())
    }

    /// Generate Helm values YAML
    fn generate_helm_values(&self) -> Result<String> {
        let values = format!(
            r#"
# KEDA Operator configuration
operator:
  name: keda-operator
  replicaCount: 1

  resources:
    requests:
      cpu: "100m"
      memory: "128Mi"
    limits:
      cpu: "500m"
      memory: "512Mi"

  # Log level: debug, info, error
  logLevel: {log_level}

# Metrics Server configuration
metricsServer:
  enabled: {metrics_server_enabled}
  replicaCount: 1

  resources:
    requests:
      cpu: "100m"
      memory: "128Mi"
    limits:
      cpu: "500m"
      memory: "512Mi"

# Admission webhooks (for validating ScaledObject/ScaledJob)
webhooks:
  enabled: true

  resources:
    requests:
      cpu: "50m"
      memory: "64Mi"
    limits:
      cpu: "200m"
      memory: "256Mi"

# Service account
serviceAccount:
  create: true
  name: keda-operator

# RBAC
rbac:
  create: true
"#,
            log_level = self.config.log_level,
            metrics_server_enabled = self.config.metrics_server_enabled,
        );

        Ok(values)
    }

    /// Deploy KEDA using Helm
    pub fn deploy_keda(&self) -> Result<()> {
        info!("Deploying KEDA via Helm");

        // Generate Helm values
        let values = self.generate_helm_values()?;
        let values_file = std::env::temp_dir().join("keda-values.yaml");
        fs::write(&values_file, values)
            .context("Failed to write Helm values file")?;

        // Build Helm install command
        let mut cmd = Command::new("helm");
        cmd.arg("upgrade")
            .arg("--install")
            .arg(&self.config.release_name)
            .arg(KEDA_CHART_NAME)
            .arg("--namespace")
            .arg(&self.config.namespace)
            .arg("--create-namespace")
            .arg("--values")
            .arg(&values_file)
            .arg("--wait")
            .arg("--timeout")
            .arg("5m")
            .env("KUBECONFIG", &self.config.kubeconfig_path);

        if let Some(ref version) = self.config.chart_version {
            cmd.arg("--version").arg(version);
        }

        debug!("Running Helm install: {:?}", cmd);

        let output = cmd.output()
            .context("Failed to run Helm install")?;

        // Clean up values file
        let _ = fs::remove_file(&values_file);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Helm install failed: {}", stderr));
        }

        info!("KEDA deployed successfully");
        Ok(())
    }

    /// Wait for KEDA to be ready
    pub fn wait_for_ready(&self) -> Result<()> {
        info!("Waiting for KEDA to be ready");

        // Wait for operator
        let output = Command::new("kubectl")
            .arg("wait")
            .arg("--for=condition=ready")
            .arg("pod")
            .arg("--selector")
            .arg("app=keda-operator")
            .arg("--namespace")
            .arg(&self.config.namespace)
            .arg("--timeout=300s")
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to wait for KEDA operator pods")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("KEDA operator pods not ready: {}", stderr));
        }

        // Wait for metrics server if enabled
        if self.config.metrics_server_enabled {
            let output = Command::new("kubectl")
                .arg("wait")
                .arg("--for=condition=ready")
                .arg("pod")
                .arg("--selector")
                .arg("app=keda-metrics-apiserver")
                .arg("--namespace")
                .arg(&self.config.namespace)
                .arg("--timeout=300s")
                .env("KUBECONFIG", &self.config.kubeconfig_path)
                .output()
                .context("Failed to wait for KEDA metrics server pods")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow!("KEDA metrics server pods not ready: {}", stderr));
            }
        }

        info!("KEDA is ready");
        Ok(())
    }

    /// Validate KEDA installation
    pub fn validate(&self) -> Result<()> {
        info!("Validating KEDA installation");

        // Check CRDs exist
        let crds = vec!["scaledobjects.keda.sh", "scaledjobs.keda.sh", "triggerauthentications.keda.sh"];

        for crd in crds {
            let output = Command::new("kubectl")
                .arg("get")
                .arg("crd")
                .arg(crd)
                .env("KUBECONFIG", &self.config.kubeconfig_path)
                .output()
                .context(format!("Failed to check CRD: {}", crd))?;

            if !output.status.success() {
                return Err(anyhow!("KEDA CRD not found: {}", crd));
            }
        }

        // Check operator deployment
        let output = Command::new("kubectl")
            .arg("get")
            .arg("deployment")
            .arg("keda-operator")
            .arg("--namespace")
            .arg(&self.config.namespace)
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to check KEDA operator deployment")?;

        if !output.status.success() {
            return Err(anyhow!("KEDA operator deployment not found"));
        }

        info!("KEDA validation successful");
        Ok(())
    }

    /// Generate ScaledObject YAML manifest
    fn generate_scaled_object_yaml(&self, config: &ScaledObjectConfig) -> Result<String> {
        let target_ref = match config.target_kind {
            TargetKind::Deployment => format!(
                r#"  scaleTargetRef:
    name: {}
    kind: Deployment
    apiVersion: apps/v1"#,
                config.target_name
            ),
            TargetKind::Job => format!(
                r#"  jobTargetRef:
    template:
      metadata:
        name: {}
      spec:
        containers:
        - name: ci-agent
          image: placeholder:latest
        restartPolicy: Never"#,
                config.target_name
            ),
        };

        let yaml = format!(
            r#"apiVersion: keda.sh/v1alpha1
kind: ScaledObject
metadata:
  name: {name}
  namespace: {namespace}
spec:
{target_ref}
  pollingInterval: {polling_interval}
  minReplicaCount: {min_replica_count}
  maxReplicaCount: {max_replica_count}
  triggers:
  - type: redis-streams
    metadata:
      address: {redis_address}
      stream: {stream_name}
      consumerGroup: {consumer_group}
      pendingEntriesCount: "{pending_entries_count}"
      lagCount: "5"
"#,
            name = config.name,
            namespace = config.namespace,
            target_ref = target_ref,
            polling_interval = config.polling_interval,
            min_replica_count = config.min_replica_count,
            max_replica_count = config.max_replica_count,
            redis_address = config.redis_address,
            stream_name = config.stream_name,
            consumer_group = config.consumer_group,
            pending_entries_count = config.pending_entries_count,
        );

        Ok(yaml)
    }

    /// Create ScaledObject for Redis Streams autoscaling
    pub fn create_scaled_object(&self) -> Result<()> {
        let config = match &self.config.scaled_object {
            Some(cfg) => cfg,
            None => {
                info!("No ScaledObject configuration provided, skipping");
                return Ok(());
            }
        };

        info!("Creating ScaledObject: {}", config.name);

        // Create target namespace if it doesn't exist
        let _ = Command::new("kubectl")
            .arg("create")
            .arg("namespace")
            .arg(&config.namespace)
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output();

        // Generate ScaledObject YAML
        let yaml = self.generate_scaled_object_yaml(config)?;
        let yaml_file = std::env::temp_dir().join("scaled-object.yaml");
        fs::write(&yaml_file, &yaml)
            .context("Failed to write ScaledObject YAML file")?;

        // Apply ScaledObject
        let output = Command::new("kubectl")
            .arg("apply")
            .arg("-f")
            .arg(&yaml_file)
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to apply ScaledObject")?;

        // Clean up YAML file
        let _ = fs::remove_file(&yaml_file);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to create ScaledObject: {}", stderr));
        }

        info!("ScaledObject created successfully");
        Ok(())
    }

    /// Get ScaledObject status
    pub fn get_scaled_object_status(&self) -> Result<String> {
        let config = match &self.config.scaled_object {
            Some(cfg) => cfg,
            None => return Err(anyhow!("No ScaledObject configured")),
        };

        let output = Command::new("kubectl")
            .arg("get")
            .arg("scaledobject")
            .arg(&config.name)
            .arg("--namespace")
            .arg(&config.namespace)
            .arg("-o")
            .arg("yaml")
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to get ScaledObject status")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to get ScaledObject status: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Complete installation workflow
    #[allow(dead_code)]
    pub fn install(&self) -> Result<()> {
        info!("Starting KEDA installation");

        // Pre-flight checks
        self.check_kubectl()?;
        self.check_helm()?;

        // Add Helm repository
        self.add_helm_repo()?;

        // Create namespace
        self.create_namespace()?;

        // Deploy KEDA
        self.deploy_keda()?;

        // Wait for ready
        self.wait_for_ready()?;

        // Validate installation
        self.validate()?;

        // Create ScaledObject if configured
        if self.config.scaled_object.is_some() {
            self.create_scaled_object()?;
        }

        info!("KEDA installation completed successfully");

        Ok(())
    }

    /// Uninstall KEDA
    pub fn uninstall(&self) -> Result<()> {
        info!("Uninstalling KEDA");

        // Delete ScaledObject first if configured
        if let Some(ref config) = self.config.scaled_object {
            let _ = Command::new("kubectl")
                .arg("delete")
                .arg("scaledobject")
                .arg(&config.name)
                .arg("--namespace")
                .arg(&config.namespace)
                .env("KUBECONFIG", &self.config.kubeconfig_path)
                .output();
        }

        // Uninstall Helm release
        let output = Command::new("helm")
            .arg("uninstall")
            .arg(&self.config.release_name)
            .arg("--namespace")
            .arg(&self.config.namespace)
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to uninstall KEDA")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Helm uninstall warning: {}", stderr);
        }

        // Delete namespace
        let output = Command::new("kubectl")
            .arg("delete")
            .arg("namespace")
            .arg(&self.config.namespace)
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to delete namespace")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Namespace deletion warning: {}", stderr);
        }

        info!("KEDA uninstalled");
        Ok(())
    }
}

impl Default for KedaInstaller {
    fn default() -> Self {
        Self::new().expect("Failed to create default KedaInstaller")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = KedaConfig::default();
        assert_eq!(config.namespace, KEDA_NAMESPACE);
        assert_eq!(config.release_name, KEDA_RELEASE_NAME);
        assert_eq!(config.log_level, "info");
        assert!(config.metrics_server_enabled);
        assert!(config.scaled_object.is_some());
    }

    #[test]
    fn test_default_scaled_object_config() {
        let config = ScaledObjectConfig::default();
        assert_eq!(config.name, "raibid-ci-agent-scaler");
        assert_eq!(config.namespace, "raibid-ci");
        assert_eq!(config.stream_name, "raibid:jobs");
        assert_eq!(config.consumer_group, "raibid-workers");
        assert_eq!(config.min_replica_count, 0);
        assert_eq!(config.max_replica_count, 10);
        assert_eq!(config.polling_interval, 10);
        assert_eq!(config.target_kind, TargetKind::Deployment);
    }

    #[test]
    fn test_installer_creation() {
        let installer = KedaInstaller::new();
        assert!(installer.is_ok());
    }

    #[test]
    fn test_helm_values_generation() {
        let installer = KedaInstaller::new().unwrap();
        let values = installer.generate_helm_values();

        assert!(values.is_ok());
        let values_str = values.unwrap();
        assert!(values_str.contains("operator:"));
        assert!(values_str.contains("metricsServer:"));
        assert!(values_str.contains("logLevel: info"));
        assert!(values_str.contains("enabled: true"));
    }

    #[test]
    fn test_scaled_object_yaml_generation() {
        let installer = KedaInstaller::new().unwrap();
        let config = ScaledObjectConfig::default();
        let yaml = installer.generate_scaled_object_yaml(&config);

        assert!(yaml.is_ok());
        let yaml_str = yaml.unwrap();
        assert!(yaml_str.contains("kind: ScaledObject"));
        assert!(yaml_str.contains("type: redis-streams"));
        assert!(yaml_str.contains("stream: raibid:jobs"));
        assert!(yaml_str.contains("consumerGroup: raibid-workers"));
        assert!(yaml_str.contains("minReplicaCount: 0"));
        assert!(yaml_str.contains("maxReplicaCount: 10"));
    }

    #[test]
    fn test_scaled_object_yaml_with_job_target() {
        let installer = KedaInstaller::new().unwrap();
        let mut config = ScaledObjectConfig::default();
        config.target_kind = TargetKind::Job;

        let yaml = installer.generate_scaled_object_yaml(&config);

        assert!(yaml.is_ok());
        let yaml_str = yaml.unwrap();
        assert!(yaml_str.contains("jobTargetRef:"));
        assert!(yaml_str.contains("restartPolicy: Never"));
    }

    #[test]
    fn test_custom_config() {
        let mut config = KedaConfig::default();
        config.log_level = "debug".to_string();
        config.metrics_server_enabled = false;

        let installer = KedaInstaller::with_config(config.clone());
        assert!(installer.is_ok());

        let inst = installer.unwrap();
        assert_eq!(inst.config.log_level, "debug");
        assert!(!inst.config.metrics_server_enabled);
    }

    #[test]
    fn test_scaled_object_config_customization() {
        let mut config = ScaledObjectConfig::default();
        config.min_replica_count = 2;
        config.max_replica_count = 20;
        config.pending_entries_count = "5".to_string();

        assert_eq!(config.min_replica_count, 2);
        assert_eq!(config.max_replica_count, 20);
        assert_eq!(config.pending_entries_count, "5");
    }
}
