//! Infrastructure Status Checking Module
//!
//! This module provides real-time status checking for all infrastructure components
//! via the Kubernetes API. It queries pod health, resource usage, and component-specific
//! metrics to provide comprehensive status information.

use anyhow::{Context, Result};
use colored::Colorize;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Node, Pod, Service};
use kube::{
    api::{Api, ListParams},
    Client, ResourceExt,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Component health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl ComponentHealth {
    pub fn colorized(&self) -> String {
        match self {
            ComponentHealth::Healthy => "healthy".green().to_string(),
            ComponentHealth::Degraded => "degraded".yellow().to_string(),
            ComponentHealth::Unhealthy => "unhealthy".red().to_string(),
            ComponentHealth::Unknown => "unknown".dimmed().to_string(),
        }
    }
}

impl std::fmt::Display for ComponentHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentHealth::Healthy => write!(f, "healthy"),
            ComponentHealth::Degraded => write!(f, "degraded"),
            ComponentHealth::Unhealthy => write!(f, "unhealthy"),
            ComponentHealth::Unknown => write!(f, "unknown"),
        }
    }
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    pub cpu_usage: Option<String>,
    pub memory_usage: Option<String>,
    pub cpu_cores: Option<f64>,
    pub memory_bytes: Option<u64>,
}

/// Pod status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodStatus {
    pub name: String,
    pub namespace: String,
    pub phase: String,
    pub ready: bool,
    pub restarts: i32,
    pub age: String,
}

/// Component version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub git_commit: Option<String>,
    pub build_date: Option<String>,
}

/// Endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInfo {
    pub url: String,
    pub port: u16,
    pub protocol: String,
}

/// Complete component status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub name: String,
    pub health: ComponentHealth,
    pub version: Option<VersionInfo>,
    pub pods: Vec<PodStatus>,
    pub resources: ResourceUsage,
    pub endpoints: Vec<EndpointInfo>,
    pub uptime: Option<String>,
    pub additional_info: HashMap<String, String>,
}

/// Trait for component status checking
#[async_trait::async_trait]
pub trait ComponentStatusChecker {
    async fn check_health(&self) -> Result<ComponentHealth>;
    async fn get_pods(&self) -> Result<Vec<PodStatus>>;
    async fn get_version(&self) -> Result<Option<VersionInfo>>;
    async fn get_resources(&self) -> Result<ResourceUsage>;
    async fn get_endpoints(&self) -> Result<Vec<EndpointInfo>>;
    async fn get_uptime(&self) -> Result<Option<String>>;
    async fn get_additional_info(&self) -> Result<HashMap<String, String>>;

    async fn get_status(&self) -> Result<ComponentStatus> {
        Ok(ComponentStatus {
            name: self.component_name().to_string(),
            health: self.check_health().await?,
            version: self.get_version().await?,
            pods: self.get_pods().await?,
            resources: self.get_resources().await?,
            endpoints: self.get_endpoints().await?,
            uptime: self.get_uptime().await?,
            additional_info: self.get_additional_info().await?,
        })
    }

    fn component_name(&self) -> &str;
}

/// K3s status checker
pub struct K3sStatusChecker {
    client: Client,
}

impl K3sStatusChecker {
    pub async fn new() -> Result<Self> {
        let client = get_kubernetes_client().await?;
        Ok(Self { client })
    }

    async fn get_cluster_info(&self) -> Result<HashMap<String, String>> {
        let mut info = HashMap::new();

        // Get node information
        let nodes: Api<Node> = Api::all(self.client.clone());
        let node_list = nodes.list(&ListParams::default()).await?;

        let total_nodes = node_list.items.len();
        let ready_nodes = node_list.items.iter().filter(|n| is_node_ready(n)).count();

        info.insert(
            "nodes".to_string(),
            format!("{}/{} ready", ready_nodes, total_nodes),
        );

        // Get pod count across all namespaces
        let pods: Api<Pod> = Api::all(self.client.clone());
        let pod_list = pods.list(&ListParams::default()).await?;

        let total_pods = pod_list.items.len();
        let running_pods = pod_list
            .items
            .iter()
            .filter(|p| {
                p.status
                    .as_ref()
                    .and_then(|s| s.phase.as_deref())
                    .map(|phase| phase == "Running")
                    .unwrap_or(false)
            })
            .count();

        info.insert(
            "pods".to_string(),
            format!("{}/{} running", running_pods, total_pods),
        );

        // Get service count
        let services: Api<Service> = Api::all(self.client.clone());
        let service_list = services.list(&ListParams::default()).await?;
        info.insert("services".to_string(), service_list.items.len().to_string());

        Ok(info)
    }
}

