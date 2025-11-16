//! Status command implementation
//!
//! Real implementation of the status command for infrastructure components.
//! Queries Kubernetes API to show actual status information with colorful table output.

use anyhow::Result;
use colored::Colorize;
use comfy_table::{Table, Row, Cell, Color, Attribute, ContentArrangement, presets::UTF8_FULL};

use super::setup::Component;
use crate::infrastructure::{
    ComponentStatusChecker, K3sStatusChecker, GiteaStatusChecker,
    RedisStatusChecker, KedaStatusChecker, FluxStatusChecker,
    ComponentStatus, ComponentHealth,
};

/// Execute the status command for a component
pub fn execute(component: Option<Component>) -> Result<()> {
    let component = component.unwrap_or(Component::All);

    // Create tokio runtime for async operations
    let runtime = tokio::runtime::Runtime::new()?;

    if component == Component::All {
        runtime.block_on(show_all_status())
    } else {
        runtime.block_on(show_component_status(component))
    }
}

/// Show status for all components
async fn show_all_status() -> Result<()> {
    println!("{}", "Infrastructure Status".bold().cyan());
    println!();

    let mut statuses = Vec::new();

    // Collect status for each component
    for component in Component::all_components() {
        match get_component_status(component).await {
            Ok(status) => statuses.push(status),
            Err(e) => {
                // If we can't get status for a component, create an error status
                eprintln!(
                    "{} Failed to get status for {}: {}",
                    "⚠".yellow(),
                    component.name(),
                    e
                );

                statuses.push(ComponentStatus {
                    name: component.name().to_string(),
                    health: ComponentHealth::Unknown,
                    version: None,
                    pods: vec![],
                    resources: crate::infrastructure::ResourceUsage::default(),
                    endpoints: vec![],
                    uptime: None,
                    additional_info: std::collections::HashMap::new(),
                });
            }
        }
    }

    print_status_table(&statuses);

    Ok(())
}

/// Show status for a single component
async fn show_component_status(component: Component) -> Result<()> {
    println!(
        "{} {}",
        format!("{} Status", component.name()).bold().cyan(),
        "📊".bold()
    );
    println!();

    match get_component_status(component).await {
        Ok(status) => {
            print_detailed_status(&status)?;
        }
        Err(e) => {
            println!("{} {}", "Error:".red().bold(), e);
            println!();
            println!("{}", "Possible issues:".yellow().bold());
            println!("  {} k3s cluster may not be running", "•".blue());
            println!("  {} Component may not be installed", "•".blue());
            println!("  {} Kubeconfig may not be configured", "•".blue());
            println!();
            println!("{}", "Try:".green().bold());
            println!("  {} raibid-cli setup k3s", "→".blue());
            println!("  {} raibid-cli setup {}", "→".blue(), component.name());
        }
    }

    Ok(())
}

/// Get status for a component
async fn get_component_status(component: Component) -> Result<ComponentStatus> {
    match component {
        Component::K3s => {
            let checker = K3sStatusChecker::new().await?;
            checker.get_status().await
        }
        Component::Gitea => {
            let checker = GiteaStatusChecker::new().await?;
            checker.get_status().await
        }
        Component::Redis => {
            let checker = RedisStatusChecker::new().await?;
            checker.get_status().await
        }
        Component::Keda => {
            let checker = KedaStatusChecker::new().await?;
            checker.get_status().await
        }
        Component::Flux => {
            let checker = FluxStatusChecker::new().await?;
            checker.get_status().await
        }
        Component::All => Err(anyhow::anyhow!("Cannot get status for 'all' component")),
    }
}

/// Print status table for multiple components
fn print_status_table(statuses: &[ComponentStatus]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // Add header
    let mut header = Row::new();
    header.add_cell(Cell::new("COMPONENT").add_attribute(Attribute::Bold));
    header.add_cell(Cell::new("VERSION").add_attribute(Attribute::Bold));
    header.add_cell(Cell::new("HEALTH").add_attribute(Attribute::Bold));
    header.add_cell(Cell::new("PODS").add_attribute(Attribute::Bold));
    header.add_cell(Cell::new("CPU").add_attribute(Attribute::Bold));
    header.add_cell(Cell::new("MEMORY").add_attribute(Attribute::Bold));
    header.add_cell(Cell::new("UPTIME").add_attribute(Attribute::Bold));
    table.add_row(header);

    // Add rows for each component
    for status in statuses {
        let mut row = Row::new();

        // Component name
        row.add_cell(
            Cell::new(&status.name)
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan)
        );

        // Version
        let version = status.version.as_ref()
            .map(|v| v.version.clone())
            .unwrap_or_else(|| "N/A".to_string());
        row.add_cell(Cell::new(&version));

        // Health status with color
        let health_cell = match status.health {
            ComponentHealth::Healthy => Cell::new("healthy").fg(Color::Green),
            ComponentHealth::Degraded => Cell::new("degraded").fg(Color::Yellow),
            ComponentHealth::Unhealthy => Cell::new("unhealthy").fg(Color::Red),
            ComponentHealth::Unknown => Cell::new("unknown").fg(Color::Grey),
        };
        row.add_cell(health_cell);

        // Pod counts
        let total_pods = status.pods.len();
        let ready_pods = status.pods.iter().filter(|p| p.ready).count();
        let pod_info = format!("{}/{}", ready_pods, total_pods);
        let pod_cell = if ready_pods == total_pods && total_pods > 0 {
            Cell::new(&pod_info).fg(Color::Green)
        } else if ready_pods > 0 {
            Cell::new(&pod_info).fg(Color::Yellow)
        } else {
            Cell::new(&pod_info).fg(Color::Red)
        };
        row.add_cell(pod_cell);

        // CPU usage
        let cpu = status.resources.cpu_usage.as_deref().unwrap_or("N/A");
        row.add_cell(Cell::new(cpu));

        // Memory usage
        let memory = status.resources.memory_usage.as_deref().unwrap_or("N/A");
        row.add_cell(Cell::new(memory));

        // Uptime
        let uptime = status.uptime.as_deref().unwrap_or("N/A");
        row.add_cell(Cell::new(uptime).fg(Color::Grey));

        table.add_row(row);
    }

    println!("{}", table);
    println!();
}

