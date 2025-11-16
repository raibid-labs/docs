//! Redis Streams job consumer

use crate::config::AgentConfig;
use crate::error::{AgentError, AgentResult};
use crate::executor::JobExecutor;
use chrono::Utc;
use raibid_common::jobs::{Job, JobStatus};
use redis::aio::MultiplexedConnection;
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{AsyncCommands, Client, FromRedisValue};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Job message from Redis Streams
#[derive(Debug, Clone)]
pub struct JobMessage {
    /// Stream message ID
    pub id: String,
    /// Job data
    pub job: Job,
}

/// Redis Streams job consumer
pub struct JobConsumer {
    config: Arc<AgentConfig>,
    client: Client,
    executor: JobExecutor,
}

impl JobConsumer {
    /// Create a new job consumer
    pub async fn new(config: Arc<AgentConfig>) -> AgentResult<Self> {
        // Create Redis client
        let url = config.redis.connection_url();
        let client = Client::open(url.as_str()).map_err(|e| {
            AgentError::Configuration(format!("Failed to create Redis client: {}", e))
        })?;

        // Test connection
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(AgentError::RedisConnection)?;

        // Ping Redis to verify connection
        redis::cmd("PING")
            .query_async::<MultiplexedConnection, String>(&mut conn)
            .await?;

        info!("Successfully connected to Redis at {}", config.redis.host);

        // Create consumer group if it doesn't exist
        Self::ensure_consumer_group(&client, &config).await?;

        let executor = JobExecutor::new(config.clone());

        Ok(Self {
            config,
            client,
            executor,
        })
    }

