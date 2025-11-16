//! Server configuration

use serde::{Deserialize, Serialize};

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerConfig {
    /// Server host address
    pub host: String,

    /// Server port
    pub port: u16,

    /// Log format (json or text)
    pub log_format: String,

    /// Enable CORS
    pub cors_enabled: bool,

    /// Maximum request body size in bytes
    pub max_body_size: usize,

    /// Redis connection URL
    pub redis_url: String,

    /// Gitea webhook secret
    pub gitea_webhook_secret: Option<String>,

    /// GitHub webhook secret
    pub github_webhook_secret: Option<String>,

    /// Rate limit (requests per minute)
    pub rate_limit_rpm: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            log_format: "text".to_string(),
            cors_enabled: true,
            max_body_size: 10 * 1024 * 1024, // 10MB
            redis_url: "redis://127.0.0.1:6379".to_string(),
            gitea_webhook_secret: None,
            github_webhook_secret: None,
            rate_limit_rpm: 100,
        }
    }
}

impl ServerConfig {
    /// Create configuration from the common config
    pub fn from_common_config(config: &raibid_common::Config) -> Self {
        Self {
            host: config.api.host.clone(),
            port: config.api.port,
            log_format: "text".to_string(),
            cors_enabled: true,
            max_body_size: 10 * 1024 * 1024,
            redis_url: std::env::var("RAIBID_REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            gitea_webhook_secret: std::env::var("RAIBID_GITEA_WEBHOOK_SECRET").ok(),
            github_webhook_secret: std::env::var("RAIBID_GITHUB_WEBHOOK_SECRET").ok(),
            rate_limit_rpm: 100,
        }
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("RAIBID_SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("RAIBID_SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            log_format: std::env::var("RAIBID_LOG_FORMAT").unwrap_or_else(|_| "text".to_string()),
            cors_enabled: std::env::var("RAIBID_CORS_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            max_body_size: std::env::var("RAIBID_MAX_BODY_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10 * 1024 * 1024),
            redis_url: std::env::var("RAIBID_REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            gitea_webhook_secret: std::env::var("RAIBID_GITEA_WEBHOOK_SECRET").ok(),
            github_webhook_secret: std::env::var("RAIBID_GITHUB_WEBHOOK_SECRET").ok(),
            rate_limit_rpm: std::env::var("RAIBID_RATE_LIMIT_RPM")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.log_format, "text");
        assert!(config.cors_enabled);
        assert_eq!(config.max_body_size, 10 * 1024 * 1024);
        assert_eq!(config.redis_url, "redis://127.0.0.1:6379");
        assert_eq!(config.rate_limit_rpm, 100);
    }

    #[test]
    fn test_from_env_with_defaults() {
        // Just test that it doesn't panic when vars are not set
        let config = ServerConfig::from_env();
        // We can't assert specific values as env vars might be set in CI
        assert!(!config.host.is_empty());
        assert!(config.port > 0);
    }

    #[test]
    fn test_config_clone() {
        let config = ServerConfig::default();
        let cloned = config.clone();
        assert_eq!(config, cloned);
    }
}
