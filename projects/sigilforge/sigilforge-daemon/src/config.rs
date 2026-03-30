//! Daemon configuration handling.

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Path to the Unix socket (Linux/macOS) or named pipe (Windows).
    pub socket_path: PathBuf,

    /// Path to the configuration file that was loaded.
    #[serde(skip)]
    pub config_path: PathBuf,

    /// Directory for storing account metadata.
    pub data_dir: PathBuf,

    /// Logging level.
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for DaemonConfig {
    fn default() -> Self {
        let dirs = project_dirs();
        let data_dir = dirs
            .as_ref()
            .map(|d| d.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".sigilforge"));

        let socket_path = if cfg!(unix) {
            dirs.as_ref()
                .map(|d| d.runtime_dir().unwrap_or(d.data_dir()).join("sigilforge.sock"))
                .unwrap_or_else(|| PathBuf::from("/tmp/sigilforge.sock"))
        } else {
            PathBuf::from(r"\\.\pipe\sigilforge")
        };

        Self {
            socket_path,
            config_path: PathBuf::new(),
            data_dir,
            log_level: default_log_level(),
        }
    }
}

/// Load configuration from the default location or create defaults.
pub fn load_config() -> Result<DaemonConfig> {
    let dirs = project_dirs();
    let config_path = dirs
        .as_ref()
        .map(|d| d.config_dir().join("daemon.toml"))
        .unwrap_or_else(|| PathBuf::from("sigilforge-daemon.toml"));

    let mut config = if config_path.exists() {
        let contents = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config from {:?}", config_path))?;
        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config from {:?}", config_path))?
    } else {
        DaemonConfig::default()
    };

    config.config_path = config_path;

    std::fs::create_dir_all(&config.data_dir)
        .with_context(|| format!("Failed to create data directory {:?}", config.data_dir))?;

    Ok(config)
}

fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "raibid-labs", "sigilforge")
}
