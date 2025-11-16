//! Infrastructure provisioning modules
//!
//! This module provides infrastructure component installation and management,
//! including k3s, Gitea, Redis, KEDA, and Flux.
//!
//! It also provides comprehensive error handling, retry logic, pre-flight validation,
//! rollback mechanisms, and health checking capabilities.

pub mod k3s;
pub mod gitea;
pub mod redis;
pub mod keda;
pub mod flux;
pub mod status;

// Error handling and utilities
pub mod error;
pub mod retry;
pub mod preflight;
pub mod rollback;
pub mod healthcheck;
pub mod utils;

pub use k3s::K3sInstaller;
pub use gitea::GiteaInstaller;
pub use redis::RedisInstaller;
pub use keda::KedaInstaller;
pub use flux::{FluxInstaller, FluxConfig};
pub use status::{
    ComponentStatusChecker, K3sStatusChecker, GiteaStatusChecker,
    RedisStatusChecker, KedaStatusChecker, FluxStatusChecker,
    ComponentHealth, ComponentStatus, ResourceUsage,
};

// Error handling exports (for tests and external use)
#[allow(unused_imports)]
pub use error::{InfraError, InfraResult, InstallPhase, HelmOperation, ValidationError, ErrorContext};
#[allow(unused_imports)]
pub use retry::{RetryConfig, retry_with_backoff, retry_with_backoff_async, poll_until, poll_until_async};
#[allow(unused_imports)]
pub use preflight::{
    SystemRequirements, PreFlightValidator, PreFlightResult,
    k3s_requirements, gitea_requirements, redis_requirements,
    keda_requirements, flux_requirements,
};
#[allow(unused_imports)]
pub use rollback::{RollbackManager, RollbackContext, RollbackAction};
#[allow(unused_imports)]
pub use healthcheck::{
    HealthStatus, HealthCheckResult, CheckResult,
    K3sHealthChecker, HelmHealthChecker,
};

// Config exports (for tests and commands)
#[allow(unused_imports)]
pub use k3s::K3sConfig;
#[allow(unused_imports)]
pub use gitea::{GiteaConfig, ServiceType};
#[allow(unused_imports)]
pub use redis::{RedisConfig, RedisConnectionInfo, RedisStreamsConfig};
#[allow(unused_imports)]
pub use keda::{KedaConfig, ScaledObjectConfig, TargetKind};

// Status exports (for TUI and commands)
#[allow(unused_imports)]
pub use status::{PodStatus, VersionInfo, EndpointInfo};
