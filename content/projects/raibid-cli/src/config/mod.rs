//! Configuration management
//!
//! This module handles loading and managing configuration from various sources:
//! - Configuration files (YAML format)
//! - Environment variables
//! - Command-line flags
//! - Defaults
//!
//! Configuration sources are merged with the following precedence (highest to lowest):
//! 1. Environment variables (RAIBID_*)
//! 2. Local file (./raibid.yaml)
//! 3. User file (~/.config/raibid/config.yaml)
//! 4. System file (/etc/raibid/config.yaml)
//! 5. Defaults

mod loader;
mod schema;

// Re-export public API
pub use loader::{
    apply_env_overrides, discover_config_files, expand_paths, load_config, load_config_file,
    substitute_env_vars, validate_config,
};
pub use schema::{
    AgentsConfig, ApiConfig, ClusterConfig, Config, GiteaConfig, RedisConfig, UiConfig,
};
