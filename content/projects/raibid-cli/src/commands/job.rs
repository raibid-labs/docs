//! Job management commands implementation

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use comfy_table::{presets::ASCII_FULL, Cell, Color, Table};
use serde::{Deserialize, Serialize};
use colored::Colorize;
use dialoguer::Confirm;

use crate::cli::JobCommands;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub repository: String,
    pub branch: String,
    pub status: JobStatus,
    pub started_at: DateTime<Utc>,
    pub duration_secs: u64,
    pub progress: Option<u8>,
    pub agent_id: Option<String>,
    pub commit_hash: String,
    pub commit_author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Running,
    Success,
    Failed,
    Pending,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Running => write!(f, "Running"),
            JobStatus::Success => write!(f, "Success"),
            JobStatus::Failed => write!(f, "Failed"),
            JobStatus::Pending => write!(f, "Pending"),
        }
    }
}

pub fn execute(cmd: JobCommands) -> Result<()> {
    match cmd {
        JobCommands::List { status, repo, limit, output } => list_jobs(status, repo, limit, &output),
        JobCommands::Show { job_id, output } => show_job(&job_id, &output),
        JobCommands::Cancel { job_id, force } => cancel_job(&job_id, force),
        JobCommands::Retry { job_id } => retry_job(&job_id),
    }
}

fn generate_mock_jobs() -> Vec<Job> {
    let now = Utc::now();
    vec![
        Job {
            id: "a1b2c3".to_string(),
            repository: "raibid/core".to_string(),
            branch: "feature/ci".to_string(),
            status: JobStatus::Running,
            started_at: now - Duration::minutes(2),
            duration_secs: 135,
            progress: Some(45),
            agent_id: Some("rust-builder-1".to_string()),
            commit_hash: "abc123def456".to_string(),
            commit_author: "alice@example.com".to_string(),
        },
        Job {
            id: "d4e5f6".to_string(),
            repository: "raibid/cli".to_string(),
            branch: "main".to_string(),
            status: JobStatus::Success,
            started_at: now - Duration::minutes(15),
            duration_secs: 522,
            progress: Some(100),
            agent_id: Some("rust-builder-2".to_string()),
            commit_hash: "789ghi012jkl".to_string(),
            commit_author: "bob@example.com".to_string(),
        },
        Job {
            id: "g7h8i9".to_string(),
            repository: "raibid/core".to_string(),
            branch: "fix-bug".to_string(),
            status: JobStatus::Failed,
            started_at: now - Duration::hours(1),
            duration_secs: 723,
            progress: Some(67),
            agent_id: Some("rust-builder-1".to_string()),
            commit_hash: "mno345pqr678".to_string(),
            commit_author: "charlie@example.com".to_string(),
        },
        Job {
            id: "j1k2l3".to_string(),
            repository: "raibid/api".to_string(),
            branch: "develop".to_string(),
            status: JobStatus::Pending,
            started_at: now,
            duration_secs: 0,
            progress: None,
            agent_id: None,
            commit_hash: "stu901vwx234".to_string(),
            commit_author: "diana@example.com".to_string(),
        },
        Job {
            id: "m4n5o6".to_string(),
            repository: "raibid/docs".to_string(),
            branch: "main".to_string(),
            status: JobStatus::Success,
            started_at: now - Duration::hours(2),
            duration_secs: 180,
            progress: Some(100),
            agent_id: Some("rust-builder-3".to_string()),
            commit_hash: "yz5abc678def".to_string(),
            commit_author: "eve@example.com".to_string(),
        },
    ]
}

fn list_jobs(status_filter: Option<String>, repo_filter: Option<String>, limit: usize, output: &str) -> Result<()> {
    let mut jobs = generate_mock_jobs();
    if let Some(status) = status_filter {
        let status_lower = status.to_lowercase();
        jobs.retain(|job| job.status.to_string().to_lowercase() == status_lower);
    }
    if let Some(repo) = repo_filter {
        jobs.retain(|job| job.repository.contains(&repo));
    }
    jobs.truncate(limit);
    if output == "json" {
        let json = serde_json::to_string_pretty(&jobs)?;
        println!("{}", json);
    } else {
        print_jobs_table(&jobs);
    }
    Ok(())
}

