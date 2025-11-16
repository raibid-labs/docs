//! # raibid-core
//!
//! Core library for raibid-cli - meta-management tool for raibid-labs GitHub organization.
//!
//! This library provides the foundational functionality for:
//! - GitHub API integration
//! - Git repository operations
//! - Repository synchronization
//! - Configuration management
//! - Filtering and search

pub mod config;
pub mod error;
pub mod filter;
pub mod git;
pub mod github;
pub mod sync;
pub mod types;

// Re-export commonly used types
pub use config::Config;
pub use error::{Error, Result};
pub use types::{Repository, SyncStatus};
