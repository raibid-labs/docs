//! Redis Installation Module
//!
//! This module handles deploying Redis with Helm to k3s cluster and configuring
//! Redis Streams for job queue management.

use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

/// Redis Helm chart information
const REDIS_HELM_REPO: &str = "https://charts.bitnami.com/bitnami";
const REDIS_HELM_REPO_NAME: &str = "bitnami";
const REDIS_CHART_NAME: &str = "bitnami/redis";
const REDIS_RELEASE_NAME: &str = "raibid-redis";
const REDIS_NAMESPACE: &str = "raibid-redis";

/// Redis Streams configuration for job queue
#[derive(Debug, Clone)]
pub struct RedisStreamsConfig {
    /// Job queue stream name
    pub queue_stream: String,
    /// Consumer group name
    pub consumer_group: String,
    /// Max length of stream (0 = unlimited)
    #[allow(dead_code)]
    pub max_length: u64,
}

impl Default for RedisStreamsConfig {
    fn default() -> Self {
        Self {
            queue_stream: "raibid:jobs".to_string(),
            consumer_group: "raibid-workers".to_string(),
            max_length: 10000, // Keep last 10k jobs
        }
    }
}

/// Redis installation configuration
#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// Redis Helm chart version
    pub chart_version: Option<String>,
    /// Kubernetes namespace for Redis
    pub namespace: String,
    /// Helm release name
    pub release_name: String,
    /// Enable persistence
    pub persistence_enabled: bool,
    /// Persistence size
    pub persistence_size: String,
    /// Enable authentication
    pub auth_enabled: bool,
    /// Password for Redis (generated if not provided)
    pub password: Option<String>,
    /// Enable Redis Sentinel for HA
    pub sentinel_enabled: bool,
    /// Number of Redis replicas
    pub replica_count: u32,
    /// Streams configuration
    pub streams_config: RedisStreamsConfig,
    /// Kubeconfig path
    pub kubeconfig_path: PathBuf,
}

impl Default for RedisConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/root"));
        Self {
            chart_version: None, // Use latest
            namespace: REDIS_NAMESPACE.to_string(),
            release_name: REDIS_RELEASE_NAME.to_string(),
            persistence_enabled: true,
            persistence_size: "8Gi".to_string(),
            auth_enabled: true,
            password: None, // Auto-generate
            sentinel_enabled: false, // MVP: single instance
            replica_count: 0, // MVP: no replicas
            streams_config: RedisStreamsConfig::default(),
            kubeconfig_path: home.join(".kube").join("config"),
        }
    }
}

/// Redis installer
pub struct RedisInstaller {
    config: RedisConfig,
}