fn print_jobs_table(jobs: &[Job]) {
    let mut table = Table::new();
    table.load_preset(ASCII_FULL).set_header(vec!["ID", "REPOSITORY", "BRANCH", "STATUS", "STARTED", "DURATION"]);
    for job in jobs {
        let status_cell = match job.status {
            JobStatus::Running => Cell::new(job.status.to_string()).fg(Color::Yellow),
            JobStatus::Success => Cell::new(job.status.to_string()).fg(Color::Green),
            JobStatus::Failed => Cell::new(job.status.to_string()).fg(Color::Red),
            JobStatus::Pending => Cell::new(job.status.to_string()).fg(Color::Grey),
        };
        table.add_row(vec![
            Cell::new(&job.id),
            Cell::new(&job.repository),
            Cell::new(&job.branch),
            status_cell,
            Cell::new(format_relative_time(job.started_at)),
            Cell::new(format_duration(job.duration_secs)),
        ]);
    }
    println!("{}", table);
}

fn show_job(job_id: &str, output: &str) -> Result<()> {
    let jobs = generate_mock_jobs();
    let job = jobs.iter().find(|j| j.id == job_id).ok_or_else(|| anyhow::anyhow!("Job not found: {}", job_id))?;
    if output == "json" {
        let json = serde_json::to_string_pretty(&job)?;
        println!("{}", json);
    } else {
        print_job_details(job);
    }
    Ok(())
}

fn print_job_details(job: &Job) {
    println!("\n{}", "=== Job Details ===".bold());
    println!("{:15} {}", "ID:", job.id);
    println!("{:15} {}", "Repository:", job.repository);
    println!("{:15} {}", "Branch:", job.branch);
    let status_str = match job.status {
        JobStatus::Running => job.status.to_string().yellow().to_string(),
        JobStatus::Success => job.status.to_string().green().to_string(),
        JobStatus::Failed => job.status.to_string().red().to_string(),
        JobStatus::Pending => job.status.to_string().bright_black().to_string(),
    };
    println!("{:15} {}", "Status:", status_str);
    println!("{:15} {}", "Started:", job.started_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("{:15} {}", "Duration:", format_duration(job.duration_secs));
    if let Some(progress) = job.progress {
        println!("{:15} {}%", "Progress:", progress);
    }
    if let Some(agent_id) = &job.agent_id {
        println!("{:15} {}", "Agent:", agent_id);
    } else {
        println!("{:15} {}", "Agent:", "None (pending)");
    }
    println!("{:15} {}", "Commit:", job.commit_hash);
    println!("{:15} {}", "Author:", job.commit_author);
    println!();
}

fn cancel_job(job_id: &str, force: bool) -> Result<()> {
    let jobs = generate_mock_jobs();
    let job = jobs.iter().find(|j| j.id == job_id).ok_or_else(|| anyhow::anyhow!("Job not found: {}", job_id))?;
    if !force {
        let progress_str = job.progress.map(|p| format!(", {}% complete", p)).unwrap_or_default();
        let confirmed = Confirm::new()
            .with_prompt(format!("Cancel job {} ({}, running for {}{})?", job_id, job.status, format_duration(job.duration_secs), progress_str))
            .default(false)
            .interact()?;
        if !confirmed {
            println!("Cancelled operation.");
            return Ok(());
        }
    }
    println!("{} Cancelled job: {}", "✓".green(), job_id);
    println!("  Repository: {}", job.repository);
    println!("  Branch: {}", job.branch);
    Ok(())
}

fn retry_job(job_id: &str) -> Result<()> {
    let jobs = generate_mock_jobs();
    let job = jobs.iter().find(|j| j.id == job_id).ok_or_else(|| anyhow::anyhow!("Job not found: {}", job_id))?;
    println!("{} Retrying job: {}", "✓".green(), job_id);
    println!("  Repository: {}", job.repository);
    println!("  Branch: {}", job.branch);
    println!("  New job ID: {}", format!("{}-retry", job_id).bright_blue());
    Ok(())
}

fn format_duration(secs: u64) -> String {
    let minutes = secs / 60;
    let seconds = secs % 60;
    if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

fn format_relative_time(timestamp: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(timestamp);
    if diff.num_seconds() < 60 {
        format!("{}s ago", diff.num_seconds())
    } else if diff.num_minutes() < 60 {
        format!("{}m ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h ago", diff.num_hours())
    } else {
        format!("{}d ago", diff.num_days())
    }
}
