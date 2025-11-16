//! Agent configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique agent identifier
    pub agent_id: String,

    /// Redis connection configuration
    pub redis: RedisConfig,

    /// Workspace directory for cloning repositories
    pub workspace_dir: PathBuf,

    /// Maximum number of concurrent jobs
    pub max_concurrent_jobs: usize,

    /// Poll interval in milliseconds
    pub poll_interval_ms: u64,

    /// Maximum number of retry attempts for failed jobs
    pub max_retries: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        let agent_id = uuid::Uuid::new_v4().to_string();
        let workspace_dir = std::env::temp_dir().join("raibid-agent").join(&agent_id);

        Self {
            agent_id,
            redis: RedisConfig::default(),
            workspace_dir,
            max_concurrent_jobs: 1,
            poll_interval_ms: 1000, // 1 second
            max_retries: 3,
        }
    }
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis host
    pub host: String,

    /// Redis port
    pub port: u16,

    /// Redis password (optional)
    pub password: Option<String>,

    /// Job queue stream name
    pub queue_stream: String,

    /// Consumer group name
    pub consumer_group: String,

    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            host: "raibid-redis-master.raibid-redis.svc.cluster.local".to_string(),
            port: 6379,
            password: None,
            queue_stream: "raibid:jobs".to_string(),
            consumer_group: "raibid-workers".to_string(),
            connection_timeout_secs: 30,
        }
    }
}

impl RedisConfig {
    /// Get Redis connection URL
    pub fn connection_url(&self) -> String {
        if let Some(ref pwd) = self.password {
            format!("redis://:{}@{}:{}", pwd, self.host, self.port)
        } else {
            format!("redis://{}:{}", self.host, self.port)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AgentConfig::default();
        assert!(!config.agent_id.is_empty());
        assert_eq!(config.max_concurrent_jobs, 1);
        assert_eq!(config.poll_interval_ms, 1000);
    }

    #[test]
    fn test_redis_connection_url() {
        let config = RedisConfig {
            host: "localhost".to_string(),
            port: 6379,
            password: Some("secret".to_string()),
            ..Default::default()
        };
        assert_eq!(config.connection_url(), "redis://:secret@localhost:6379");

        let config_no_auth = RedisConfig {
            host: "localhost".to_string(),
            port: 6379,
            password: None,
            ..Default::default()
        };
        assert_eq!(config_no_auth.connection_url(), "redis://localhost:6379");
    }
}
