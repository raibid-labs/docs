//! raibid-agent
//!
//! CI agent runner that polls the job queue and executes builds.
//! This crate handles:
//! - Job polling from Redis Streams
//! - Build execution in isolated environments
//! - Cache management for dependencies
//! - Result reporting back to the server
//! - Complete Rust build pipeline (check, test, build, clippy, audit)
//! - Docker image building and publishing
//! - Log streaming to Redis

#![allow(dead_code)]

use std::sync::Arc;

pub mod config;
pub mod consumer;
pub mod error;
pub mod executor;
pub mod git;
pub mod pipeline;

// Re-export commonly used types
pub use config::{AgentConfig, RedisConfig};
pub use consumer::{JobConsumer, JobMessage};
pub use error::{AgentError, AgentResult};
pub use executor::JobExecutor;
pub use git::GitManager;
pub use pipeline::{
    ArtifactMetadata, BuildStep, PipelineConfig, PipelineExecutor, PipelineResult, StepResult,
};

/// Main Agent structure that orchestrates the CI agent
pub struct Agent {
    config: Arc<AgentConfig>,
    consumer: JobConsumer,
}

impl Agent {
    /// Create a new agent instance
    pub async fn new(config: AgentConfig) -> AgentResult<Self> {
        let config = Arc::new(config);
        let consumer = JobConsumer::new(config.clone()).await?;

        Ok(Self { config, consumer })
    }

    /// Run the agent
    pub async fn run(self) -> AgentResult<()> {
        self.consumer.run().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert!(!config.agent_id.is_empty());
        assert_eq!(config.max_concurrent_jobs, 1);
    }
}
