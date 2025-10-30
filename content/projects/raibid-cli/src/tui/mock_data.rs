//! Mock data generators for TUI development and testing
//!
//! This module provides mock data structures and generators for simulating
//! CI/CD job execution, agent status, and queue metrics.

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Job execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job is waiting to be executed
    Pending,
    /// Job is currently running
    Running,
    /// Job completed successfully
    Success,
    /// Job failed
    Failed,
}

impl JobStatus {
    /// Get a display string for the status
    pub fn as_str(&self) -> &str {
        match self {
            JobStatus::Pending => "Pending",
            JobStatus::Running => "Running",
            JobStatus::Success => "Success",
            JobStatus::Failed => "Failed",
        }
    }

    /// Get an icon/symbol for the status
    pub fn icon(&self) -> &str {
        match self {
            JobStatus::Pending => "⏳",
            JobStatus::Running => "▶",
            JobStatus::Success => "✓",
            JobStatus::Failed => "✗",
        }
    }
}

/// Mock CI job data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockJob {
    /// Unique job identifier
    pub id: String,
    /// Repository name
    pub repo: String,
    /// Git branch
    pub branch: String,
    /// Current job status
    pub status: JobStatus,
    /// Progress percentage (0-100)
    pub progress: u8,
    /// Job start time
    pub start_time: DateTime<Utc>,
    /// Job duration in seconds (if completed)
    pub duration: Option<u64>,
}

impl MockJob {
    /// Generate a random mock job
    pub fn random(rng: &mut impl Rng) -> Self {
        let repos = [
            "backend-api",
            "frontend-web",
            "mobile-app",
            "ml-pipeline",
            "data-service",
            "auth-service",
            "payment-gateway",
            "notification-service",
        ];

        let branches = [
            "main",
            "develop",
            "feature/new-api",
            "fix/memory-leak",
            "release/v2.0",
        ];

        let statuses = [
            JobStatus::Pending,
            JobStatus::Running,
            JobStatus::Success,
            JobStatus::Failed,
        ];

        let status = statuses[rng.gen_range(0..statuses.len())];
        let progress = match status {
            JobStatus::Pending => 0,
            JobStatus::Running => rng.gen_range(10..95),
            JobStatus::Success => 100,
            JobStatus::Failed => rng.gen_range(20..90),
        };

        let duration = match status {
            JobStatus::Success | JobStatus::Failed => Some(rng.gen_range(30..600)),
            _ => None,
        };

        let start_offset = rng.gen_range(0..3600);
        let start_time = Utc::now() - Duration::seconds(start_offset);

        Self {
            id: format!("job-{}", rng.gen_range(1000..9999)),
            repo: repos[rng.gen_range(0..repos.len())].to_string(),
            branch: branches[rng.gen_range(0..branches.len())].to_string(),
            status,
            progress,
            start_time,
            duration,
        }
    }
}

/// Agent execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Agent is idle and ready for work
    Idle,
    /// Agent is executing a job
    Busy,
    /// Agent is starting up
    Starting,
    /// Agent is shutting down
    Stopping,
}

impl AgentStatus {
    /// Get a display string for the status
    pub fn as_str(&self) -> &str {
        match self {
            AgentStatus::Idle => "Idle",
            AgentStatus::Busy => "Busy",
            AgentStatus::Starting => "Starting",
            AgentStatus::Stopping => "Stopping",
        }
    }
}

/// Mock CI agent data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockAgent {
    /// Unique agent identifier
    pub id: String,
    /// Agent display name
    pub name: String,
    /// Current agent status
    pub status: AgentStatus,
    /// CPU usage percentage (0-100)
    pub cpu: u8,
    /// Memory usage percentage (0-100)
    pub memory: u8,
    /// Uptime in seconds
    pub uptime: u64,
}

impl MockAgent {
    /// Generate a random mock agent
    pub fn random(rng: &mut impl Rng, index: usize) -> Self {
        let statuses = [
            AgentStatus::Idle,
            AgentStatus::Busy,
            AgentStatus::Starting,
            AgentStatus::Stopping,
        ];

        let status = statuses[rng.gen_range(0..statuses.len())];

        let (cpu, memory) = match status {
            AgentStatus::Idle => (rng.gen_range(5..20), rng.gen_range(10..30)),
            AgentStatus::Busy => (rng.gen_range(60..95), rng.gen_range(50..85)),
            AgentStatus::Starting => (rng.gen_range(10..30), rng.gen_range(20..40)),
            AgentStatus::Stopping => (rng.gen_range(5..15), rng.gen_range(15..35)),
        };

        Self {
            id: format!("agent-{:03}", index + 1),
            name: format!("dgx-agent-{:03}", index + 1),
            status,
            cpu,
            memory,
            uptime: rng.gen_range(300..86400),
        }
    }
}