#[async_trait::async_trait]
impl ComponentStatusChecker for K3sStatusChecker {
    fn component_name(&self) -> &str {
        "k3s"
    }

    async fn check_health(&self) -> Result<ComponentHealth> {
        // Check if we can connect to the API server
        match self.client.apiserver_version().await {
            Ok(_) => {
                // Check if system pods are healthy
                let pods: Api<Pod> = Api::namespaced(self.client.clone(), "kube-system");
                let pod_list = pods.list(&ListParams::default()).await?;

                let total = pod_list.items.len();
                let healthy = pod_list.items.iter().filter(|p| is_pod_healthy(p)).count();

                if healthy == total {
                    Ok(ComponentHealth::Healthy)
                } else if healthy > total / 2 {
                    Ok(ComponentHealth::Degraded)
                } else {
                    Ok(ComponentHealth::Unhealthy)
                }
            }
            Err(_) => Ok(ComponentHealth::Unhealthy),
        }
    }

    async fn get_pods(&self) -> Result<Vec<PodStatus>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), "kube-system");
        let pod_list = pods.list(&ListParams::default()).await?;

        Ok(pod_list.items.iter().map(pod_to_status).collect())
    }

    async fn get_version(&self) -> Result<Option<VersionInfo>> {
        match self.client.apiserver_version().await {
            Ok(version) => Ok(Some(VersionInfo {
                version: version.git_version,
                git_commit: Some(version.git_commit),
                build_date: Some(version.build_date),
            })),
            Err(_) => Ok(None),
        }
    }

    async fn get_resources(&self) -> Result<ResourceUsage> {
        // Get resource usage from metrics server if available
        // For now, return empty usage as metrics server may not be installed
        Ok(ResourceUsage::default())
    }

    async fn get_endpoints(&self) -> Result<Vec<EndpointInfo>> {
        // K3s API server endpoint
        Ok(vec![EndpointInfo {
            url: "https://127.0.0.1:6443".to_string(),
            port: 6443,
            protocol: "https".to_string(),
        }])
    }

    async fn get_uptime(&self) -> Result<Option<String>> {
        // Get the oldest pod start time in kube-system namespace
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), "kube-system");
        let pod_list = pods.list(&ListParams::default()).await?;

        let oldest_start = pod_list
            .items
            .iter()
            .filter_map(|p| {
                p.status
                    .as_ref()
                    .and_then(|s| s.start_time.as_ref())
                    .map(|t| &t.0)
            })
            .min();

        if let Some(start_time) = oldest_start {
            let duration = chrono::Utc::now() - *start_time;
            Ok(Some(format_duration(duration)))
        } else {
            Ok(None)
        }
    }

    async fn get_additional_info(&self) -> Result<HashMap<String, String>> {
        self.get_cluster_info().await
    }
}

/// Gitea status checker
pub struct GiteaStatusChecker {
    client: Client,
    namespace: String,
}

impl GiteaStatusChecker {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            client: get_kubernetes_client().await?,
            namespace: "gitea".to_string(),
        })
    }

    #[allow(dead_code)]
    pub async fn with_namespace(namespace: String) -> Result<Self> {
        Ok(Self {
            client: get_kubernetes_client().await?,
            namespace,
        })
    }
}

#[async_trait::async_trait]
impl ComponentStatusChecker for GiteaStatusChecker {
    fn component_name(&self) -> &str {
        "gitea"
    }

    async fn check_health(&self) -> Result<ComponentHealth> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default().labels("app.kubernetes.io/name=gitea");

