//! Health command implementation
//!
//! Implements comprehensive health checks for infrastructure components.
//! Returns proper exit codes for scripting and automation.

use anyhow::Result;
use colored::Colorize;
use comfy_table::{Table, Row, Cell, Color, Attribute, ContentArrangement, presets::UTF8_FULL};

use crate::commands::setup::Component;
use raibid_common::infrastructure::{
    ComponentStatusChecker, K3sStatusChecker, GiteaStatusChecker,
    RedisStatusChecker, KedaStatusChecker, FluxStatusChecker,
    ComponentStatus, ComponentHealth,
};

/// Execute the health command
pub fn execute(component: Option<String>, json: bool) -> Result<()> {
    let component = match component {
        Some(c) => c.parse()?,
        None => Component::All,
    };

    // Create tokio runtime for async operations
    let runtime = tokio::runtime::Runtime::new()?;

    if component == Component::All {
        runtime.block_on(check_all_health(json))
    } else {
        runtime.block_on(check_component_health(component, json))
    }
}

/// Check health for all components
async fn check_all_health(json: bool) -> Result<()> {
    if !json {
        println!("{}", "Infrastructure Health Check".bold().cyan());
        println!();
    }

    let mut statuses = Vec::new();
    let mut all_healthy = true;

    // Collect status for each component
    for component in Component::all_components() {
        match get_component_status(component).await {
            Ok(status) => {
                if status.health != ComponentHealth::Healthy {
                    all_healthy = false;
                }
                statuses.push(status);
            }
            Err(e) => {
                all_healthy = false;

                if !json {
                    eprintln!(
                        "{} Failed to get health for {}: {}",
                        "⚠".yellow(),
                        component.name(),
                        e
                    );
                }

                statuses.push(ComponentStatus {
                    name: component.name().to_string(),
                    health: ComponentHealth::Unknown,
                    version: None,
                    pods: vec![],
                    resources: raibid_common::infrastructure::ResourceUsage::default(),
                    endpoints: vec![],
                    uptime: None,
                    additional_info: std::collections::HashMap::new(),
                });
            }
        }
    }

    if json {
        print_json_health(&statuses)?;
    } else {
        print_health_table(&statuses);

        // Print summary
        println!();
        let healthy_count = statuses.iter().filter(|s| s.health == ComponentHealth::Healthy).count();
        let total = statuses.len();

        if all_healthy {
            println!("{} All {} components are healthy!", "✓".bold().green(), total);
        } else {
            println!(
                "{} {} of {} components are healthy",
                "⚠".bold().yellow(),
                healthy_count,
                total
            );
        }
    }

    // Exit with proper code for scripting
    if all_healthy {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

/// Check health for a single component
async fn check_component_health(component: Component, json: bool) -> Result<()> {
    if !json {
        println!(
            "{} {}",
            format!("{} Health Check", component.name()).bold().cyan(),
            "📊".bold()
        );
        println!();
    }

    match get_component_status(component).await {
        Ok(status) => {
            let is_healthy = status.health == ComponentHealth::Healthy;

            if json {
                print_json_health(&[status])?;
            } else {
                print_detailed_health(&status)?;
            }

            if is_healthy {
                Ok(())
            } else {
                std::process::exit(1);
            }
        }
        Err(e) => {
            if json {
                let error_json = serde_json::json!({
                    "component": component.name(),
                    "health": "error",
                    "error": e.to_string()
                });
                println!("{}", serde_json::to_string_pretty(&error_json)?);
            } else {
                println!("{} {}", "Error:".red().bold(), e);
                println!();
                println!("{}", "Possible issues:".yellow().bold());
                println!("  {} k3s cluster may not be running", "•".blue());
                println!("  {} Component may not be installed", "•".blue());
                println!("  {} Kubeconfig may not be configured", "•".blue());
                println!();
                println!("{}", "Try:".green().bold());
                println!("  {} raibid init k3s", "→".blue());
                println!("  {} raibid init {}", "→".blue(), component.name());
            }
            std::process::exit(2);
        }
    }
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
        Component::All => Err(anyhow::anyhow!("Cannot get health for 'all' component")),
    }
}

/// Print health table for multiple components
fn print_health_table(statuses: &[ComponentStatus]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // Add header
    let mut header = Row::new();
    header.add_cell(Cell::new("COMPONENT").add_attribute(Attribute::Bold));
    header.add_cell(Cell::new("HEALTH").add_attribute(Attribute::Bold));
    header.add_cell(Cell::new("PODS").add_attribute(Attribute::Bold));
    header.add_cell(Cell::new("VERSION").add_attribute(Attribute::Bold));
    header.add_cell(Cell::new("UPTIME").add_attribute(Attribute::Bold));
    header.add_cell(Cell::new("ISSUES").add_attribute(Attribute::Bold));
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

        // Health status with color and icon
        let health_cell = match status.health {
            ComponentHealth::Healthy => Cell::new("✓ Healthy").fg(Color::Green).add_attribute(Attribute::Bold),
            ComponentHealth::Degraded => Cell::new("⚠ Degraded").fg(Color::Yellow).add_attribute(Attribute::Bold),
            ComponentHealth::Unhealthy => Cell::new("✗ Unhealthy").fg(Color::Red).add_attribute(Attribute::Bold),
            ComponentHealth::Unknown => Cell::new("? Unknown").fg(Color::Grey),
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

        // Version
        let version = status.version.as_ref()
            .map(|v| v.version.clone())
            .unwrap_or_else(|| "N/A".to_string());
        row.add_cell(Cell::new(&version));

        // Uptime
        let uptime = status.uptime.as_deref().unwrap_or("N/A");
        row.add_cell(Cell::new(uptime).fg(Color::Grey));

        // Issues
        let issues = get_health_issues(status);
        let issues_cell = if issues.is_empty() {
            Cell::new("None").fg(Color::Green)
        } else {
            Cell::new(&issues.join(", ")).fg(Color::Yellow)
        };
        row.add_cell(issues_cell);

        table.add_row(row);
    }

    println!("{}", table);
}

/// Print detailed health for a single component
fn print_detailed_health(status: &ComponentStatus) -> Result<()> {
    // Overall Health
    println!("{}", "Overall Health:".bold());
    println!("  {} {}", "Status:".dimmed(), status.health.colorized_detailed());
    println!();

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

    if let Some(uptime) = &status.uptime {
        println!("  {} {}", "Uptime:".dimmed(), uptime);
    }

    println!();

    // Health Checks
    let issues = get_health_issues(status);
    if !issues.is_empty() {
        println!("{}", "Health Issues:".bold().yellow());
        for issue in issues {
            println!("  {} {}", "⚠".yellow(), issue);
        }
        println!();
    }

    // Pod Information
    if !status.pods.is_empty() {
        println!("{}", "Pods:".bold());

        let mut pod_table = Table::new();
        pod_table.load_preset(UTF8_FULL);
        pod_table.set_content_arrangement(ContentArrangement::Dynamic);

        // Header
        let mut header = Row::new();
        header.add_cell(Cell::new("NAME").add_attribute(Attribute::Bold));
        header.add_cell(Cell::new("STATUS").add_attribute(Attribute::Bold));
        header.add_cell(Cell::new("READY").add_attribute(Attribute::Bold));
        header.add_cell(Cell::new("RESTARTS").add_attribute(Attribute::Bold));
        header.add_cell(Cell::new("AGE").add_attribute(Attribute::Bold));
        pod_table.add_row(header);

        // Rows
        for pod in &status.pods {
            let mut row = Row::new();
            row.add_cell(Cell::new(&pod.name));

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

    Ok(())
}

/// Print health status as JSON
fn print_json_health(statuses: &[ComponentStatus]) -> Result<()> {
    let health_data: Vec<serde_json::Value> = statuses
        .iter()
        .map(|status| {
            let issues = get_health_issues(status);

            serde_json::json!({
                "component": status.name,
                "health": format!("{:?}", status.health).to_lowercase(),
                "healthy": status.health == ComponentHealth::Healthy,
                "version": status.version.as_ref().map(|v| &v.version),
                "uptime": status.uptime,
                "pods": {
                    "total": status.pods.len(),
                    "ready": status.pods.iter().filter(|p| p.ready).count(),
                },
                "resources": {
                    "cpu": status.resources.cpu_usage,
                    "memory": status.resources.memory_usage,
                },
                "issues": issues,
                "endpoints": status.endpoints.iter().map(|e| &e.url).collect::<Vec<_>>(),
            })
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&health_data)?);
    Ok(())
}

/// Get list of health issues for a component
fn get_health_issues(status: &ComponentStatus) -> Vec<String> {
    let mut issues = Vec::new();

    // Check pods
    let total_pods = status.pods.len();
    let ready_pods = status.pods.iter().filter(|p| p.ready).count();

    if total_pods > 0 && ready_pods < total_pods {
        issues.push(format!("{}/{} pods not ready", total_pods - ready_pods, total_pods));
    }

    // Check restarts
    for pod in &status.pods {
        if pod.restarts > 5 {
            issues.push(format!("Pod {} has {} restarts", pod.name, pod.restarts));
        }
    }

    // Check pod phase
    for pod in &status.pods {
        if pod.phase == "Failed" || pod.phase == "CrashLoopBackOff" {
            issues.push(format!("Pod {} is in {} state", pod.name, pod.phase));
        }
    }

    issues
}

// Trait extension for ComponentHealth
trait ComponentHealthExt {
    #[allow(dead_code)]
    fn colorized(&self) -> colored::ColoredString;
    fn colorized_detailed(&self) -> colored::ColoredString;
}

impl ComponentHealthExt for ComponentHealth {
    fn colorized(&self) -> colored::ColoredString {
        match self {
            ComponentHealth::Healthy => "Healthy".green().bold(),
            ComponentHealth::Degraded => "Degraded".yellow().bold(),
            ComponentHealth::Unhealthy => "Unhealthy".red().bold(),
            ComponentHealth::Unknown => "Unknown".dimmed(),
        }
    }

    fn colorized_detailed(&self) -> colored::ColoredString {
        match self {
            ComponentHealth::Healthy => "✓ Healthy - All checks passed".green().bold(),
            ComponentHealth::Degraded => "⚠ Degraded - Some issues detected".yellow().bold(),
            ComponentHealth::Unhealthy => "✗ Unhealthy - Critical issues detected".red().bold(),
            ComponentHealth::Unknown => "? Unknown - Unable to determine health".dimmed(),
        }
    }
}
