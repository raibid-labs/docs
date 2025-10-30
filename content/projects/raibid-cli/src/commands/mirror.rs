//! Repository mirroring commands implementation
//!
//! Mock implementations for repository mirroring operations

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use comfy_table::{presets::ASCII_FULL, Cell, Color, Table};
use serde::{Deserialize, Serialize};
use colored::Colorize;
use dialoguer::Confirm;

use crate::cli::MirrorCommands;

/// Mock mirror data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mirror {
    pub repository: String,
    pub name: String,
    pub sync_interval_minutes: u32,
    pub last_sync: DateTime<Utc>,
    pub status: MirrorStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MirrorStatus {
    Synced,
    Syncing,
    Error,
}

impl std::fmt::Display for MirrorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MirrorStatus::Synced => write!(f, "Synced"),
            MirrorStatus::Syncing => write!(f, "Syncing"),
            MirrorStatus::Error => write!(f, "Error"),
        }
    }
}

/// Execute mirror commands
pub fn execute(cmd: MirrorCommands) -> Result<()> {
    match cmd {
        MirrorCommands::Add {
            github_url,
            name,
            sync_interval,
        } => add_mirror(&github_url, name, sync_interval),
        MirrorCommands::List { output } => list_mirrors(&output),
        MirrorCommands::Sync { repo, force } => sync_mirror(&repo, force),
        MirrorCommands::Remove { repo, force } => remove_mirror(&repo, force),
    }
}

/// Generate mock mirrors
fn generate_mock_mirrors() -> Vec<Mirror> {
    let now = Utc::now();

    vec![
        Mirror {
            repository: "github.com/raibid/core".to_string(),
            name: "raibid-core".to_string(),
            sync_interval_minutes: 60,
            last_sync: now - Duration::minutes(15),
            status: MirrorStatus::Synced,
        },
        Mirror {
            repository: "github.com/raibid/cli".to_string(),
            name: "raibid-cli".to_string(),
            sync_interval_minutes: 60,
            last_sync: now - Duration::minutes(5),
            status: MirrorStatus::Synced,
        },
        Mirror {
            repository: "github.com/raibid/api".to_string(),
            name: "raibid-api".to_string(),
            sync_interval_minutes: 30,
            last_sync: now - Duration::minutes(2),
            status: MirrorStatus::Syncing,
        },
        Mirror {
            repository: "github.com/raibid/docs".to_string(),
            name: "raibid-docs".to_string(),
            sync_interval_minutes: 120,
            last_sync: now - Duration::hours(1),
            status: MirrorStatus::Synced,
        },
        Mirror {
            repository: "github.com/raibid/old-repo".to_string(),
            name: "old-repo".to_string(),
            sync_interval_minutes: 60,
            last_sync: now - Duration::hours(3),
            status: MirrorStatus::Error,
        },
    ]
}

/// Add a mirror
fn add_mirror(github_url: &str, name: Option<String>, sync_interval: u32) -> Result<()> {
    let mirror_name = name.unwrap_or_else(|| {
        // Extract repo name from URL
        github_url
            .split('/')
            .last()
            .unwrap_or("mirror")
            .to_string()
    });

    println!("{} Added mirror: {}", "✓".green(), github_url);
    println!("  Name: {}", mirror_name);
    println!("  Sync interval: {} minutes", sync_interval);
    println!("\n{}", "Note: This is a mock implementation. No mirror was actually created.".bright_black());

    Ok(())
}

/// List mirrors
fn list_mirrors(output: &str) -> Result<()> {
    let mirrors = generate_mock_mirrors();

    if output == "json" {
        let json = serde_json::to_string_pretty(&mirrors)?;
        println!("{}", json);
    } else {
        print_mirrors_table(&mirrors);
    }

    Ok(())
}

/// Print mirrors as ASCII table
fn print_mirrors_table(mirrors: &[Mirror]) {
    let mut table = Table::new();
    table
        .load_preset(ASCII_FULL)
        .set_header(vec![
            "REPOSITORY",
            "NAME",
            "SYNC INTERVAL",
            "LAST SYNC",
            "STATUS",
        ]);

    for mirror in mirrors {
        let interval_str = format!("{}m", mirror.sync_interval_minutes);
        let last_sync_str = format_relative_time(mirror.last_sync);

        let status_cell = match mirror.status {
            MirrorStatus::Synced => Cell::new(mirror.status.to_string()).fg(Color::Green),
            MirrorStatus::Syncing => Cell::new(mirror.status.to_string()).fg(Color::Yellow),
            MirrorStatus::Error => Cell::new(mirror.status.to_string()).fg(Color::Red),
        };

        table.add_row(vec![
            Cell::new(&mirror.repository),
            Cell::new(&mirror.name),
            Cell::new(interval_str),
            Cell::new(last_sync_str),
            status_cell,
        ]);
    }

    println!("{}", table);
}

/// Sync a mirror
fn sync_mirror(repo: &str, force: bool) -> Result<()> {
    let mirrors = generate_mock_mirrors();
    let mirror = mirrors
        .iter()
        .find(|m| m.repository == repo)
        .ok_or_else(|| anyhow::anyhow!("Mirror not found: {}", repo))?;

    if force {
        println!("{} Force syncing repository: {}", "⟳".cyan(), repo);
    } else {
        println!("{} Syncing repository: {}", "⟳".cyan(), repo);
    }

    println!("  Name: {}", mirror.name);
    println!("  Last sync: {}", format_relative_time(mirror.last_sync));
    println!("\n{}", "Note: This is a mock implementation. No sync was actually performed.".bright_black());

    Ok(())
}

/// Remove a mirror
fn remove_mirror(repo: &str, force: bool) -> Result<()> {
    let mirrors = generate_mock_mirrors();
    let mirror = mirrors
        .iter()
        .find(|m| m.repository == repo)
        .ok_or_else(|| anyhow::anyhow!("Mirror not found: {}", repo))?;

    if !force {
        let confirmed = Confirm::new()
            .with_prompt(format!(
                "Remove mirror {} (name: {}, synced {})?",
                repo,
                mirror.name,
                format_relative_time(mirror.last_sync)
            ))
            .default(false)
            .interact()?;

        if !confirmed {
            println!("Cancelled operation.");
            return Ok(());
        }
    }

    println!("{} Removed mirror: {}", "✓".green(), repo);
    println!("  Name: {}", mirror.name);
    println!("\n{}", "Note: This is a mock implementation. No mirror was actually removed.".bright_black());

    Ok(())
}

/// Format timestamp to relative time
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
