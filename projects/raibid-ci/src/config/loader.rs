//! Configuration loading and merging
//!
//! Handles loading configuration from multiple sources with proper precedence:
//! - Environment variables (highest priority)
//! - Local file (./raibid.yaml)
//! - User file (~/.config/raibid/config.yaml)
//! - System file (/etc/raibid/config.yaml)
//! - Defaults (lowest priority)

use super::schema::Config;
use anyhow::{Context, Result};
use regex::Regex;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Discover configuration files in standard locations
///
/// Returns paths in order of priority (lowest to highest):
/// 1. System config: /etc/raibid/config.yaml
/// 2. User config: ~/.config/raibid/config.yaml
/// 3. Local config: ./raibid.yaml
pub fn discover_config_files() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // System config
    let system_config = PathBuf::from("/etc/raibid/config.yaml");
    if system_config.exists() {
        paths.push(system_config);
    }

    // User config
    if let Some(config_dir) = dirs::config_dir() {
        let user_config = config_dir.join("raibid").join("config.yaml");
        if user_config.exists() {
            paths.push(user_config);
        }
    }

    // Local config
    let local_config = PathBuf::from("./raibid.yaml");
    if local_config.exists() {
        paths.push(local_config);
    }

    paths
}

/// Load and merge configuration from all sources
///
/// Precedence (highest to lowest):
/// 1. Environment variables (RAIBID_*)
/// 2. Local file (./raibid.yaml)
/// 3. User file (~/.config/raibid/config.yaml)
/// 4. System file (/etc/raibid/config.yaml)
/// 5. Default values
pub fn load_config() -> Result<Config> {
    // Start with defaults
    let mut config = Config::default();

    // Load and merge config files in order (system -> user -> local)
    let config_files = discover_config_files();
    for path in config_files {
        let file_config = load_config_file(&path)?;
        config = merge_configs(config, file_config);
    }

    // Expand paths (~ -> home directory)
    config = expand_paths(config)?;

    // Substitute environment variables (${VAR})
    config = substitute_env_vars(config)?;

    // Apply environment variable overrides (RAIBID_*)
    config = apply_env_overrides(config)?;

    // Validate final configuration
    validate_config(&config)?;

    Ok(config)
}

/// Load configuration from a specific file
pub fn load_config_file(path: &Path) -> Result<Config> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    // Use serde_path_to_error for better error messages
    let deserializer = serde_yaml::Deserializer::from_str(&contents);
    let config: Config = serde_path_to_error::deserialize(deserializer)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

    Ok(config)
}

/// Merge two configurations (base + override)
///
/// For scalars and options: override wins
/// For arrays: override completely replaces base
fn merge_configs(_base: Config, override_cfg: Config) -> Config {
    Config {
        cluster: override_cfg.cluster,
        api: override_cfg.api,
        agents: override_cfg.agents,
        gitea: override_cfg.gitea,
        redis: override_cfg.redis,
        ui: override_cfg.ui,
    }
}

/// Expand ~ and other path shortcuts to absolute paths
pub fn expand_paths(mut config: Config) -> Result<Config> {
    // Expand kubeconfig path
    let kubeconfig_str = config
        .cluster
        .kubeconfig_path
        .to_str()
        .context("Invalid kubeconfig path")?;
    let expanded = shellexpand::tilde(kubeconfig_str);
    config.cluster.kubeconfig_path = PathBuf::from(expanded.as_ref());

    // Expand TLS cert path if present
    if let Some(ref cert_path) = config.api.tls_cert_path {
        let cert_str = cert_path.to_str().context("Invalid TLS cert path")?;
        let expanded = shellexpand::tilde(cert_str);
        config.api.tls_cert_path = Some(PathBuf::from(expanded.as_ref()));
    }

    // Expand TLS key path if present
    if let Some(ref key_path) = config.api.tls_key_path {
        let key_str = key_path.to_str().context("Invalid TLS key path")?;
        let expanded = shellexpand::tilde(key_str);
        config.api.tls_key_path = Some(PathBuf::from(expanded.as_ref()));
    }

    Ok(config)
}

