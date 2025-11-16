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
pub use loader::{discover_config_files, load_config, load_config_file, validate_config};
pub use schema::Config;