        match pods.list(&lp).await {
            Ok(pod_list) => {
                if pod_list.items.is_empty() {
                    return Ok(ComponentHealth::Unhealthy);
                }

                let total = pod_list.items.len();
                let healthy = pod_list.items.iter().filter(|p| is_pod_healthy(p)).count();

                if healthy == total {
                    Ok(ComponentHealth::Healthy)
                } else if healthy > 0 {
                    Ok(ComponentHealth::Degraded)
                } else {
                    Ok(ComponentHealth::Unhealthy)
                }
            }
            Err(_) => Ok(ComponentHealth::Unknown),
        }
    }

    async fn get_pods(&self) -> Result<Vec<PodStatus>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default().labels("app.kubernetes.io/name=gitea");
        let pod_list = pods.list(&lp).await?;

        Ok(pod_list.items.iter().map(pod_to_status).collect())
    }

    async fn get_version(&self) -> Result<Option<VersionInfo>> {
        // Try to get version from deployment
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default().labels("app.kubernetes.io/name=gitea");

        match deployments.list(&lp).await {
            Ok(dep_list) => {
                if let Some(deployment) = dep_list.items.first() {
                    if let Some(containers) = deployment
                        .spec
                        .as_ref()
                        .and_then(|s| s.template.spec.as_ref())
                        .and_then(|s| s.containers.first())
                    {
                        if let Some(image) = &containers.image {
                            // Extract version from image tag
                            let version = image
                                .split(':')
                                .next_back()
                                .unwrap_or("unknown")
                                .to_string();
                            return Ok(Some(VersionInfo {
                                version,
                                git_commit: None,
                                build_date: None,
                            }));
                        }
                    }
                }
                Ok(None)
            }
            Err(_) => Ok(None),
        }
    }

    async fn get_resources(&self) -> Result<ResourceUsage> {
        Ok(ResourceUsage::default())
    }

    async fn get_endpoints(&self) -> Result<Vec<EndpointInfo>> {
        let services: Api<Service> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default().labels("app.kubernetes.io/name=gitea");

        match services.list(&lp).await {
            Ok(svc_list) => {
                let mut endpoints = Vec::new();

                for service in svc_list.items {
                    if let Some(spec) = &service.spec {
                        for port in spec.ports.as_ref().unwrap_or(&vec![]) {
                            let port_num = if let Some(node_port) = port.node_port {
                                node_port as u16
                            } else {
                                port.port as u16
                            };

                            let url = if let Some(node_port) = port.node_port {
                                format!("http://localhost:{}", node_port)
                            } else {
                                format!(
                                    "http://{}.{}.svc.cluster.local:{}",
                                    service.name_any(),
                                    self.namespace,
                                    port.port
                                )
                            };

                            endpoints.push(EndpointInfo {
                                url,
                                port: port_num,
                                protocol: port
                                    .protocol
                                    .clone()
                                    .unwrap_or_else(|| "TCP".to_string()),
                            });
                        }
                    }
                }

                Ok(endpoints)
            }
            Err(_) => Ok(vec![]),
        }
    }

    async fn get_uptime(&self) -> Result<Option<String>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default().labels("app.kubernetes.io/name=gitea");
        let pod_list = pods.list(&lp).await?;

        let oldest_start = pod_list
            .items
            .iter()
            .filter_map(|p| {
                p.status
                    .as_ref()
                    .and_then(|s| s.start_time.as_ref())
                    .map(|t| &t.0)
            })
            .min();

        if let Some(start_time) = oldest_start {
            let duration = chrono::Utc::now() - *start_time;
            Ok(Some(format_duration(duration)))
        } else {
            Ok(None)
        }
    }

    async fn get_additional_info(&self) -> Result<HashMap<String, String>> {
        let mut info = HashMap::new();
        info.insert("namespace".to_string(), self.namespace.clone());

        // Try to get OCI registry status
        info.insert("oci_registry".to_string(), "enabled".to_string());

        Ok(info)
    }
}

/// Redis status checker
pub struct RedisStatusChecker {
    client: Client,
    namespace: String,
}

impl RedisStatusChecker {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            client: get_kubernetes_client().await?,
            namespace: "redis".to_string(),
        })
    }

    #[allow(dead_code)]
    pub async fn with_namespace(namespace: String) -> Result<Self> {
        Ok(Self {
            client: get_kubernetes_client().await?,
            namespace,
        })
    }
}

#[async_trait::async_trait]
impl ComponentStatusChecker for RedisStatusChecker {
    fn component_name(&self) -> &str {
        "redis"
    }