/// Substitute environment variables in string fields
///
/// Replaces ${VAR_NAME} with the value of $VAR_NAME
pub fn substitute_env_vars(mut config: Config) -> Result<Config> {
    let env_var_re = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();

    // Helper function to substitute in a string
    let substitute = |s: String| -> Result<String> {
        let mut result = s.clone();
        for cap in env_var_re.captures_iter(&s) {
            let var_name = &cap[1];
            if let Ok(var_value) = env::var(var_name) {
                let pattern = format!("${{{}}}", var_name);
                result = result.replace(&pattern, &var_value);
            }
        }
        Ok(result)
    };

    // Substitute in cluster config
    config.cluster.name = substitute(config.cluster.name)?;
    config.cluster.namespace = substitute(config.cluster.namespace)?;

    // Substitute in API config
    config.api.host = substitute(config.api.host)?;

    // Substitute in Gitea config
    config.gitea.url = substitute(config.gitea.url)?;
    config.gitea.admin_user = substitute(config.gitea.admin_user)?;
    if let Some(password) = config.gitea.admin_password {
        config.gitea.admin_password = Some(substitute(password)?);
    }

    // Substitute in Redis config
    config.redis.host = substitute(config.redis.host)?;
    if let Some(password) = config.redis.password {
        config.redis.password = Some(substitute(password)?);
    }
    config.redis.job_stream = substitute(config.redis.job_stream)?;

    // Substitute in UI config
    config.ui.color_scheme = substitute(config.ui.color_scheme)?;

    Ok(config)
}

/// Apply environment variable overrides
///
/// Environment variables follow the pattern: RAIBID_<SECTION>_<KEY>
/// Examples:
/// - RAIBID_CLUSTER_NAME
/// - RAIBID_API_PORT
/// - RAIBID_AGENTS_MAX_AGENTS
pub fn apply_env_overrides(mut config: Config) -> Result<Config> {
    // Cluster overrides
    if let Ok(val) = env::var("RAIBID_CLUSTER_NAME") {
        config.cluster.name = val;
    }
    if let Ok(val) = env::var("RAIBID_CLUSTER_API_PORT") {
        config.cluster.api_port = val.parse().context("Invalid RAIBID_CLUSTER_API_PORT")?;
    }
    if let Ok(val) = env::var("RAIBID_CLUSTER_NAMESPACE") {
        config.cluster.namespace = val;
    }
    if let Ok(val) = env::var("RAIBID_CLUSTER_RESERVED_CORES") {
        config.cluster.reserved_cores = val
            .parse()
            .context("Invalid RAIBID_CLUSTER_RESERVED_CORES")?;
    }
    if let Ok(val) = env::var("RAIBID_CLUSTER_RESERVED_MEMORY_GB") {
        config.cluster.reserved_memory_gb = val
            .parse()
            .context("Invalid RAIBID_CLUSTER_RESERVED_MEMORY_GB")?;
    }

    // API overrides
    if let Ok(val) = env::var("RAIBID_API_HOST") {
        config.api.host = val;
    }
    if let Ok(val) = env::var("RAIBID_API_PORT") {
        config.api.port = val.parse().context("Invalid RAIBID_API_PORT")?;
    }
    if let Ok(val) = env::var("RAIBID_API_TLS_ENABLED") {
        config.api.tls_enabled = val.parse().context("Invalid RAIBID_API_TLS_ENABLED")?;
    }

    // Agent overrides
    if let Ok(val) = env::var("RAIBID_AGENTS_MIN_AGENTS") {
        config.agents.min_agents = val.parse().context("Invalid RAIBID_AGENTS_MIN_AGENTS")?;
    }
    if let Ok(val) = env::var("RAIBID_AGENTS_MAX_AGENTS") {
        config.agents.max_agents = val.parse().context("Invalid RAIBID_AGENTS_MAX_AGENTS")?;
    }
    if let Ok(val) = env::var("RAIBID_AGENTS_IDLE_TIMEOUT_SECONDS") {
        config.agents.idle_timeout_seconds = val
            .parse()
            .context("Invalid RAIBID_AGENTS_IDLE_TIMEOUT_SECONDS")?;
    }

    // Gitea overrides
    if let Ok(val) = env::var("RAIBID_GITEA_URL") {
        config.gitea.url = val;
    }
    if let Ok(val) = env::var("RAIBID_GITEA_ADMIN_USER") {
        config.gitea.admin_user = val;
    }
    if let Ok(val) = env::var("RAIBID_GITEA_ADMIN_PASSWORD") {
        config.gitea.admin_password = Some(val);
    }

    // Redis overrides
    if let Ok(val) = env::var("RAIBID_REDIS_HOST") {
        config.redis.host = val;
    }
    if let Ok(val) = env::var("RAIBID_REDIS_PORT") {
        config.redis.port = val.parse().context("Invalid RAIBID_REDIS_PORT")?;
    }
    if let Ok(val) = env::var("RAIBID_REDIS_PASSWORD") {
        config.redis.password = Some(val);
    }
    if let Ok(val) = env::var("RAIBID_REDIS_DATABASE") {
        config.redis.database = val.parse().context("Invalid RAIBID_REDIS_DATABASE")?;
    }

    // UI overrides
    if let Ok(val) = env::var("RAIBID_UI_TUI_ENABLED") {
        config.ui.tui_enabled = val.parse().context("Invalid RAIBID_UI_TUI_ENABLED")?;
    }
    if let Ok(val) = env::var("RAIBID_UI_REFRESH_RATE_MS") {
        config.ui.refresh_rate_ms = val.parse().context("Invalid RAIBID_UI_REFRESH_RATE_MS")?;
    }
    if let Ok(val) = env::var("RAIBID_UI_COLOR_SCHEME") {
        config.ui.color_scheme = val;
    }

    Ok(config)
}

