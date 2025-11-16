//! Job management commands
//!
//! This module implements CLI commands for managing CI/CD jobs including
//! listing jobs, viewing details, triggering builds, canceling jobs, and viewing logs.

use anyhow::{Context, Result};
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, CellAlignment, ContentArrangement, Table};
use raibid_common::{Job, JobListQuery, JobStatus, JobTrigger};
use serde_json;
use std::time::Duration;

use crate::api::ApiClient;
use crate::cli::{JobsCommand, JobsSubcommand};

/// Handle jobs command
pub fn handle(cmd: &JobsCommand) -> Result<()> {
    match &cmd.command {
        JobsSubcommand::List {
            status,
            repo,
            branch,
            limit,
            offset,
            json,
        } => list_jobs(status, repo, branch, *limit, *offset, *json),
        JobsSubcommand::Show { job_id, json } => show_job(job_id, *json),
        JobsSubcommand::Logs {
            job_id,
            follow,
            tail,
        } => show_logs(job_id, *follow, *tail),
        JobsSubcommand::Trigger {
            repo,
            branch,
            commit,
            json,
        } => trigger_job(repo, branch, commit.as_deref(), *json),
        JobsSubcommand::Cancel { job_id, json } => cancel_job(job_id, *json),
    }
}

/// List jobs with optional filters
fn list_jobs(
    status: &Option<String>,
    repo: &Option<String>,
    branch: &Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    json: bool,
) -> Result<()> {
    let client = ApiClient::from_env().context("Failed to create API client")?;

    // Parse status filter
    let status_filter = if let Some(s) = status {
        Some(s.parse::<JobStatus>().context("Invalid status value")?)
    } else {
        None
    };

    let query = JobListQuery {
        status: status_filter,
        repo: repo.clone(),
        branch: branch.clone(),
        limit,
        offset,
    };

    let job_list = client.list_jobs(&query).context("Failed to fetch jobs")?;

    if json {
        // Output as JSON
        let json_str =
            serde_json::to_string_pretty(&job_list).context("Failed to serialize jobs to JSON")?;
        println!("{}", json_str);
    } else {
        // Output as formatted table
        if job_list.jobs.is_empty() {
            println!("{}", "No jobs found.".yellow());
            return Ok(());
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("ID").set_alignment(CellAlignment::Center),
                Cell::new("Repository").set_alignment(CellAlignment::Left),
                Cell::new("Branch").set_alignment(CellAlignment::Left),
                Cell::new("Status").set_alignment(CellAlignment::Center),
                Cell::new("Started").set_alignment(CellAlignment::Left),
                Cell::new("Duration").set_alignment(CellAlignment::Right),
            ]);

        for job in &job_list.jobs {
            let status_cell = format!("{} {}", job.status.icon(), job.status.as_str());
            let status_colored = match job.status {
                JobStatus::Success => status_cell.green(),
                JobStatus::Failed => status_cell.red(),
                JobStatus::Running => status_cell.blue(),
                JobStatus::Pending => status_cell.yellow(),
                JobStatus::Cancelled => status_cell.truecolor(128, 128, 128),
            };

            table.add_row(vec![
                Cell::new(&job.id),
                Cell::new(&job.repo),
                Cell::new(&job.branch),
                Cell::new(status_colored),
                Cell::new(format_timestamp(&job.started_at)),
                Cell::new(job.duration_string()),
            ]);
        }

        println!("{}", table);
        println!(
            "\n{} Showing {} of {} jobs (offset: {})",
            "Info:".cyan().bold(),
            job_list.jobs.len(),
            job_list.total,
            job_list.offset
        );
    }

    Ok(())
}

/// Show detailed information about a specific job
fn show_job(job_id: &str, json: bool) -> Result<()> {
    let client = ApiClient::from_env().context("Failed to create API client")?;
    let job = client.get_job(job_id).context("Failed to fetch job")?;

    if json {
        let json_str =
            serde_json::to_string_pretty(&job).context("Failed to serialize job to JSON")?;
        println!("{}", json_str);
    } else {
        print_job_details(&job);
    }

    Ok(())
}