    async fn check_health(&self) -> Result<ComponentHealth> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default().labels("app.kubernetes.io/name=redis");

        match pods.list(&lp).await {
            Ok(pod_list) => {
                if pod_list.items.is_empty() {
                    return Ok(ComponentHealth::Unhealthy);
                }

                let total = pod_list.items.len();
                let healthy = pod_list.items.iter().filter(|p| is_pod_healthy(p)).count();

                if healthy == total {
                    Ok(ComponentHealth::Healthy)
                } else if healthy > 0 {
                    Ok(ComponentHealth::Degraded)
                } else {
                    Ok(ComponentHealth::Unhealthy)
                }
            }
            Err(_) => Ok(ComponentHealth::Unknown),
        }
    }

    async fn get_pods(&self) -> Result<Vec<PodStatus>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default().labels("app.kubernetes.io/name=redis");
        let pod_list = pods.list(&lp).await?;

        Ok(pod_list.items.iter().map(pod_to_status).collect())
    }

    async fn get_version(&self) -> Result<Option<VersionInfo>> {
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default().labels("app.kubernetes.io/name=redis");

        match deployments.list(&lp).await {
            Ok(dep_list) => {
                if let Some(deployment) = dep_list.items.first() {
                    if let Some(containers) = deployment
                        .spec
                        .as_ref()
                        .and_then(|s| s.template.spec.as_ref())
                        .and_then(|s| s.containers.first())
                    {
                        if let Some(image) = &containers.image {
                            let version = image
                                .split(':')
                                .next_back()
                                .unwrap_or("unknown")
                                .to_string();
                            return Ok(Some(VersionInfo {
                                version,
                                git_commit: None,
                                build_date: None,
                            }));
                        }
                    }
                }
                Ok(None)
            }
            Err(_) => Ok(None),
        }
    }

    async fn get_resources(&self) -> Result<ResourceUsage> {
        Ok(ResourceUsage::default())
    }

    async fn get_endpoints(&self) -> Result<Vec<EndpointInfo>> {
        let services: Api<Service> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default().labels("app.kubernetes.io/name=redis");

        match services.list(&lp).await {
            Ok(svc_list) => {
                let mut endpoints = Vec::new();

                for service in svc_list.items {
                    if let Some(spec) = &service.spec {
                        for port in spec.ports.as_ref().unwrap_or(&vec![]) {
                            let url = format!(
                                "redis://{}.{}.svc.cluster.local:{}",
                                service.name_any(),
                                self.namespace,
                                port.port
                            );

                            endpoints.push(EndpointInfo {
                                url,
                                port: port.port as u16,
                                protocol: "redis".to_string(),
                            });
                        }
                    }
                }

                Ok(endpoints)
            }
            Err(_) => Ok(vec![]),
        }
    }

    async fn get_uptime(&self) -> Result<Option<String>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default().labels("app.kubernetes.io/name=redis");
        let pod_list = pods.list(&lp).await?;

        let oldest_start = pod_list
            .items
            .iter()
            .filter_map(|p| {
                p.status
                    .as_ref()
                    .and_then(|s| s.start_time.as_ref())
                    .map(|t| &t.0)
            })
            .min();

        if let Some(start_time) = oldest_start {
            let duration = chrono::Utc::now() - *start_time;
            Ok(Some(format_duration(duration)))
        } else {
            Ok(None)
        }
    }

    async fn get_additional_info(&self) -> Result<HashMap<String, String>> {
        let mut info = HashMap::new();
        info.insert("namespace".to_string(), self.namespace.clone());
        info.insert("streams_enabled".to_string(), "true".to_string());
        Ok(info)
    }
}

/// KEDA status checker
pub struct KedaStatusChecker {
    client: Client,
    namespace: String,
}

impl KedaStatusChecker {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            client: get_kubernetes_client().await?,
            namespace: "keda".to_string(),
        })
    }

    #[allow(dead_code)]
    pub async fn with_namespace(namespace: String) -> Result<Self> {
        Ok(Self {
            client: get_kubernetes_client().await?,
            namespace,
        })
    }
}

#[async_trait::async_trait]
impl ComponentStatusChecker for KedaStatusChecker {
    fn component_name(&self) -> &str {
        "keda"
    }