/// Print detailed status for a single component
fn print_detailed_status(status: &ComponentStatus) -> Result<()> {
    // Component Information
    println!("{}", "Component Information:".bold());
    println!("  {} {}", "Name:".dimmed(), status.name.bold());

    if let Some(version) = &status.version {
        println!("  {} {}", "Version:".dimmed(), version.version.bold());
        if let Some(commit) = &version.git_commit {
            println!("  {} {}", "Git Commit:".dimmed(), commit);
        }
        if let Some(build_date) = &version.build_date {
            println!("  {} {}", "Build Date:".dimmed(), build_date);
        }
    }

    println!("  {} {}", "Health:".dimmed(), status.health.colorized());

    if let Some(uptime) = &status.uptime {
        println!("  {} {}", "Uptime:".dimmed(), uptime);
    }

    println!();

    // Pod Information
    if !status.pods.is_empty() {
        println!("{}", "Pods:".bold());

        let mut pod_table = Table::new();
        pod_table.load_preset(UTF8_FULL);
        pod_table.set_content_arrangement(ContentArrangement::Dynamic);

        // Header
        let mut header = Row::new();
        header.add_cell(Cell::new("NAME").add_attribute(Attribute::Bold));
        header.add_cell(Cell::new("NAMESPACE").add_attribute(Attribute::Bold));
        header.add_cell(Cell::new("STATUS").add_attribute(Attribute::Bold));
        header.add_cell(Cell::new("READY").add_attribute(Attribute::Bold));
        header.add_cell(Cell::new("RESTARTS").add_attribute(Attribute::Bold));
        header.add_cell(Cell::new("AGE").add_attribute(Attribute::Bold));
        pod_table.add_row(header);

        // Rows
        for pod in &status.pods {
            let mut row = Row::new();
            row.add_cell(Cell::new(&pod.name));
            row.add_cell(Cell::new(&pod.namespace));

            let phase_cell = match pod.phase.as_str() {
                "Running" => Cell::new(&pod.phase).fg(Color::Green),
                "Pending" => Cell::new(&pod.phase).fg(Color::Yellow),
                "Failed" | "CrashLoopBackOff" => Cell::new(&pod.phase).fg(Color::Red),
                _ => Cell::new(&pod.phase),
            };
            row.add_cell(phase_cell);

            let ready_cell = if pod.ready {
                Cell::new("Yes").fg(Color::Green)
            } else {
                Cell::new("No").fg(Color::Red)
            };
            row.add_cell(ready_cell);

            let restart_cell = if pod.restarts == 0 {
                Cell::new(&pod.restarts.to_string())
            } else if pod.restarts < 5 {
                Cell::new(&pod.restarts.to_string()).fg(Color::Yellow)
            } else {
                Cell::new(&pod.restarts.to_string()).fg(Color::Red)
            };
            row.add_cell(restart_cell);

            row.add_cell(Cell::new(&pod.age).fg(Color::Grey));

            pod_table.add_row(row);
        }

        println!("{}", pod_table);
        println!();
    }

    // Resource Usage
    if status.resources.cpu_usage.is_some() || status.resources.memory_usage.is_some() {
        println!("{}", "Resource Usage:".bold());
        if let Some(cpu) = &status.resources.cpu_usage {
            println!("  {} {}", "CPU:".dimmed(), cpu);
        }
        if let Some(memory) = &status.resources.memory_usage {
            println!("  {} {}", "Memory:".dimmed(), memory);
        }
        println!();
    }

    // Endpoints
    if !status.endpoints.is_empty() {
        println!("{}", "Endpoints:".bold());
        for endpoint in &status.endpoints {
            println!(
                "  {} {} (port: {}, protocol: {})",
                "→".blue(),
                endpoint.url.bold().green(),
                endpoint.port,
                endpoint.protocol
            );
        }
        println!();
    }

    // Additional Information
    if !status.additional_info.is_empty() {
        println!("{}", "Additional Information:".bold());
        for (key, value) in &status.additional_info {
            println!("  {} {}", format!("{}:", key).dimmed(), value);
        }
        println!();
    }

    Ok(())
}