impl RedisInstaller {
    /// Create a new Redis installer with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(RedisConfig::default())
    }

    /// Create a new Redis installer with custom configuration
    pub fn with_config(config: RedisConfig) -> Result<Self> {
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
    #[allow(dead_code)]
    fn check_helm(&self) -> Result<()> {
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

    /// Add Bitnami Helm repository
    pub fn add_helm_repo(&self) -> Result<()> {
        info!("Adding Bitnami Helm repository");

        let output = Command::new("helm")
            .arg("repo")
            .arg("add")
            .arg(REDIS_HELM_REPO_NAME)
            .arg(REDIS_HELM_REPO)
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

    /// Create namespace for Redis
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

    /// Generate password if not provided
    fn get_or_generate_password(&mut self) -> String {
        if let Some(ref pwd) = self.config.password {
            pwd.clone()
        } else {
            use rand::Rng;
            const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                      abcdefghijklmnopqrstuvwxyz\
                                      0123456789";
            let mut rng = rand::thread_rng();
            let password: String = (0..32)
                .map(|_| {
                    let idx = rng.gen_range(0..CHARSET.len());
                    CHARSET[idx] as char
                })
                .collect();
            self.config.password = Some(password.clone());
            password
        }
    }

    /// Generate Helm values YAML
    fn generate_helm_values(&mut self) -> Result<String> {
        let password = self.get_or_generate_password();

        let values = format!(
            r#"
auth:
  enabled: {auth_enabled}
  password: "{password}"

architecture: standalone

master:
  persistence:
    enabled: {persistence_enabled}
    size: {persistence_size}

  resources:
    requests:
      memory: "256Mi"
      cpu: "100m"
    limits:
      memory: "512Mi"
      cpu: "500m"

replica:
  replicaCount: {replica_count}
  persistence:
    enabled: {persistence_enabled}
    size: {persistence_size}

sentinel:
  enabled: {sentinel_enabled}

metrics:
  enabled: true
  serviceMonitor:
    enabled: false

# Redis configuration
commonConfiguration: |-
  # Enable AOF persistence for durability
  appendonly yes
  appendfsync everysec

  # Stream configuration
  stream-node-max-entries 100
"#,
            auth_enabled = self.config.auth_enabled,
            password = password,
            persistence_enabled = self.config.persistence_enabled,
            persistence_size = self.config.persistence_size,
            replica_count = self.config.replica_count,
            sentinel_enabled = self.config.sentinel_enabled,
        );

        Ok(values)
    }

    /// Deploy Redis using Helm
    pub fn deploy_redis(&mut self) -> Result<()> {
        info!("Deploying Redis via Helm");

        // Generate Helm values
        let values = self.generate_helm_values()?;
        let values_file = std::env::temp_dir().join("redis-values.yaml");
        fs::write(&values_file, values)
            .context("Failed to write Helm values file")?;

        // Build Helm install command
        let mut cmd = Command::new("helm");
        cmd.arg("upgrade")
            .arg("--install")
            .arg(&self.config.release_name)
            .arg(REDIS_CHART_NAME)
            .arg("--namespace")
            .arg(&self.config.namespace)
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

        info!("Redis deployed successfully");
        Ok(())
    }

    /// Wait for Redis to be ready
    pub fn wait_for_ready(&self) -> Result<()> {
        info!("Waiting for Redis to be ready");

        let output = Command::new("kubectl")
            .arg("wait")
            .arg("--for=condition=ready")
            .arg("pod")
            .arg("--selector")
            .arg(format!("app.kubernetes.io/name=redis"))
            .arg("--namespace")
            .arg(&self.config.namespace)
            .arg("--timeout=300s")
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to wait for Redis pods")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Redis pods not ready: {}", stderr));
        }

        info!("Redis is ready");
        Ok(())
    }

    /// Get Redis connection details
    pub fn get_connection_info(&self) -> Result<RedisConnectionInfo> {
        let host = format!("{}-master.{}.svc.cluster.local",
            self.config.release_name,
            self.config.namespace
        );
        let port = 6379;
        let password = self.config.password.clone();

        Ok(RedisConnectionInfo {
            host,
            port,
            password,
            namespace: self.config.namespace.clone(),
        })
    }

    /// Initialize Redis Streams for job queue
    pub fn initialize_streams(&self) -> Result<()> {
        info!("Initializing Redis Streams for job queue");

        let conn_info = self.get_connection_info()?;
        let pod_name = self.get_master_pod_name()?;

        // Create consumer group using kubectl exec
        let group_cmd = if self.config.auth_enabled {
            format!(
                "redis-cli -a {} XGROUP CREATE {} {} $ MKSTREAM",
                conn_info.password.as_ref().unwrap_or(&String::new()),
                self.config.streams_config.queue_stream,
                self.config.streams_config.consumer_group,
            )
        } else {
            format!(
                "redis-cli XGROUP CREATE {} {} $ MKSTREAM",
                self.config.streams_config.queue_stream,
                self.config.streams_config.consumer_group,
            )
        };

        let output = Command::new("kubectl")
            .arg("exec")
            .arg("-n")
            .arg(&self.config.namespace)
            .arg(&pod_name)
            .arg("--")
            .arg("sh")
            .arg("-c")
            .arg(&group_cmd)
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to create consumer group")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Ignore error if group already exists
            if !stderr.contains("BUSYGROUP") {
                return Err(anyhow!("Failed to create consumer group: {}", stderr));
            }
            debug!("Consumer group already exists");
        }

        info!("Redis Streams initialized");
        Ok(())
    }

    /// Get master pod name
    fn get_master_pod_name(&self) -> Result<String> {
        let output = Command::new("kubectl")
            .arg("get")
            .arg("pods")
            .arg("-n")
            .arg(&self.config.namespace)
            .arg("--selector")
            .arg("app.kubernetes.io/component=master")
            .arg("-o")
            .arg("jsonpath={.items[0].metadata.name}")
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to get master pod name")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to get master pod name: {}", stderr));
        }

        let pod_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if pod_name.is_empty() {
            return Err(anyhow!("No master pod found"));
        }

        Ok(pod_name)
    }

    /// Validate Redis installation
    pub fn validate(&self) -> Result<()> {
        info!("Validating Redis installation");

        let pod_name = self.get_master_pod_name()?;

        // Test connection with PING
        let ping_cmd = if self.config.auth_enabled && self.config.password.is_some() {
            format!(
                "redis-cli -a {} PING",
                self.config.password.as_ref().unwrap()
            )
        } else {
            "redis-cli PING".to_string()
        };

        let output = Command::new("kubectl")
            .arg("exec")
            .arg("-n")
            .arg(&self.config.namespace)
            .arg(&pod_name)
            .arg("--")
            .arg("sh")
            .arg("-c")
            .arg(&ping_cmd)
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to ping Redis")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Redis PING failed: {}", stderr));
        }

        let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if response != "PONG" {
            return Err(anyhow!("Unexpected Redis response: {}", response));
        }

        info!("Redis validation successful");
        Ok(())
    }

    /// Save connection credentials to file
    pub fn save_credentials(&self, path: &Path) -> Result<()> {
        info!("Saving Redis credentials to {:?}", path);

        let conn_info = self.get_connection_info()?;
        let credentials = serde_json::json!({
            "host": conn_info.host,
            "port": conn_info.port,
            "password": conn_info.password,
            "namespace": conn_info.namespace,
            "stream": self.config.streams_config.queue_stream,
            "consumer_group": self.config.streams_config.consumer_group,
        });

        let json = serde_json::to_string_pretty(&credentials)
            .context("Failed to serialize credentials")?;

        fs::write(path, json)
            .context("Failed to write credentials file")?;

        // Set proper permissions (600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms)?;
        }

        info!("Credentials saved");
        Ok(())
    }

    /// Complete installation workflow
    #[allow(dead_code)]
    pub fn install(&mut self) -> Result<RedisConnectionInfo> {
        info!("Starting Redis installation");

        // Pre-flight checks
        self.check_kubectl()?;
        self.check_helm()?;

        // Add Helm repository
        self.add_helm_repo()?;

        // Create namespace
        self.create_namespace()?;

        // Deploy Redis
        self.deploy_redis()?;

        // Wait for ready
        self.wait_for_ready()?;

        // Initialize streams
        self.initialize_streams()?;

        // Validate installation
        self.validate()?;

        // Get connection info
        let conn_info = self.get_connection_info()?;

        info!("Redis installation completed successfully");

        Ok(conn_info)
    }

    /// Uninstall Redis
    pub fn uninstall(&self) -> Result<()> {
        info!("Uninstalling Redis");

        let output = Command::new("helm")
            .arg("uninstall")
            .arg(&self.config.release_name)
            .arg("--namespace")
            .arg(&self.config.namespace)
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to uninstall Redis")?;

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

        info!("Redis uninstalled");
        Ok(())
    }
}

