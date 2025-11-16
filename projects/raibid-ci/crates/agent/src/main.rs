//! raibid-agent binary
//!
//! CI agent that consumes jobs from Redis Streams and executes builds.

use raibid_agent::{Agent, AgentConfig};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    if let Err(e) = init_tracing() {
        eprintln!("Failed to initialize tracing: {}", e);
        std::process::exit(1);
    }

    // Load configuration
    let config = match load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    info!("Starting raibid-agent with config: {:?}", config);

    // Create and run the agent
    match Agent::new(config).await {
        Ok(agent) => {
            if let Err(e) = agent.run().await {
                error!("Agent error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("Failed to create agent: {}", e);
            std::process::exit(1);
        }
    }

    info!("Agent shut down successfully");
}

/// Initialize tracing/logging
fn init_tracing() -> anyhow::Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "raibid_agent=info,raibid_common=info".into());

    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string());

    if log_format == "json" {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    Ok(())
}

/// Load agent configuration from environment and defaults
fn load_config() -> anyhow::Result<AgentConfig> {
    let mut config = AgentConfig::default();

    // Load from environment variables
    if let Ok(agent_id) = std::env::var("AGENT_ID") {
        config.agent_id = agent_id;
    }

    if let Ok(redis_host) = std::env::var("REDIS_HOST") {
        config.redis.host = redis_host;
    }

    if let Ok(redis_port) = std::env::var("REDIS_PORT") {
        config.redis.port = redis_port.parse()?;
    }

    if let Ok(redis_password) = std::env::var("REDIS_PASSWORD") {
        config.redis.password = Some(redis_password);
    }

    if let Ok(queue_stream) = std::env::var("QUEUE_STREAM") {
        config.redis.queue_stream = queue_stream;
    }

    if let Ok(consumer_group) = std::env::var("CONSUMER_GROUP") {
        config.redis.consumer_group = consumer_group;
    }

    if let Ok(workspace_dir) = std::env::var("WORKSPACE_DIR") {
        config.workspace_dir = workspace_dir.into();
    }

    if let Ok(max_concurrent) = std::env::var("MAX_CONCURRENT_JOBS") {
        config.max_concurrent_jobs = max_concurrent.parse()?;
    }

    if let Ok(poll_interval) = std::env::var("POLL_INTERVAL_MS") {
        config.poll_interval_ms = poll_interval.parse()?;
    }

    Ok(config)
}
