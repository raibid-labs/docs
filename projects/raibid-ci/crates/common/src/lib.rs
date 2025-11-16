//! raibid-common
//!
//! Common types, utilities, and infrastructure components shared across the raibid-ci workspace.
//! This crate provides:
//! - Configuration management
//! - Infrastructure deployment and management (k3s, Gitea, Flux, Redis, KEDA)
//! - Job types and data structures
//! - Shared error types
//! - Utility functions

pub mod config;
pub mod infrastructure;
pub mod jobs;

// Re-export commonly used types
pub use config::Config;
pub use infrastructure::error::InfraError;
pub use jobs::{Job, JobList, JobListQuery, JobLogEntry, JobLogs, JobStatus, JobTrigger};