/// Show logs for a specific job
fn show_logs(job_id: &str, follow: bool, tail: Option<usize>) -> Result<()> {
    let client = ApiClient::from_env().context("Failed to create API client")?;

    if follow {
        // Follow mode: poll for new logs
        println!(
            "{} Following logs for job {}...",
            "Info:".cyan().bold(),
            job_id
        );
        println!("{}", "─".repeat(80).truecolor(100, 100, 100));

        let mut last_line_count = 0;

        loop {
            let logs = client
                .get_job_logs(job_id, None)
                .context("Failed to fetch logs")?;

            // Print new lines only
            for entry in logs.entries.iter().skip(last_line_count) {
                println!(
                    "[{}] {}",
                    entry
                        .timestamp
                        .format("%H:%M:%S")
                        .to_string()
                        .truecolor(150, 150, 150),
                    entry.message
                );
            }

            last_line_count = logs.entries.len();

            // Check if job is finished
            let job = client
                .get_job(job_id)
                .context("Failed to fetch job status")?;
            if job.status.is_terminal() {
                println!("{}", "─".repeat(80).truecolor(100, 100, 100));
                println!(
                    "{} Job finished with status: {}",
                    "Info:".cyan().bold(),
                    format_status(&job.status)
                );
                break;
            }

            // Wait before next poll
            std::thread::sleep(Duration::from_secs(2));
        }
    } else {
        // One-time fetch
        let logs = client
            .get_job_logs(job_id, tail)
            .context("Failed to fetch logs")?;

        if logs.entries.is_empty() {
            println!("{}", "No logs available.".yellow());
            return Ok(());
        }

        println!("{} Logs for job {}:", "Info:".cyan().bold(), job_id);
        println!("{}", "─".repeat(80).truecolor(100, 100, 100));

        for entry in &logs.entries {
            println!(
                "[{}] {}",
                entry
                    .timestamp
                    .format("%H:%M:%S")
                    .to_string()
                    .truecolor(150, 150, 150),
                entry.message
            );
        }
    }

    Ok(())
}

/// Trigger a new job
fn trigger_job(repo: &str, branch: &str, commit: Option<&str>, json: bool) -> Result<()> {
    let client = ApiClient::from_env().context("Failed to create API client")?;

    let trigger = JobTrigger {
        repo: repo.to_string(),
        branch: branch.to_string(),
        commit: commit.map(|s| s.to_string()),
    };

    println!(
        "{} Triggering build for {}/{} {}...",
        "Info:".cyan().bold(),
        repo,
        branch,
        commit.map(|c| format!("({})", c)).unwrap_or_default()
    );

    let job = client
        .trigger_job(&trigger)
        .context("Failed to trigger job")?;

    if json {
        let json_str =
            serde_json::to_string_pretty(&job).context("Failed to serialize job to JSON")?;
        println!("{}", json_str);
    } else {
        println!("{} Job created successfully!", "Success:".green().bold());
        print_job_details(&job);
    }

    Ok(())
}

/// Cancel a job
fn cancel_job(job_id: &str, json: bool) -> Result<()> {
    let client = ApiClient::from_env().context("Failed to create API client")?;

    println!("{} Cancelling job {}...", "Info:".cyan().bold(), job_id);

    let job = client.cancel_job(job_id).context("Failed to cancel job")?;

    if json {
        let json_str =
            serde_json::to_string_pretty(&job).context("Failed to serialize job to JSON")?;
        println!("{}", json_str);
    } else {
        println!("{} Job cancelled successfully!", "Success:".green().bold());
        print_job_details(&job);
    }

    Ok(())
}

/// Print detailed job information
fn print_job_details(job: &Job) {
    println!("\n{}", "Job Details".cyan().bold().underline());
    println!("{:<15} {}", "ID:", job.id);
    println!("{:<15} {}", "Repository:", job.repo);
    println!("{:<15} {}", "Branch:", job.branch);
    println!("{:<15} {}", "Commit:", job.commit);
    println!("{:<15} {}", "Status:", format_status(&job.status));
    println!("{:<15} {}", "Started:", format_timestamp(&job.started_at));

    if let Some(finished) = job.finished_at {
        println!("{:<15} {}", "Finished:", format_timestamp(&finished));
    }

    println!("{:<15} {}", "Duration:", job.duration_string());

    if let Some(agent_id) = &job.agent_id {
        println!("{:<15} {}", "Agent:", agent_id);
    }

    if let Some(exit_code) = job.exit_code {
        let exit_str = if exit_code == 0 {
            exit_code.to_string().green()
        } else {
            exit_code.to_string().red()
        };
        println!("{:<15} {}", "Exit Code:", exit_str);
    }
}

/// Format job status with color
fn format_status(status: &JobStatus) -> colored::ColoredString {
    let status_str = format!("{} {}", status.icon(), status.as_str());
    match status {
        JobStatus::Success => status_str.green(),
        JobStatus::Failed => status_str.red(),
        JobStatus::Running => status_str.blue(),
        JobStatus::Pending => status_str.yellow(),
        JobStatus::Cancelled => status_str.truecolor(128, 128, 128),
    }
}

/// Format timestamp as relative time
fn format_timestamp(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(*timestamp);

    if duration.num_seconds() < 60 {
        format!("{}s ago", duration.num_seconds())
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else {
        format!("{}d ago", duration.num_days())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_format_status() {
        let status = JobStatus::Success;
        let formatted = format_status(&status);
        assert!(formatted.to_string().contains("Success"));
    }

    #[test]
    fn test_format_timestamp() {
        let now = Utc::now();
        let result = format_timestamp(&now);
        assert!(result.contains("ago") || result.contains("0s ago"));
    }
}