/// Validate configuration values
///
/// Checks for:
/// - Valid port ranges (1-65535)
/// - Valid numeric ranges (e.g., min <= max)
/// - Required fields when features are enabled
pub fn validate_config(config: &Config) -> Result<()> {
    // Validate ports
    validate_port("cluster.api_port", config.cluster.api_port)?;
    validate_port("api.port", config.api.port)?;
    validate_port("gitea.registry_port", config.gitea.registry_port)?;
    validate_port("redis.port", config.redis.port)?;

    // Validate agent configuration
    if config.agents.min_agents > config.agents.max_agents {
        anyhow::bail!(
            "agents.min_agents ({}) cannot be greater than agents.max_agents ({})",
            config.agents.min_agents,
            config.agents.max_agents
        );
    }

    if config.agents.max_agents == 0 {
        anyhow::bail!("agents.max_agents must be greater than 0");
    }

    if config.agents.types.is_empty() {
        anyhow::bail!("agents.types cannot be empty");
    }

    // Validate reserved resources
    if config.cluster.reserved_cores > 20 {
        anyhow::bail!(
            "cluster.reserved_cores ({}) cannot exceed 20 (DGX Spark total)",
            config.cluster.reserved_cores
        );
    }

    if config.cluster.reserved_memory_gb > 128 {
        anyhow::bail!(
            "cluster.reserved_memory_gb ({}) cannot exceed 128 (DGX Spark total)",
            config.cluster.reserved_memory_gb
        );
    }

    // Validate TLS configuration
    if config.api.tls_enabled {
        if config.api.tls_cert_path.is_none() {
            anyhow::bail!("api.tls_cert_path is required when api.tls_enabled is true");
        }
        if config.api.tls_key_path.is_none() {
            anyhow::bail!("api.tls_key_path is required when api.tls_enabled is true");
        }
    }

    // Validate color scheme
    let valid_schemes = ["dark", "light", "auto"];
    if !valid_schemes.contains(&config.ui.color_scheme.as_str()) {
        anyhow::bail!(
            "ui.color_scheme must be one of: {:?}, got: {}",
            valid_schemes,
            config.ui.color_scheme
        );
    }

    Ok(())
}

/// Validate a port number is in valid range
fn validate_port(field: &str, port: u16) -> Result<()> {
    if port == 0 {
        anyhow::bail!("{} cannot be 0", field);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_port() {
        assert!(validate_port("test", 0).is_err());
        assert!(validate_port("test", 1).is_ok());
        assert!(validate_port("test", 8080).is_ok());
        assert!(validate_port("test", 65535).is_ok());
    }

    #[test]
    fn test_validate_config_valid() {
        let config = Config::default();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_invalid_agent_range() {
        let mut config = Config::default();
        config.agents.min_agents = 10;
        config.agents.max_agents = 5;
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_substitute_env_vars() {
        env::set_var("TEST_VAR", "test_value");

        let mut config = Config::default();
        config.cluster.name = "cluster-${TEST_VAR}".to_string();

        let result = substitute_env_vars(config).unwrap();
        assert_eq!(result.cluster.name, "cluster-test_value");

        env::remove_var("TEST_VAR");
    }
}
