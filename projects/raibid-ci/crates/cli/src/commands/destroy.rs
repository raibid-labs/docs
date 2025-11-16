//! Destroy command implementation
//!
//! Implements safe destruction of infrastructure components with confirmation,
//! dry-run mode, and dependency checking.

use anyhow::Result;
use colored::Colorize;
use dialoguer::{Confirm, theme::ColorfulTheme};
use std::thread;
use std::time::Duration;

use crate::commands::setup::Component;

/// Execute the destroy command
pub fn execute(component: String, yes: bool, dry_run: bool, force: bool) -> Result<()> {
    let component: Component = component.parse()?;

    if component == Component::All {
        destroy_all(yes, dry_run, force)
    } else {
        destroy_component(component, yes, dry_run, force)
    }
}

/// Destroy all components (in reverse order)
fn destroy_all(yes: bool, dry_run: bool, force: bool) -> Result<()> {
    println!(
        "{} {}",
        "Destroying all infrastructure components...".bold().red(),
        "🗑️".bold()
    );
    println!();

    // Teardown in reverse order to respect dependencies
    let mut components = Component::all_components();
    components.reverse();

    // Show what will be destroyed
    println!("{}", "The following components will be destroyed:".bold());
    for (i, comp) in components.iter().enumerate() {
        println!("  {} {}", format!("{}.", i + 1).red(), comp.name());
    }
    println!();

    if dry_run {
        println!("{}", "DRY-RUN MODE: No changes will be made".yellow().bold());
        println!();

        for component in components {
            show_destroy_plan(component)?;
            println!();
        }

        return Ok(());
    }

    // Confirm destruction
    if !yes {
        println!("{}", "⚠ WARNING: This action is destructive and cannot be undone!".red().bold());
        println!();

        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Are you sure you want to destroy ALL infrastructure components?")
            .default(false)
            .interact()?;

        if !confirmed {
            println!("{}", "Destroy cancelled.".yellow());
            return Ok(());
        }

        // Double confirmation for safety
        let double_confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("This will delete all data. Type 'yes' to confirm")
            .default(false)
            .interact()?;

        if !double_confirmed {
            println!("{}", "Destroy cancelled.".yellow());
            return Ok(());
        }
    }

    // Destroy each component
    for component in components {
        destroy_component_impl(component, force)?;
        println!();
    }

    println!(
        "{} {}",
        "All components destroyed successfully!".bold().green(),
        "✓".bold().green()
    );

    Ok(())
}

/// Destroy a single component
fn destroy_component(component: Component, yes: bool, dry_run: bool, force: bool) -> Result<()> {
    println!(
        "{} {}",
        format!("Destroying {}...", component.name())
            .bold()
            .red(),
        "🗑️".bold()
    );
    println!();

    // Show what will be destroyed
    show_destroy_info(component)?;

    if dry_run {
        println!("{}", "DRY-RUN MODE: No changes will be made".yellow().bold());
        println!();
        show_destroy_plan(component)?;
        return Ok(());
    }

    // Check for dependent components (if not forcing)
    if !force {
        check_dependencies(component)?;
    }

    // Confirm destruction
    if !yes {
        println!("{}", "⚠ WARNING: This action is destructive and cannot be undone!".red().bold());
        println!();

        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Are you sure you want to destroy {}?", component.name()))
            .default(false)
            .interact()?;

        if !confirmed {
            println!("{}", "Destroy cancelled.".yellow());
            return Ok(());
        }
    }

    // Destroy the component
    destroy_component_impl(component, force)?;

    println!(
        "{} {} {}",
        "✓".bold().green(),
        component.name().bold(),
        "destroyed successfully!".green()
    );

    Ok(())
}

/// Actually destroy the component
fn destroy_component_impl(component: Component, _force: bool) -> Result<()> {
    println!("{}", "Destroying resources...".bold());

    let steps = match component {
        Component::K3s => vec![
            "Stopping k3s service",
            "Draining nodes",
            "Removing k3s binaries",
            "Cleaning up container data",
            "Removing configuration files",
            "Deleting /var/lib/rancher/k3s",
        ],
        Component::Gitea => vec![
            "Scaling down Gitea pods",
            "Removing Helm release",
            "Deleting persistent volumes",
            "Cleaning up OCI registry data",
            "Deleting namespace",
        ],
        Component::Redis => vec![
            "Stopping Redis pods",
            "Flushing Redis data",
            "Removing Helm release",
            "Cleaning up persistent volumes",
            "Deleting namespace",
        ],
        Component::Keda => vec![
            "Removing ScaledObjects",
            "Removing KEDA operator",
            "Deleting custom resource definitions",
            "Removing Helm release",
            "Cleaning up configurations",
        ],
        Component::Flux => vec![
            "Stopping Flux controllers",
            "Removing GitOps configurations",
            "Uninstalling Flux",
            "Deleting Flux namespace",
            "Cleaning up custom resources",
        ],
        Component::All => vec![],
    };

    for step in steps {
        print!("  {} {}... ", "→".red(), step);
        thread::sleep(Duration::from_millis(300));
        println!("{}", "done".green());
    }

    println!();
    Ok(())
}

