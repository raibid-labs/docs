//! Agent management commands implementation
//!
//! Mock implementations for agent management operations

use anyhow::Result;
use comfy_table::{presets::ASCII_FULL, Cell, Color, Table};
use serde::{Deserialize, Serialize};
use colored::Colorize;
use dialoguer::Confirm;

use crate::cli::AgentCommands;

/// Mock agent data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub status: AgentStatus,
    pub current_job: Option<String>,
    pub cpu_usage: u8,
    pub memory_usage: u8,
    pub uptime_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Running,
    Idle,
    Offline,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Running => write!(f, "Running"),
            AgentStatus::Idle => write!(f, "Idle"),
            AgentStatus::Offline => write!(f, "Offline"),
        }
    }
}

/// Execute agent commands
pub fn execute(cmd: AgentCommands) -> Result<()> {
    match cmd {
        AgentCommands::List { status, output } => list_agents(status, &output),
        AgentCommands::Show { agent_id, output } => show_agent(&agent_id, &output),
        AgentCommands::Restart { agent_id, force } => restart_agent(&agent_id, force),
        AgentCommands::Scale { count, min, max } => scale_agents(count, min, max),
    }
}

/// Generate mock agents
fn generate_mock_agents() -> Vec<Agent> {
    vec![
        Agent {
            id: "rust-builder-1".to_string(),
            status: AgentStatus::Running,
            current_job: Some("a1b2c3".to_string()),
            cpu_usage: 78,
            memory_usage: 65,
            uptime_hours: 12,
        },
        Agent {
            id: "rust-builder-2".to_string(),
            status: AgentStatus::Idle,
            current_job: None,
            cpu_usage: 5,
            memory_usage: 20,
            uptime_hours: 24,
        },
        Agent {
            id: "rust-builder-3".to_string(),
            status: AgentStatus::Running,
            current_job: Some("m4n5o6".to_string()),
            cpu_usage: 82,
            memory_usage: 70,
            uptime_hours: 6,
        },
        Agent {
            id: "rust-builder-4".to_string(),
            status: AgentStatus::Offline,
            current_job: None,
            cpu_usage: 0,
            memory_usage: 0,
            uptime_hours: 0,
        },
    ]
}

/// List agents
fn list_agents(status_filter: Option<String>, output: &str) -> Result<()> {
    let mut agents = generate_mock_agents();

    // Apply status filter
    if let Some(status) = status_filter {
        let status_lower = status.to_lowercase();
        agents.retain(|agent| agent.status.to_string().to_lowercase() == status_lower);
    }

    if output == "json" {
        let json = serde_json::to_string_pretty(&agents)?;
        println!("{}", json);
    } else {
        print_agents_table(&agents);
    }

    Ok(())
}

/// Print agents as ASCII table
fn print_agents_table(agents: &[Agent]) {
    let mut table = Table::new();
    table
        .load_preset(ASCII_FULL)
        .set_header(vec!["ID", "STATUS", "CURRENT JOB", "CPU %", "MEMORY %", "UPTIME"]);

    for agent in agents {
        let status_cell = match agent.status {
            AgentStatus::Running => Cell::new(agent.status.to_string()).fg(Color::Green),
            AgentStatus::Idle => Cell::new(agent.status.to_string()).fg(Color::Yellow),
            AgentStatus::Offline => Cell::new(agent.status.to_string()).fg(Color::Red),
        };

        let job_str = agent
            .current_job
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("-");

        table.add_row(vec![
            Cell::new(&agent.id),
            status_cell,
            Cell::new(job_str),
            Cell::new(&agent.cpu_usage.to_string()),
            Cell::new(&agent.memory_usage.to_string()),
            Cell::new(format!("{}h", agent.uptime_hours)),
        ]);
    }

    println!("{}", table);
}

/// Show agent details
fn show_agent(agent_id: &str, output: &str) -> Result<()> {
    let agents = generate_mock_agents();
    let agent = agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

    if output == "json" {
        let json = serde_json::to_string_pretty(&agent)?;
        println!("{}", json);
    } else {
        print_agent_details(agent);
    }

    Ok(())
}

/// Print agent details in human-readable format
fn print_agent_details(agent: &Agent) {
    println!("\n{}", "=== Agent Details ===".bold());
    println!("{:15} {}", "ID:", agent.id);

    let status_str = match agent.status {
        AgentStatus::Running => agent.status.to_string().green().to_string(),
        AgentStatus::Idle => agent.status.to_string().yellow().to_string(),
        AgentStatus::Offline => agent.status.to_string().red().to_string(),
    };
    println!("{:15} {}", "Status:", status_str);

    if let Some(job) = &agent.current_job {
        println!("{:15} {}", "Current Job:", job);
    } else {
        println!("{:15} {}", "Current Job:", "None");
    }

    println!("{:15} {}%", "CPU Usage:", agent.cpu_usage);
    println!("{:15} {}%", "Memory Usage:", agent.memory_usage);
    println!("{:15} {} hours", "Uptime:", agent.uptime_hours);
    println!();
}

/// Restart an agent
fn restart_agent(agent_id: &str, force: bool) -> Result<()> {
    let agents = generate_mock_agents();
    let agent = agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

    if !force {
        let job_info = agent
            .current_job
            .as_ref()
            .map(|j| format!(", currently running job {}", j))
            .unwrap_or_default();

        let confirmed = Confirm::new()
            .with_prompt(format!(
                "Restart agent {} ({}{})?",
                agent_id, agent.status, job_info
            ))
            .default(false)
            .interact()?;

        if !confirmed {
            println!("Cancelled operation.");
            return Ok(());
        }
    }

    println!("{} Restarted agent: {}", "✓".green(), agent_id);
    println!("  Previous status: {}", agent.status);

    if let Some(job) = &agent.current_job {
        println!("  {} Job {} will be rescheduled", "⚠".yellow(), job);
    }

    Ok(())
}

/// Scale agents
fn scale_agents(count: usize, min: Option<usize>, max: Option<usize>) -> Result<()> {
    let min_agents = min.unwrap_or(0);
    let max_agents = max.unwrap_or(10);

    // Validate count is within min/max
    if count < min_agents {
        anyhow::bail!("Count {} is less than minimum {}", count, min_agents);
    }
    if count > max_agents {
        anyhow::bail!("Count {} is greater than maximum {}", count, max_agents);
    }

    println!("{} Scaled agents to: {}", "✓".green(), count);
    println!("  Minimum agents: {}", min_agents);
    println!("  Maximum agents: {}", max_agents);
    println!("\n{}", "Note: This is a mock implementation. No agents were actually scaled.".bright_black());

    Ok(())
}