impl Default for RedisInstaller {
    fn default() -> Self {
        Self::new().expect("Failed to create default RedisInstaller")
    }
}

/// Redis connection information
#[derive(Debug, Clone)]
pub struct RedisConnectionInfo {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub namespace: String,
}

impl RedisConnectionInfo {
    /// Get connection URL
    #[allow(dead_code)]
    pub fn connection_url(&self) -> String {
        if let Some(ref pwd) = self.password {
            format!("redis://:{}@{}:{}", pwd, self.host, self.port)
        } else {
            format!("redis://{}:{}", self.host, self.port)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RedisConfig::default();
        assert_eq!(config.namespace, REDIS_NAMESPACE);
        assert_eq!(config.release_name, REDIS_RELEASE_NAME);
        assert!(config.persistence_enabled);
        assert!(config.auth_enabled);
    }

    #[test]
    fn test_default_streams_config() {
        let config = RedisStreamsConfig::default();
        assert_eq!(config.queue_stream, "raibid:jobs");
        assert_eq!(config.consumer_group, "raibid-workers");
        assert_eq!(config.max_length, 10000);
    }

    #[test]
    fn test_installer_creation() {
        let installer = RedisInstaller::new();
        assert!(installer.is_ok());
    }

    #[test]
    fn test_password_generation() {
        let mut installer = RedisInstaller::new().unwrap();
        let password1 = installer.get_or_generate_password();
        let password2 = installer.get_or_generate_password();

        // Should be same password (cached)
        assert_eq!(password1, password2);
        assert_eq!(password1.len(), 32);
    }

    #[test]
    fn test_helm_values_generation() {
        let mut installer = RedisInstaller::new().unwrap();
        let values = installer.generate_helm_values();

        assert!(values.is_ok());
        let values_str = values.unwrap();
        assert!(values_str.contains("auth:"));
        assert!(values_str.contains("persistence:"));
        assert!(values_str.contains("appendonly yes"));
    }

    #[test]
    fn test_connection_info_url() {
        let conn_info = RedisConnectionInfo {
            host: "redis-master.raibid-redis.svc.cluster.local".to_string(),
            port: 6379,
            password: Some("testpass".to_string()),
            namespace: "raibid-redis".to_string(),
        };

        let url = conn_info.connection_url();
        assert_eq!(url, "redis://:testpass@redis-master.raibid-redis.svc.cluster.local:6379");
    }

    #[test]
    fn test_connection_info_url_no_auth() {
        let conn_info = RedisConnectionInfo {
            host: "redis-master.raibid-redis.svc.cluster.local".to_string(),
            port: 6379,
            password: None,
            namespace: "raibid-redis".to_string(),
        };

        let url = conn_info.connection_url();
        assert_eq!(url, "redis://redis-master.raibid-redis.svc.cluster.local:6379");
    }
}