    /// Ensure the consumer group exists
    async fn ensure_consumer_group(client: &Client, config: &AgentConfig) -> AgentResult<()> {
        let mut conn = client.get_multiplexed_async_connection().await?;

        // Try to create the consumer group
        // XGROUP CREATE stream group id [MKSTREAM]
        let result: Result<String, redis::RedisError> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(&config.redis.queue_stream)
            .arg(&config.redis.consumer_group)
            .arg("$") // Start from the end
            .arg("MKSTREAM") // Create stream if it doesn't exist
            .query_async(&mut conn)
            .await;

        match result {
            Ok(_) => {
                info!("Created consumer group: {}", config.redis.consumer_group);
                Ok(())
            }
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.contains("BUSYGROUP") || err_msg.contains("already exists") {
                    debug!(
                        "Consumer group already exists: {}",
                        config.redis.consumer_group
                    );
                    Ok(())
                } else {
                    Err(AgentError::RedisConnection(e))
                }
            }
        }
    }

    /// Run the consumer loop
    pub async fn run(self) -> AgentResult<()> {
        info!(
            "Starting job consumer loop for agent {}",
            self.config.agent_id
        );

        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let last_id = ">".to_string(); // Start with new messages

        loop {
            match self.poll_jobs(&mut conn, &last_id).await {
                Ok(messages) => {
                    if !messages.is_empty() {
                        info!("Received {} job(s)", messages.len());

                        for msg in messages {
                            // Process each job
                            if let Err(e) = self.process_job(&mut conn, &msg).await {
                                error!("Failed to process job {}: {}", msg.job.id, e);

                                // Try to mark job as failed
                                let _ = self
                                    .update_job_status(
                                        &mut conn,
                                        &msg.job.id,
                                        JobStatus::Failed,
                                        Some(format!("Error: {}", e)),
                                    )
                                    .await;

                                // Acknowledge the message even on failure to avoid reprocessing
                                let _ = self.acknowledge_message(&mut conn, &msg.id).await;
                            } else {
                                // Acknowledge successful processing
                                if let Err(e) = self.acknowledge_message(&mut conn, &msg.id).await {
                                    error!("Failed to acknowledge message {}: {}", msg.id, e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error polling jobs: {}", e);
                    // Wait before retrying
                    tokio::time::sleep(Duration::from_millis(self.config.poll_interval_ms * 2))
                        .await;
                }
            }

            // Wait before next poll
            tokio::time::sleep(Duration::from_millis(self.config.poll_interval_ms)).await;
        }
    }

    /// Poll for new jobs from Redis Streams
    async fn poll_jobs(
        &self,
        conn: &mut MultiplexedConnection,
        last_id: &str,
    ) -> AgentResult<Vec<JobMessage>> {
        // XREADGROUP GROUP group consumer [COUNT count] [BLOCK milliseconds] STREAMS key [key ...] id [id ...]
        let opts = StreamReadOptions::default()
            .group(&self.config.redis.consumer_group, &self.config.agent_id)
            .count(self.config.max_concurrent_jobs)
            .block(self.config.poll_interval_ms as usize);

        let streams: StreamReadReply = conn
            .xread_options(&[&self.config.redis.queue_stream], &[last_id], &opts)
            .await?;

        let mut messages = Vec::new();

        for stream_key in streams.keys {
            for stream_id in stream_key.ids {
                match Self::parse_job_message(&stream_id.id, &stream_id.map) {
                    Ok(msg) => messages.push(msg),
                    Err(e) => {
                        warn!("Failed to parse job message {}: {}", stream_id.id, e);
                        // Acknowledge invalid messages to remove them from the stream
                        let _ = self.acknowledge_message(conn, &stream_id.id).await;
                    }
                }
            }
        }

        Ok(messages)
    }

    /// Parse a job message from Redis stream data
    fn parse_job_message(
        id: &str,
        data: &HashMap<String, redis::Value>,
    ) -> AgentResult<JobMessage> {
        // Extract job JSON from the stream data
        let job_json = data
            .get("job")
            .and_then(|v| {
                // Convert Redis Value to String using FromRedisValue
                String::from_redis_value(v).ok()
            })
            .ok_or_else(|| AgentError::JobParsing("Missing 'job' field in message".to_string()))?;

        // Parse job from JSON
        let job: Job = serde_json::from_str(&job_json)
            .map_err(|e| AgentError::JobParsing(format!("Invalid job JSON: {}", e)))?;

        Ok(JobMessage {
            id: id.to_string(),
            job,
        })
    }

    /// Process a single job
    async fn process_job(
        &self,
        conn: &mut MultiplexedConnection,
        msg: &JobMessage,
    ) -> AgentResult<()> {
        let job_id = &msg.job.id;
        info!("Processing job: {}", job_id);

        // Update job status to running
        self.update_job_status(conn, job_id, JobStatus::Running, None)
            .await?;

        // Execute the job
        let result = self.executor.execute(&msg.job).await;

        // Update final status based on result
        match result {
            Ok(exit_code) => {
                let status = if exit_code == 0 {
                    JobStatus::Success
                } else {
                    JobStatus::Failed
                };

                self.update_job_status(
                    conn,
                    job_id,
                    status,
                    Some(format!("Exit code: {}", exit_code)),
                )
                .await?;

                info!("Job {} completed with exit code {}", job_id, exit_code);
            }
            Err(e) => {
                self.update_job_status(
                    conn,
                    job_id,
                    JobStatus::Failed,
                    Some(format!("Execution error: {}", e)),
                )
                .await?;

                error!("Job {} failed: {}", job_id, e);
            }
        }

        Ok(())
    }

    /// Update job status in Redis
    async fn update_job_status(
        &self,
        conn: &mut MultiplexedConnection,
        job_id: &str,
        status: JobStatus,
        message: Option<String>,
    ) -> AgentResult<()> {
        let status_key = format!("raibid:job:{}:status", job_id);
        let status_value = serde_json::json!({
            "status": status.as_str(),
            "agent_id": self.config.agent_id,
            "updated_at": Utc::now().to_rfc3339(),
            "message": message,
        });

        let _: () = conn.set(&status_key, status_value.to_string()).await?;

        // Set TTL on status key (24 hours)
        let _: () = conn.expire(&status_key, 86400).await?;

        debug!("Updated job {} status to {}", job_id, status);

        Ok(())
    }

    /// Acknowledge a processed message
    async fn acknowledge_message(
        &self,
        conn: &mut MultiplexedConnection,
        message_id: &str,
    ) -> AgentResult<()> {
        // XACK key group id [id ...]
        let _: i32 = redis::cmd("XACK")
            .arg(&self.config.redis.queue_stream)
            .arg(&self.config.redis.consumer_group)
            .arg(message_id)
            .query_async(conn)
            .await?;

        debug!("Acknowledged message: {}", message_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_job_message() {
        let mut data = HashMap::new();
        let job = Job {
            id: "test-job-1".to_string(),
            repo: "test/repo".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
            status: JobStatus::Pending,
            started_at: Utc::now(),
            finished_at: None,
            duration: None,
            agent_id: None,
            exit_code: None,
        };

        let job_json = serde_json::to_string(&job).unwrap();
        data.insert("job".to_string(), redis::Value::Data(job_json.into_bytes()));

        let msg = JobConsumer::parse_job_message("1234567890-0", &data).unwrap();
        assert_eq!(msg.id, "1234567890-0");
        assert_eq!(msg.job.id, "test-job-1");
        assert_eq!(msg.job.repo, "test/repo");
    }

    #[test]
    fn test_parse_job_message_missing_field() {
        let data = HashMap::new();
        let result = JobConsumer::parse_job_message("1234567890-0", &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_job_message_invalid_json() {
        let mut data = HashMap::new();
        data.insert(
            "job".to_string(),
            redis::Value::Data(b"invalid json".to_vec()),
        );

        let result = JobConsumer::parse_job_message("1234567890-0", &data);
        assert!(result.is_err());
    }
}
