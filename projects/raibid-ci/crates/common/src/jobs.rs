//! Job types and data structures
//!
//! This module provides common job-related types used across the raibid-ci workspace.
//! These types are shared between the CLI, TUI, server, and agent components.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Job execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Job is waiting to be executed
    Pending,
    /// Job is currently running
    Running,
    /// Job completed successfully
    Success,
    /// Job failed
    Failed,
    /// Job was cancelled by user
    Cancelled,
}

impl JobStatus {
    /// Get a display string for the status
    pub fn as_str(&self) -> &str {
        match self {
            JobStatus::Pending => "Pending",
            JobStatus::Running => "Running",
            JobStatus::Success => "Success",
            JobStatus::Failed => "Failed",
            JobStatus::Cancelled => "Cancelled",
        }
    }

    /// Get an icon/symbol for the status
    pub fn icon(&self) -> &str {
        match self {
            JobStatus::Pending => "⏳",
            JobStatus::Running => "▶",
            JobStatus::Success => "✓",
            JobStatus::Failed => "✗",
            JobStatus::Cancelled => "⊘",
        }
    }

    /// Check if the job is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            JobStatus::Success | JobStatus::Failed | JobStatus::Cancelled
        )
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for JobStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(JobStatus::Pending),
            "running" => Ok(JobStatus::Running),
            "success" => Ok(JobStatus::Success),
            "failed" => Ok(JobStatus::Failed),
            "cancelled" => Ok(JobStatus::Cancelled),
            _ => Err(anyhow::anyhow!("Invalid job status: {}", s)),
        }
    }
}

/// Complete job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique job identifier
    pub id: String,
    /// Repository name
    pub repo: String,
    /// Git branch
    pub branch: String,
    /// Git commit SHA
    pub commit: String,
    /// Current job status
    pub status: JobStatus,
    /// Job start time
    pub started_at: DateTime<Utc>,
    /// Job completion time (if finished)
    pub finished_at: Option<DateTime<Utc>>,
    /// Job duration in seconds (if finished)
    pub duration: Option<u64>,
    /// Agent ID assigned to this job
    pub agent_id: Option<String>,
    /// Exit code (if finished)
    pub exit_code: Option<i32>,
}

impl Job {
    /// Calculate duration in seconds
    pub fn calculate_duration(&self) -> Option<u64> {
        self.finished_at
            .map(|finished| (finished - self.started_at).num_seconds() as u64)
    }

    /// Get human-readable duration string
    pub fn duration_string(&self) -> String {
        match self.calculate_duration().or(self.duration) {
            Some(secs) => format_duration(secs),
            None => {
                if self.status == JobStatus::Running {
                    let elapsed = (Utc::now() - self.started_at).num_seconds() as u64;
                    format!("{}...", format_duration(elapsed))
                } else {
                    "-".to_string()
                }
            }
        }
    }
}

/// Format duration in seconds to human-readable string
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Job list query parameters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobListQuery {
    /// Filter by job status
    pub status: Option<JobStatus>,
    /// Filter by repository name
    pub repo: Option<String>,
    /// Filter by branch name
    pub branch: Option<String>,
    /// Maximum number of jobs to return
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Job list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobList {
    /// List of jobs
    pub jobs: Vec<Job>,
    /// Total number of jobs matching the query
    pub total: usize,
    /// Offset used for this page
    pub offset: usize,
    /// Limit used for this page
    pub limit: usize,
}

/// Job trigger request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobTrigger {
    /// Repository to build
    pub repo: String,
    /// Branch to build
    pub branch: String,
    /// Commit SHA to build (optional, defaults to latest)
    pub commit: Option<String>,
}

/// Job log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobLogEntry {
    /// Timestamp of the log entry
    pub timestamp: DateTime<Utc>,
    /// Log message
    pub message: String,
}

/// Job logs response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobLogs {
    /// Job ID
    pub job_id: String,
    /// Log entries
    pub entries: Vec<JobLogEntry>,
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
        assert_eq!(JobStatus::Cancelled.as_str(), "Cancelled");
    }

    #[test]
    fn test_job_status_parse() {
        assert_eq!("pending".parse::<JobStatus>().unwrap(), JobStatus::Pending);
        assert_eq!("running".parse::<JobStatus>().unwrap(), JobStatus::Running);
        assert_eq!("success".parse::<JobStatus>().unwrap(), JobStatus::Success);
        assert_eq!("failed".parse::<JobStatus>().unwrap(), JobStatus::Failed);
        assert_eq!(
            "cancelled".parse::<JobStatus>().unwrap(),
            JobStatus::Cancelled
        );
    }

    #[test]
    fn test_job_status_is_terminal() {
        assert!(!JobStatus::Pending.is_terminal());
        assert!(!JobStatus::Running.is_terminal());
        assert!(JobStatus::Success.is_terminal());
        assert!(JobStatus::Failed.is_terminal());
        assert!(JobStatus::Cancelled.is_terminal());
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m");
    }
}