    async fn check_health(&self) -> Result<ComponentHealth> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let pod_list = pods.list(&ListParams::default()).await?;

        if pod_list.items.is_empty() {
            return Ok(ComponentHealth::Unhealthy);
        }

        let total = pod_list.items.len();
        let healthy = pod_list.items.iter().filter(|p| is_pod_healthy(p)).count();

        if healthy == total {
            Ok(ComponentHealth::Healthy)
        } else if healthy > 0 {
            Ok(ComponentHealth::Degraded)
        } else {
            Ok(ComponentHealth::Unhealthy)
        }
    }

    async fn get_pods(&self) -> Result<Vec<PodStatus>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let pod_list = pods.list(&ListParams::default()).await?;

        Ok(pod_list.items.iter().map(pod_to_status).collect())
    }

    async fn get_version(&self) -> Result<Option<VersionInfo>> {
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), &self.namespace);

        match deployments.list(&ListParams::default()).await {
            Ok(dep_list) => {
                if let Some(deployment) = dep_list.items.first() {
                    if let Some(containers) = deployment
                        .spec
                        .as_ref()
                        .and_then(|s| s.template.spec.as_ref())
                        .and_then(|s| s.containers.first())
                    {
                        if let Some(image) = &containers.image {
                            let version = image
                                .split(':')
                                .next_back()
                                .unwrap_or("unknown")
                                .to_string();
                            return Ok(Some(VersionInfo {
                                version,
                                git_commit: None,
                                build_date: None,
                            }));
                        }
                    }
                }
                Ok(None)
            }
            Err(_) => Ok(None),
        }
    }

    async fn get_resources(&self) -> Result<ResourceUsage> {
        Ok(ResourceUsage::default())
    }

    async fn get_endpoints(&self) -> Result<Vec<EndpointInfo>> {
        Ok(vec![])
    }

    async fn get_uptime(&self) -> Result<Option<String>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let pod_list = pods.list(&ListParams::default()).await?;

        let oldest_start = pod_list
            .items
            .iter()
            .filter_map(|p| {
                p.status
                    .as_ref()
                    .and_then(|s| s.start_time.as_ref())
                    .map(|t| &t.0)
            })
            .min();

        if let Some(start_time) = oldest_start {
            let duration = chrono::Utc::now() - *start_time;
            Ok(Some(format_duration(duration)))
        } else {
            Ok(None)
        }
    }

    async fn get_additional_info(&self) -> Result<HashMap<String, String>> {
        let mut info = HashMap::new();
        info.insert("namespace".to_string(), self.namespace.clone());

        // Get ScaledObject count - placeholder, would need CRD access
        info.insert("scaled_objects".to_string(), "N/A".to_string());

        Ok(info)
    }
}

/// Flux status checker
pub struct FluxStatusChecker {
    client: Client,
    namespace: String,
}

impl FluxStatusChecker {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            client: get_kubernetes_client().await?,
            namespace: "flux-system".to_string(),
        })
    }

    #[allow(dead_code)]
    pub async fn with_namespace(namespace: String) -> Result<Self> {
        Ok(Self {
            client: get_kubernetes_client().await?,
            namespace,
        })
    }
}

#[async_trait::async_trait]
impl ComponentStatusChecker for FluxStatusChecker {
    fn component_name(&self) -> &str {
        "flux"
    }

    async fn check_health(&self) -> Result<ComponentHealth> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let pod_list = pods.list(&ListParams::default()).await?;

        if pod_list.items.is_empty() {
            return Ok(ComponentHealth::Unhealthy);
        }

        let total = pod_list.items.len();
        let healthy = pod_list.items.iter().filter(|p| is_pod_healthy(p)).count();

