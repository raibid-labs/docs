//! Infrastructure provisioning modules
//!
//! This module provides infrastructure component installation and management,
//! including k3s, Gitea, Redis, KEDA, and Flux.
//!
//! It also provides comprehensive error handling, retry logic, pre-flight validation,
//! rollback mechanisms, and health checking capabilities.

pub mod flux;
pub mod gitea;
pub mod k3s;
pub mod keda;
pub mod redis;
pub mod status;

// Error handling and utilities
pub mod error;
pub mod healthcheck;
pub mod preflight;
pub mod retry;
pub mod rollback;
pub mod utils;

pub use flux::{FluxConfig, FluxInstaller};
pub use gitea::GiteaInstaller;
pub use k3s::K3sInstaller;
pub use keda::KedaInstaller;
pub use redis::RedisInstaller;
pub use status::{
    ComponentHealth, ComponentStatus, ComponentStatusChecker, FluxStatusChecker,
    GiteaStatusChecker, K3sStatusChecker, KedaStatusChecker, RedisStatusChecker, ResourceUsage,
};

// Error handling exports (for tests and external use)
#[allow(unused_imports)]
pub use error::{
    ErrorContext, HelmOperation, InfraError, InfraResult, InstallPhase, ValidationError,
};
#[allow(unused_imports)]
pub use healthcheck::{
    CheckResult, HealthCheckResult, HealthStatus, HelmHealthChecker, K3sHealthChecker,
};
#[allow(unused_imports)]
pub use preflight::{
    flux_requirements, gitea_requirements, k3s_requirements, keda_requirements, redis_requirements,
    PreFlightResult, PreFlightValidator, SystemRequirements,
};
#[allow(unused_imports)]
pub use retry::{
    poll_until, poll_until_async, retry_with_backoff, retry_with_backoff_async, RetryConfig,
};
#[allow(unused_imports)]
pub use rollback::{RollbackAction, RollbackContext, RollbackManager};

// Config exports (for tests and commands)
#[allow(unused_imports)]
pub use gitea::{GiteaConfig, ServiceType};
#[allow(unused_imports)]
pub use k3s::K3sConfig;
#[allow(unused_imports)]
pub use keda::{KedaConfig, ScaledObjectConfig, TargetKind};
#[allow(unused_imports)]
pub use redis::{RedisConfig, RedisConnectionInfo, RedisStreamsConfig};

// Status exports (for TUI and commands)
#[allow(unused_imports)]
pub use status::{EndpointInfo, PodStatus, VersionInfo};