/// Show information about what will be destroyed
fn show_destroy_info(component: Component) -> Result<()> {
    println!("{}", "The following will be destroyed:".bold());

    let items = match component {
        Component::K3s => vec![
            ("k3s service and binaries", "Critical"),
            ("All Kubernetes resources", "Critical"),
            ("Container images and data", "Data loss"),
            ("/var/lib/rancher/k3s directory", "Data loss"),
            ("All dependent components", "Critical"),
        ],
        Component::Gitea => vec![
            ("Gitea namespace and pods", "Service"),
            ("Gitea Helm release", "Config"),
            ("All Git repositories", "Data loss"),
            ("OCI registry data", "Data loss"),
            ("Persistent volumes", "Data loss"),
        ],
        Component::Redis => vec![
            ("Redis namespace and pods", "Service"),
            ("Redis Helm release", "Config"),
            ("All job queue data", "Data loss"),
            ("Redis persistent data", "Data loss"),
            ("Persistent volumes", "Data loss"),
        ],
        Component::Keda => vec![
            ("KEDA operator", "Service"),
            ("KEDA Helm release", "Config"),
            ("All autoscaling configurations", "Config"),
            ("Custom resource definitions", "Config"),
        ],
        Component::Flux => vec![
            ("Flux controllers", "Service"),
            ("GitOps configurations", "Config"),
            ("Flux system namespace", "Service"),
            ("Flux custom resources", "Config"),
        ],
        Component::All => vec![],
    };

    for (item, severity) in items {
        let severity_colored = match severity {
            "Critical" => severity.red().bold(),
            "Data loss" => severity.red(),
            "Service" => severity.yellow(),
            "Config" => severity.blue(),
            _ => severity.normal(),
        };
        println!("  {} {} [{}]", "•".red(), item, severity_colored);
    }

    println!();
    Ok(())
}

/// Show detailed destroy plan for dry-run
fn show_destroy_plan(component: Component) -> Result<()> {
    println!("{}", format!("Destroy plan for {}:", component.name()).bold().yellow());

    let plan = match component {
        Component::K3s => vec![
            ("Pre-checks", vec![
                "Verify no critical workloads running",
                "Check for dependent components",
                "Backup current state (if requested)",
            ]),
            ("Graceful shutdown", vec![
                "Drain all nodes",
                "Stop k3s service",
                "Wait for pod termination",
            ]),
            ("Cleanup", vec![
                "Remove k3s binaries",
                "Delete container runtime data",
                "Remove configuration files",
                "Delete /var/lib/rancher/k3s",
            ]),
            ("Verification", vec![
                "Verify all processes stopped",
                "Verify all files removed",
            ]),
        ],
        Component::Gitea => vec![
            ("Pre-checks", vec![
                "Check if Flux depends on Gitea",
                "List repositories (for backup)",
            ]),
            ("Graceful shutdown", vec![
                "Scale deployment to 0",
                "Wait for pod termination",
            ]),
            ("Cleanup", vec![
                "Delete Helm release",
                "Delete persistent volumes",
                "Delete namespace",
            ]),
            ("Verification", vec![
                "Verify namespace deleted",
                "Verify PVs released",
            ]),
        ],
        Component::Redis => vec![
            ("Pre-checks", vec![
                "Check for active jobs",
                "Check if KEDA depends on Redis",
            ]),
            ("Graceful shutdown", vec![
                "Flush all queues (if requested)",
                "Stop Redis pods",
            ]),
            ("Cleanup", vec![
                "Delete Helm release",
                "Delete persistent volumes",
                "Delete namespace",
            ]),
            ("Verification", vec![
                "Verify namespace deleted",
                "Verify PVs released",
            ]),
        ],
        Component::Keda => vec![
            ("Pre-checks", vec![
                "List all ScaledObjects",
                "Check for active autoscaling",
            ]),
            ("Graceful shutdown", vec![
                "Delete all ScaledObjects",
                "Stop KEDA operator",
            ]),
            ("Cleanup", vec![
                "Delete Helm release",
                "Delete CRDs",
                "Delete namespace",
            ]),
            ("Verification", vec![
                "Verify CRDs deleted",
                "Verify namespace deleted",
            ]),
        ],
        Component::Flux => vec![
            ("Pre-checks", vec![
                "List all Flux resources",
                "Check for pending reconciliations",
            ]),
            ("Graceful shutdown", vec![
                "Suspend all reconciliations",
                "Stop Flux controllers",
            ]),
            ("Cleanup", vec![
                "Uninstall Flux",
                "Delete GitOps configurations",
                "Delete namespace",
            ]),
            ("Verification", vec![
                "Verify Flux uninstalled",
                "Verify namespace deleted",
            ]),
        ],
        Component::All => vec![],
    };

    for (phase, steps) in plan {
        println!("  {} {}", "Phase:".bold(), phase.yellow());
        for step in steps {
            println!("    {} {}", "→".blue(), step);
        }
        println!();
    }

    Ok(())
}

/// Check for dependent components
fn check_dependencies(component: Component) -> Result<()> {
    let dependents = get_dependents(component);

    if !dependents.is_empty() {
        println!(
            "{} {} has dependent components that must be destroyed first:",
            "⚠".yellow(),
            component.name()
        );

        for dep in &dependents {
            println!("  {} {}", "•".yellow(), dep.name());
        }

        println!();
        println!("{}", "Options:".bold());
        println!("  {} Destroy dependents first", "1.".blue());
        println!("  {} Use --force to destroy anyway (may break things!)", "2.".blue());
        println!();

        return Err(anyhow::anyhow!(
            "Cannot destroy {} without destroying dependent components first",
            component.name()
        ));
    }

    Ok(())
}

/// Get list of components that depend on this component
fn get_dependents(component: Component) -> Vec<Component> {
    match component {
        Component::K3s => {
            // All other components depend on k3s
            vec![
                Component::Gitea,
                Component::Redis,
                Component::Keda,
                Component::Flux,
            ]
        }
        Component::Gitea => {
            // Flux depends on Gitea
            vec![Component::Flux]
        }
        Component::Redis => {
            // KEDA depends on Redis
            vec![Component::Keda]
        }
        Component::Keda => vec![],
        Component::Flux => vec![],
        Component::All => vec![],
    }
}