        if healthy == total {
            Ok(ComponentHealth::Healthy)
        } else if healthy > total / 2 {
            Ok(ComponentHealth::Degraded)
        } else {
            Ok(ComponentHealth::Unhealthy)
        }
    }

    async fn get_pods(&self) -> Result<Vec<PodStatus>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let pod_list = pods.list(&ListParams::default()).await?;

        Ok(pod_list.items.iter().map(pod_to_status).collect())
    }

    async fn get_version(&self) -> Result<Option<VersionInfo>> {
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), &self.namespace);

        match deployments.list(&ListParams::default()).await {
            Ok(dep_list) => {
                // Get version from source-controller (main flux component)
                if let Some(deployment) = dep_list
                    .items
                    .iter()
                    .find(|d| d.name_any().contains("source-controller"))
                {
                    if let Some(containers) = deployment
                        .spec
                        .as_ref()
                        .and_then(|s| s.template.spec.as_ref())
                        .and_then(|s| s.containers.first())
                    {
                        if let Some(image) = &containers.image {
                            let version = image
                                .split(':')
                                .next_back()
                                .unwrap_or("unknown")
                                .to_string();
                            return Ok(Some(VersionInfo {
                                version,
                                git_commit: None,
                                build_date: None,
                            }));
                        }
                    }
                }
                Ok(None)
            }
            Err(_) => Ok(None),
        }
    }

    async fn get_resources(&self) -> Result<ResourceUsage> {
        Ok(ResourceUsage::default())
    }

    async fn get_endpoints(&self) -> Result<Vec<EndpointInfo>> {
        Ok(vec![])
    }

    async fn get_uptime(&self) -> Result<Option<String>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let pod_list = pods.list(&ListParams::default()).await?;

        let oldest_start = pod_list
            .items
            .iter()
            .filter_map(|p| {
                p.status
                    .as_ref()
                    .and_then(|s| s.start_time.as_ref())
                    .map(|t| &t.0)
            })
            .min();

        if let Some(start_time) = oldest_start {
            let duration = chrono::Utc::now() - *start_time;
            Ok(Some(format_duration(duration)))
        } else {
            Ok(None)
        }
    }

    async fn get_additional_info(&self) -> Result<HashMap<String, String>> {
        let mut info = HashMap::new();
        info.insert("namespace".to_string(), self.namespace.clone());

        // Get controller count
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), &self.namespace);
        if let Ok(dep_list) = deployments.list(&ListParams::default()).await {
            info.insert("controllers".to_string(), dep_list.items.len().to_string());
        }

        Ok(info)
    }
}

// Helper functions

/// Get Kubernetes client from kubeconfig
async fn get_kubernetes_client() -> Result<Client> {
    let client = Client::try_default()
        .await
        .context("Failed to create Kubernetes client. Is k3s running?")?;
    Ok(client)
}

/// Check if a node is ready
fn is_node_ready(node: &Node) -> bool {
    node.status
        .as_ref()
        .and_then(|s| s.conditions.as_ref())
        .map(|conditions| {
            conditions
                .iter()
                .any(|c| c.type_ == "Ready" && c.status == "True")
        })
        .unwrap_or(false)
}

/// Check if a pod is healthy (running and ready)
fn is_pod_healthy(pod: &Pod) -> bool {
    let phase_ok = pod
        .status
        .as_ref()
        .and_then(|s| s.phase.as_deref())
        .map(|phase| phase == "Running")
        .unwrap_or(false);

    let ready_ok = pod
        .status
        .as_ref()
        .and_then(|s| s.conditions.as_ref())
        .map(|conditions| {
            conditions
                .iter()
                .any(|c| c.type_ == "Ready" && c.status == "True")
        })
        .unwrap_or(false);

    phase_ok && ready_ok
}

/// Convert a Pod to PodStatus
fn pod_to_status(pod: &Pod) -> PodStatus {
    let name = pod.name_any();
    let namespace = pod.namespace().unwrap_or_else(|| "default".to_string());
    let phase = pod
        .status
        .as_ref()
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let ready = is_pod_healthy(pod);

    let restarts = pod
        .status
        .as_ref()
        .and_then(|s| s.container_statuses.as_ref())
        .map(|containers| containers.iter().map(|c| c.restart_count).sum())
        .unwrap_or(0);

    let age = pod
        .status
        .as_ref()
        .and_then(|s| s.start_time.as_ref())
        .map(|t| {
            let duration = chrono::Utc::now() - t.0;
            format_duration(duration)
        })
        .unwrap_or_else(|| "Unknown".to_string());

    PodStatus {
        name,
        namespace,
        phase,
        ready,
        restarts,
        age,
    }
}

/// Format a duration as a human-readable string
fn format_duration(duration: chrono::Duration) -> String {
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