/// Mock queue depth data for sparkline visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockQueueData {
    /// Historical queue depth values (60 data points for 1 minute of history)
    pub history: Vec<u64>,
    /// Current queue depth
    pub current: u64,
}

impl MockQueueData {
    /// Generate random queue data with realistic patterns
    pub fn random(rng: &mut impl Rng) -> Self {
        let mut history = Vec::with_capacity(60);
        let mut value = rng.gen_range(0..10) as f64;

        // Generate realistic queue depth variations
        for _ in 0..60 {
            // Random walk with bounds
            let change = rng.gen_range(-2.0..3.0);
            value = (value + change).clamp(0.0, 30.0);
            history.push(value as u64);
        }

        let current = *history.last().unwrap_or(&0);

        Self { history, current }
    }

    /// Update queue data with a new value (simulating real-time updates)
    pub fn update(&mut self, rng: &mut impl Rng) {
        if self.history.len() >= 60 {
            self.history.remove(0);
        }

        let change = rng.gen_range(-2.0..3.0);
        let new_value = (self.current as f64 + change).clamp(0.0, 30.0) as u64;
        self.history.push(new_value);
        self.current = new_value;
    }
}

/// Configuration for mock data generation
#[derive(Debug, Clone)]
pub struct MockDataConfig {
    /// Number of jobs to generate
    pub job_count: usize,
    /// Number of agents to generate
    pub agent_count: usize,
}

impl Default for MockDataConfig {
    fn default() -> Self {
        Self {
            job_count: 25,
            agent_count: 5,
        }
    }
}

/// Generate a complete set of mock data
pub fn generate_mock_data(
    config: &MockDataConfig,
) -> (Vec<MockJob>, Vec<MockAgent>, MockQueueData) {
    let mut rng = rand::thread_rng();

    let jobs: Vec<MockJob> = (0..config.job_count)
        .map(|_| MockJob::random(&mut rng))
        .collect();

    let agents: Vec<MockAgent> = (0..config.agent_count)
        .map(|i| MockAgent::random(&mut rng, i))
        .collect();

    let queue_data = MockQueueData::random(&mut rng);

    (jobs, agents, queue_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_display() {
        assert_eq!(JobStatus::Pending.as_str(), "Pending");
        assert_eq!(JobStatus::Running.as_str(), "Running");
        assert_eq!(JobStatus::Success.as_str(), "Success");
        assert_eq!(JobStatus::Failed.as_str(), "Failed");
    }

    #[test]
    fn test_job_generation() {
        let mut rng = rand::thread_rng();
        let job = MockJob::random(&mut rng);

        assert!(!job.id.is_empty());
        assert!(!job.repo.is_empty());
        assert!(!job.branch.is_empty());
        assert!(job.progress <= 100);
    }

    #[test]
    fn test_agent_generation() {
        let mut rng = rand::thread_rng();
        let agent = MockAgent::random(&mut rng, 0);

        assert!(!agent.id.is_empty());
        assert!(!agent.name.is_empty());
        assert!(agent.cpu <= 100);
        assert!(agent.memory <= 100);
    }

    #[test]
    fn test_queue_data_generation() {
        let mut rng = rand::thread_rng();
        let queue_data = MockQueueData::random(&mut rng);

        assert_eq!(queue_data.history.len(), 60);
        assert!(queue_data.current <= 30);
    }

    #[test]
    fn test_queue_data_update() {
        let mut rng = rand::thread_rng();
        let mut queue_data = MockQueueData::random(&mut rng);
        let initial_len = queue_data.history.len();

        queue_data.update(&mut rng);

        assert_eq!(queue_data.history.len(), initial_len);
        assert!(queue_data.current <= 30);
    }

    #[test]
    fn test_generate_mock_data() {
        let config = MockDataConfig::default();
        let (jobs, agents, _queue_data) = generate_mock_data(&config);

        assert_eq!(jobs.len(), config.job_count);
        assert_eq!(agents.len(), config.agent_count);
    }
}
