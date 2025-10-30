//! Configuration schema definitions
//!
//! Defines the structure of configuration for the raibid-cli application.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Cluster configuration
    #[serde(default)]
    pub cluster: ClusterConfig,

    /// API server configuration
    #[serde(default)]
    pub api: ApiConfig,

    /// Agent configuration
    #[serde(default)]
    pub agents: AgentsConfig,

    /// Gitea configuration
    #[serde(default)]
    pub gitea: GiteaConfig,

    /// Redis configuration
    #[serde(default)]
    pub redis: RedisConfig,

    /// UI configuration
    #[serde(default)]
    pub ui: UiConfig,
}

impl Config {
    /// Load configuration (temporarily returns default config)
    /// Full implementation will be completed in another issue
    pub fn load() -> Result<Self> {
        Ok(Config::default())
    }
}

/// Cluster (k3s) configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ClusterConfig {
    /// Cluster name
    #[serde(default = "default_cluster_name")]
    pub name: String,

    /// Kubernetes API server port
    #[serde(default = "default_k8s_api_port")]
    pub api_port: u16,

    /// Path to kubeconfig file
    #[serde(default = "default_kubeconfig_path")]
    pub kubeconfig_path: PathBuf,

    /// Namespace for raibid resources
    #[serde(default = "default_namespace")]
    pub namespace: String,

    /// CPU cores reserved for system
    #[serde(default = "default_reserved_cores")]
    pub reserved_cores: u16,

    /// Memory reserved for system (in GB)
    #[serde(default = "default_reserved_memory_gb")]
    pub reserved_memory_gb: u16,
}

/// API server configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ApiConfig {
    /// API server host
    #[serde(default = "default_api_host")]
    pub host: String,

    /// API server port
    #[serde(default = "default_api_port")]
    pub port: u16,

    /// Enable TLS
    #[serde(default)]
    pub tls_enabled: bool,

    /// Path to TLS certificate
    #[serde(default)]
    pub tls_cert_path: Option<PathBuf>,

    /// Path to TLS private key
    #[serde(default)]
    pub tls_key_path: Option<PathBuf>,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgentsConfig {
    /// Agent types to enable
    #[serde(default = "default_agent_types")]
    pub types: Vec<String>,

    /// Minimum number of agents
    #[serde(default)]
    pub min_agents: u16,

    /// Maximum number of agents
    #[serde(default = "default_max_agents")]
    pub max_agents: u16,

    /// Agent idle timeout in seconds
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_seconds: u32,

    /// Agent scale-down delay in seconds
    #[serde(default = "default_scaledown_delay")]
    pub scaledown_delay_seconds: u32,

    /// Cache volume size in GB
    #[serde(default = "default_cache_size_gb")]
    pub cache_size_gb: u16,
}

/// Gitea configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GiteaConfig {
    /// Gitea URL
    #[serde(default = "default_gitea_url")]
    pub url: String,

    /// Gitea admin username
    #[serde(default = "default_gitea_admin_user")]
    pub admin_user: String,

    /// Gitea admin password (should use env var)
    #[serde(default)]
    pub admin_password: Option<String>,

    /// OCI registry enabled
    #[serde(default = "default_true")]
    pub registry_enabled: bool,

    /// OCI registry port
    #[serde(default = "default_registry_port")]
    pub registry_port: u16,
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RedisConfig {
    /// Redis host
    #[serde(default = "default_redis_host")]
    pub host: String,

    /// Redis port
    #[serde(default = "default_redis_port")]
    pub port: u16,

    /// Redis password (should use env var)
    #[serde(default)]
    pub password: Option<String>,

    /// Redis database number
    #[serde(default)]
    pub database: u8,

    /// Job queue stream name
    #[serde(default = "default_job_stream")]
    pub job_stream: String,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct UiConfig {
    /// Enable TUI
    #[serde(default = "default_true")]
    pub tui_enabled: bool,

    /// TUI refresh rate in milliseconds
    #[serde(default = "default_refresh_rate_ms")]
    pub refresh_rate_ms: u32,

    /// Color scheme (dark, light, auto)
    #[serde(default = "default_color_scheme")]
    pub color_scheme: String,

    /// Enable unicode characters
    #[serde(default = "default_true")]
    pub unicode_enabled: bool,
}

// Default functions
fn default_cluster_name() -> String {
    "raibid-ci".to_string()
}

fn default_k8s_api_port() -> u16 {
    6443
}

fn default_kubeconfig_path() -> PathBuf {
    PathBuf::from("~/.kube/config")
}

fn default_namespace() -> String {
    "raibid-ci".to_string()
}

fn default_reserved_cores() -> u16 {
    2
}

fn default_reserved_memory_gb() -> u16 {
    8
}

fn default_api_host() -> String {
    "127.0.0.1".to_string()
}

fn default_api_port() -> u16 {
    8080
}

fn default_agent_types() -> Vec<String> {
    vec!["rust".to_string()]
}

fn default_max_agents() -> u16 {
    10
}

fn default_idle_timeout() -> u32 {
    300 // 5 minutes
}

fn default_scaledown_delay() -> u32 {
    60 // 1 minute
}

fn default_cache_size_gb() -> u16 {
    50
}

fn default_gitea_url() -> String {
    "http://gitea.raibid-ci.svc.cluster.local:3000".to_string()
}

fn default_gitea_admin_user() -> String {
    "admin".to_string()
}

fn default_true() -> bool {
    true
}

fn default_registry_port() -> u16 {
    5000
}

fn default_redis_host() -> String {
    "redis.raibid-ci.svc.cluster.local".to_string()
}

fn default_redis_port() -> u16 {
    6379
}

fn default_job_stream() -> String {
    "raibid:jobs".to_string()
}

fn default_refresh_rate_ms() -> u32 {
    250
}

fn default_color_scheme() -> String {
    "dark".to_string()
}

// Default trait implementations
impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            name: default_cluster_name(),
            api_port: default_k8s_api_port(),
            kubeconfig_path: default_kubeconfig_path(),
            namespace: default_namespace(),
            reserved_cores: default_reserved_cores(),
            reserved_memory_gb: default_reserved_memory_gb(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: default_api_host(),
            port: default_api_port(),
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

impl Default for AgentsConfig {
    fn default() -> Self {
        Self {
            types: default_agent_types(),
            min_agents: 0,
            max_agents: default_max_agents(),
            idle_timeout_seconds: default_idle_timeout(),
            scaledown_delay_seconds: default_scaledown_delay(),
            cache_size_gb: default_cache_size_gb(),
        }
    }
}

impl Default for GiteaConfig {
    fn default() -> Self {
        Self {
            url: default_gitea_url(),
            admin_user: default_gitea_admin_user(),
            admin_password: None,
            registry_enabled: default_true(),
            registry_port: default_registry_port(),
        }
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            host: default_redis_host(),
            port: default_redis_port(),
            password: None,
            database: 0,
            job_stream: default_job_stream(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            tui_enabled: default_true(),
            refresh_rate_ms: default_refresh_rate_ms(),
            color_scheme: default_color_scheme(),
            unicode_enabled: default_true(),
        }
    }
}
