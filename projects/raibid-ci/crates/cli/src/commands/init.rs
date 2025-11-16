//! Init command implementation
//!
//! Implements the init command for infrastructure components with dry-run support.
//! This is the new name for the setup command with enhanced features.

use anyhow::Result;
use colored::Colorize;
use std::thread;
use std::time::Duration;
use raibid_common::infrastructure::{
    K3sInstaller, GiteaInstaller, RedisInstaller, KedaInstaller, FluxInstaller, FluxConfig,
};

use crate::cli::InitSubcommand;

/// Execute the init command
pub fn execute(cmd: &InitSubcommand) -> Result<()> {
    match cmd {
        InitSubcommand::K3s {
            dry_run,
            skip_checks,
            version,
            rootless,
        } => init_k3s(*dry_run, *skip_checks, version.as_deref(), *rootless),
        InitSubcommand::Gitea {
            dry_run,
            skip_checks,
            service_type,
            admin_user,
        } => init_gitea(*dry_run, *skip_checks, service_type, admin_user),
        InitSubcommand::Redis {
            dry_run,
            skip_checks,
            persistence,
        } => init_redis(*dry_run, *skip_checks, *persistence),
        InitSubcommand::Flux {
            dry_run,
            skip_checks,
            repo_path,
        } => init_flux(*dry_run, *skip_checks, repo_path.as_deref()),
        InitSubcommand::Keda {
            dry_run,
            skip_checks,
        } => init_keda(*dry_run, *skip_checks),
        InitSubcommand::All {
            dry_run,
            skip_checks,
        } => init_all(*dry_run, *skip_checks),
    }
}

/// Initialize all components
fn init_all(dry_run: bool, skip_checks: bool) -> Result<()> {
    print_header("all components");

    if dry_run {
        println!("{}", "DRY-RUN MODE: No changes will be made".yellow().bold());
        println!();
    }

    println!("{}", "The following components will be initialized:".bold());
    println!("  {} k3s (Kubernetes cluster)", "1.".blue());
    println!("  {} Gitea (Git server with OCI registry)", "2.".blue());
    println!("  {} Redis (Job queue with Streams)", "3.".blue());
    println!("  {} KEDA (Event-driven autoscaler)", "4.".blue());
    println!("  {} Flux (GitOps continuous delivery)", "5.".blue());
    println!();

    if dry_run {
        println!("{}", "Components would be installed in this order to respect dependencies.".dimmed());
        return Ok(());
    }

    // Install in dependency order
    init_k3s(false, skip_checks, None, false)?;
    println!();

    init_gitea(false, skip_checks, "NodePort", "raibid-admin")?;
    println!();

    init_redis(false, skip_checks, true)?;
    println!();

    init_keda(false, skip_checks)?;
    println!();

    init_flux(false, skip_checks, None)?;
    println!();

    println!(
        "{} {}",
        "All components initialized successfully!".bold().green(),
        "✓".bold().green()
    );

    Ok(())
}

/// Initialize k3s
fn init_k3s(dry_run: bool, skip_checks: bool, version: Option<&str>, rootless: bool) -> Result<()> {
    print_header("k3s");

    if dry_run {
        print_dry_run_plan("k3s", &[
            "Download k3s binary",
            "Verify checksum",
            "Install k3s service",
            "Start k3s cluster",
            "Configure kubectl",
        ]);
        return Ok(());
    }

    println!("{}", "Installing k3s cluster...".bold());

    if !skip_checks {
        run_preflight_checks()?;
    }

    // Create runtime for async operations
    let runtime = tokio::runtime::Runtime::new()?;

    // Create installer
    let installer = K3sInstaller::new()?;

    // Run installation with rollback on failure
    let result = (|| -> Result<()> {
        // Download binary
        print!("  {} Downloading k3s binary", "→".blue());
        if let Some(ver) = version {
            print!(" ({})", ver.dimmed());
        }
        print!("... ");
        let binary_path = runtime.block_on(installer.download_binary())?;
        println!("{}", "done".green());

        // Download and verify checksums
        print!("  {} Verifying checksum... ", "→".blue());
        let checksums = runtime.block_on(installer.download_checksums())?;
        installer.verify_checksum(&binary_path, &checksums)?;
        println!("{}", "done".green());

        // Install binary
        print!("  {} Installing k3s binary... ", "→".blue());
        installer.install_binary(&binary_path)?;
        println!("{}", "done".green());

        // Bootstrap cluster
        print!("  {} Starting k3s cluster", "→".blue());
        if rootless {
            print!(" (rootless mode)");
        }
        print!("... ");
        installer.bootstrap_cluster()?;
        println!("{}", "done".green());

        // Configure kubeconfig
        print!("  {} Configuring kubectl... ", "→".blue());
        installer.configure_kubeconfig()?;
        println!("{}", "done".green());

        // Validate cluster
        print!("  {} Validating cluster... ", "→".blue());
        installer.validate_cluster()?;
        println!("{}", "done".green());

        Ok(())
    })();

    // Handle errors with rollback
    if let Err(e) = result {
        println!("{}", "failed".red());
        println!();
        println!("{} Installation failed: {}", "✗".bold().red(), e);
        println!("{} Rolling back changes...", "→".yellow());

        if let Err(rollback_err) = installer.rollback() {
            println!("{} Rollback failed: {}", "✗".bold().red(), rollback_err);
        } else {
            println!("{} Rollback completed", "✓".green());
        }

        return Err(e);
    }

    // Cleanup on success
    installer.cleanup()?;

    println!();
    println!("{} k3s initialized successfully!", "✓".bold().green());

    Ok(())
}

/// Initialize Gitea
fn init_gitea(
    dry_run: bool,
    skip_checks: bool,
    _service_type: &str,
    _admin_user: &str,
) -> Result<()> {
    print_header("Gitea");

    if dry_run {
        print_dry_run_plan("Gitea", &[
            "Check prerequisites",
            "Install Helm if needed",
            "Create Gitea namespace",
            "Add Gitea Helm repository",
            "Deploy Gitea Helm chart",
            "Wait for pods to be ready",
            "Configure OCI registry",
        ]);
        return Ok(());
    }

    println!("{}", "Installing Gitea via Helm...".bold());

    if !skip_checks {
        run_preflight_checks()?;
    }

    // Create installer
    let installer = GiteaInstaller::new()?;

    // Run installation with rollback on failure
    let result = (|| -> Result<()> {
        // Check prerequisites
        print!("  {} Checking prerequisites... ", "→".blue());
        installer.check_kubectl()?;
        println!("{}", "done".green());

        // Install Helm if needed
        print!("  {} Installing Helm if needed... ", "→".blue());
        if installer.check_helm().is_err() {
            installer.install_helm()?;
        }
        println!("{}", "done".green());

        // Create namespace
        print!("  {} Creating Gitea namespace... ", "→".blue());
        installer.create_namespace()?;
        println!("{}", "done".green());

        // Add Helm repository
        print!("  {} Adding Gitea Helm repository... ", "→".blue());
        installer.add_helm_repo()?;
        println!("{}", "done".green());

        // Deploy Helm chart
        print!("  {} Deploying Gitea Helm chart (this may take several minutes)... ", "→".blue());
        installer.deploy_helm_chart()?;
        println!("{}", "done".green());

        // Wait for pods to be ready
        print!("  {} Waiting for Gitea pods to be ready... ", "→".blue());
        installer.wait_for_ready()?;
        println!("{}", "done".green());

        // Validate installation
        print!("  {} Validating installation... ", "→".blue());
        installer.validate_installation()?;
        println!("{}", "done".green());

        // Get service info
        print!("  {} Getting service information... ", "→".blue());
        let service_info = installer.get_service_info()?;
        println!("{}", "done".green());

        // Print access information
        println!();
        println!("{}", "Gitea Access Information:".bold().cyan());
        println!("  {} URL: {}", "→".blue(), service_info.access_url().bold().green());

        let (admin_username, admin_password) = installer.get_credentials();
        println!("  {} Admin username: {}", "→".blue(), admin_username.bold().yellow());
        println!("  {} Admin password: {}", "→".blue(), admin_password.bold().yellow());

        // Save credentials for Flux to use later
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/root"));
        let creds_dir = home.join(".raibid");
        std::fs::create_dir_all(&creds_dir)?;

        let creds_path = creds_dir.join("gitea-credentials.json");
        let creds = serde_json::json!({
            "admin_username": admin_username,
            "admin_password": admin_password,
            "url": service_info.access_url(),
        });
        std::fs::write(&creds_path, serde_json::to_string_pretty(&creds)?)?;

        // Set file permissions to owner-only
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&creds_path, std::fs::Permissions::from_mode(0o600))?;
        }

        println!();
        println!("{}", "⚠ Credentials saved securely for Flux integration".yellow().bold());
        println!("  {} {}", "→".blue(), creds_path.display());

        Ok(())
    })();

    // Handle errors with rollback
    if let Err(e) = result {
        println!("{}", "failed".red());
        println!();
        println!("{} Installation failed: {}", "✗".bold().red(), e);
        println!("{} Rolling back changes...", "→".yellow());

        if let Err(rollback_err) = installer.rollback() {
            println!("{} Rollback failed: {}", "✗".bold().red(), rollback_err);
        } else {
            println!("{} Rollback completed", "✓".green());
        }

        return Err(e);
    }

    // Cleanup on success
    installer.cleanup()?;

    println!();
    println!("{} Gitea initialized successfully!", "✓".bold().green());

    Ok(())
}

/// Initialize Redis
fn init_redis(dry_run: bool, skip_checks: bool, persistence: bool) -> Result<()> {
    print_header("Redis");

    if dry_run {
        print_dry_run_plan("Redis", &[
            "Add Bitnami Helm repository",
            "Create Redis namespace",
            "Deploy Redis Helm chart",
            "Wait for Redis to be ready",
            "Initialize Redis Streams",
            "Validate installation",
        ]);
        return Ok(());
    }

    println!("{}", "Installing Redis with Helm...".bold());

    if !skip_checks {
        run_preflight_checks()?;
    }

    // Create installer
    let mut installer = RedisInstaller::new()?;

    // Run installation with rollback on failure
    let result = (|| -> Result<()> {
        // Add Helm repository
        print!("  {} Adding Bitnami Helm repository... ", "→".blue());
        installer.add_helm_repo()?;
        println!("{}", "done".green());

        // Create namespace
        print!("  {} Creating Redis namespace... ", "→".blue());
        installer.create_namespace()?;
        println!("{}", "done".green());

        // Deploy Redis
        print!("  {} Deploying Redis Helm chart", "→".blue());
        if persistence {
            print!(" (with persistence)");
        }
        print!("... ");
        installer.deploy_redis()?;
        println!("{}", "done".green());

        // Wait for Redis to be ready
        print!("  {} Waiting for Redis to be ready... ", "→".blue());
        installer.wait_for_ready()?;
        println!("{}", "done".green());

        // Initialize Redis Streams
        print!("  {} Initializing Redis Streams... ", "→".blue());
        installer.initialize_streams()?;
        println!("{}", "done".green());

        // Validate installation
        print!("  {} Validating Redis installation... ", "→".blue());
        installer.validate()?;
        println!("{}", "done".green());

        // Save connection credentials
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/root"));
        let creds_path = home.join(".raibid").join("redis-credentials.json");
        if let Some(parent) = creds_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        print!("  {} Saving connection credentials... ", "→".blue());
        installer.save_credentials(&creds_path)?;
        println!("{}", "done".green());

        // Display connection info
        let conn_info = installer.get_connection_info()?;
        println!();
        println!("{}", "Redis connection details:".bold().green());
        println!("  {} Host: {}", "→".blue(), conn_info.host.bold());
        println!("  {} Port: {}", "→".blue(), conn_info.port.to_string().bold());
        println!("  {} Namespace: {}", "→".blue(), conn_info.namespace.bold());
        println!("  {} Credentials saved to: {}", "→".blue(), creds_path.display().to_string().bold());

        Ok(())
    })();

    // Handle errors with rollback
    if let Err(e) = result {
        println!("{}", "failed".red());
        println!();
        println!("{} Installation failed: {}", "✗".bold().red(), e);
        println!("{} Rolling back changes...", "→".yellow());

        if let Err(rollback_err) = installer.uninstall() {
            println!("{} Rollback failed: {}", "✗".bold().red(), rollback_err);
        } else {
            println!("{} Rollback completed", "✓".green());
        }

        return Err(e);
    }

    println!();
    println!("{} Redis initialized successfully!", "✓".bold().green());

    Ok(())
}

/// Initialize KEDA
fn init_keda(dry_run: bool, skip_checks: bool) -> Result<()> {
    print_header("KEDA");

    if dry_run {
        print_dry_run_plan("KEDA", &[
            "Check Helm",
            "Add KEDA Helm repository",
            "Create KEDA namespace",
            "Deploy KEDA operators",
            "Wait for KEDA to be ready",
            "Validate installation",
            "Create ScaledObject for Redis Streams",
        ]);
        return Ok(());
    }

    println!("{}", "Installing KEDA autoscaler...".bold());

    if !skip_checks {
        run_preflight_checks()?;
    }

    // Create installer
    let installer = KedaInstaller::new()?;

    // Run installation with rollback on failure
    let result = (|| -> Result<()> {
        // Check Helm
        print!("  {} Checking Helm... ", "→".blue());
        installer.check_helm()?;
        println!("{}", "done".green());

        // Add Helm repository
        print!("  {} Adding KEDA Helm repository... ", "→".blue());
        installer.add_helm_repo()?;
        println!("{}", "done".green());

        // Create namespace
        print!("  {} Creating KEDA namespace... ", "→".blue());
        installer.create_namespace()?;
        println!("{}", "done".green());

        // Deploy KEDA
        print!("  {} Deploying KEDA operators... ", "→".blue());
        installer.deploy_keda()?;
        println!("{}", "done".green());

        // Wait for KEDA to be ready
        print!("  {} Waiting for KEDA to be ready... ", "→".blue());
        installer.wait_for_ready()?;
        println!("{}", "done".green());

        // Validate installation
        print!("  {} Validating KEDA installation... ", "→".blue());
        installer.validate()?;
        println!("{}", "done".green());

        // Create ScaledObject for Redis Streams
        print!("  {} Creating ScaledObject for Redis Streams... ", "→".blue());
        installer.create_scaled_object()?;
        println!("{}", "done".green());

        // Display KEDA status
        println!();
        println!("{}", "KEDA Status:".bold().cyan());
        match installer.get_scaled_object_status() {
            Ok(status) => {
                for line in status.lines() {
                    println!("  {}", line);
                }
            }
            Err(e) => {
                println!("  {} Failed to get status: {}", "⚠".yellow(), e);
            }
        }

        Ok(())
    })();

    // Handle errors with rollback
    if let Err(e) = result {
        println!("{}", "failed".red());
        println!();
        println!("{} Installation failed: {}", "✗".bold().red(), e);
        println!("{} Rolling back changes...", "→".yellow());

        if let Err(rollback_err) = installer.uninstall() {
            println!("{} Rollback failed: {}", "✗".bold().red(), rollback_err);
        } else {
            println!("{} Rollback completed", "✓".green());
        }

        return Err(e);
    }

    println!();
    println!("{} KEDA initialized successfully!", "✓".bold().green());

    Ok(())
}

/// Initialize Flux
fn init_flux(dry_run: bool, skip_checks: bool, _repo_path: Option<&str>) -> Result<()> {
    print_header("Flux");

    if dry_run {
        print_dry_run_plan("Flux", &[
            "Check for Flux CLI",
            "Download Flux CLI if needed",
            "Verify checksum",
            "Install Flux CLI",
            "Bootstrap Flux with Gitea",
            "Configure image automation",
            "Configure notifications",
            "Validate installation",
        ]);
        return Ok(());
    }

    println!("{}", "Installing Flux GitOps...".bold());

    if !skip_checks {
        run_preflight_checks()?;
    }

    // Create runtime for async operations
    let runtime = tokio::runtime::Runtime::new()?;

    // Get Gitea credentials from saved file
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/root"));
    let gitea_creds_path = home.join(".raibid").join("gitea-credentials.json");

    // Read Gitea credentials
    let (gitea_username, gitea_password) = if gitea_creds_path.exists() {
        let contents = std::fs::read_to_string(&gitea_creds_path)?;
        let creds: serde_json::Value = serde_json::from_str(&contents)?;

        let username = creds["admin_username"]
            .as_str()
            .unwrap_or("raibid-admin")
            .to_string();
        let password = creds["admin_password"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Gitea password not found in credentials file"))?
            .to_string();

        (username, password)
    } else {
        println!(
            "{} Gitea credentials not found at {}",
            "⚠".yellow(),
            gitea_creds_path.display()
        );
        println!("  {} Please run 'raibid init gitea' first", "→".blue());
        return Err(anyhow::anyhow!("Gitea must be installed before Flux"));
    };

    // Create Flux config
    let config = FluxConfig {
        password: gitea_password,
        username: gitea_username,
        ..Default::default()
    };

    // Create installer
    let installer = FluxInstaller::with_config(config)?;

    // Run installation with rollback on failure
    let result = (|| -> Result<()> {
        // Check if Flux CLI is installed
        print!("  {} Checking for Flux CLI... ", "→".blue());
        let flux_installed = installer.check_flux_cli()?;

        if flux_installed {
            println!("{}", "already installed".green());
        } else {
            println!("{}", "not found".yellow());

            // Download Flux CLI
            print!("  {} Downloading Flux CLI... ", "→".blue());
            let archive_path = runtime.block_on(installer.download_flux())?;
            println!("{}", "done".green());

            // Download and verify checksums
            print!("  {} Verifying checksum... ", "→".blue());
            let checksums = runtime.block_on(installer.download_checksums())?;
            installer.verify_checksum(&archive_path, &checksums)?;
            println!("{}", "done".green());

            // Install Flux CLI
            print!("  {} Installing Flux CLI... ", "→".blue());
            installer.install_flux_cli(&archive_path)?;
            println!("{}", "done".green());
        }

        // Bootstrap Flux with Gitea
        print!("  {} Bootstrapping Flux with Gitea... ", "→".blue());
        installer.bootstrap_flux()?;
        println!("{}", "done".green());

        // Configure image automation
        print!("  {} Configuring image automation... ", "→".blue());
        installer.configure_image_automation()?;
        println!("{}", "done".green());

        // Configure notifications
        print!("  {} Configuring notification controller... ", "→".blue());
        installer.configure_notifications()?;
        println!("{}", "done".green());

        // Validate installation
        print!("  {} Validating Flux installation... ", "→".blue());
        installer.validate_installation()?;
        println!("{}", "done".green());

        // Get and display status
        println!();
        println!("{}", "Flux Status:".bold().cyan());
        match installer.get_status() {
            Ok(status) => {
                for line in status.lines() {
                    println!("  {}", line);
                }
            }
            Err(e) => {
                println!("  {} Failed to get status: {}", "⚠".yellow(), e);
            }
        }

        Ok(())
    })();

    // Handle errors with rollback
    if let Err(e) = result {
        println!("{}", "failed".red());
        println!();
        println!("{} Installation failed: {}", "✗".bold().red(), e);
        println!("{} Rolling back changes...", "→".yellow());

        if let Err(rollback_err) = installer.rollback() {
            println!("{} Rollback failed: {}", "✗".bold().red(), rollback_err);
        } else {
            println!("{} Rollback completed", "✓".green());
        }

        return Err(e);
    }

    // Cleanup on success
    installer.cleanup()?;

    println!();
    println!("{} Flux initialized successfully!", "✓".bold().green());

    Ok(())
}

// Helper functions

fn print_header(component: &str) {
    println!(
        "{} {}",
        format!("Initializing {}...", component).bold().cyan(),
        "⚙️".bold()
    );
    println!();
}

fn print_dry_run_plan(component: &str, steps: &[&str]) {
    println!("{}", "DRY-RUN MODE: No changes will be made".yellow().bold());
    println!();
    println!("{}", format!("The following steps would be performed for {}:", component).bold());

    for (i, step) in steps.iter().enumerate() {
        println!("  {} {}", format!("{}.", i + 1).blue(), step);
    }
}

fn run_preflight_checks() -> Result<()> {
    println!("{}", "Running pre-flight checks...".bold());

    // Check disk space
    print!("  {} Checking disk space... ", "→".blue());
    thread::sleep(Duration::from_millis(100));
    println!("{} {} available", "✓".green(), "250 GB".bold());

    // Check memory
    print!("  {} Checking memory... ", "→".blue());
    thread::sleep(Duration::from_millis(100));
    println!("{} {} available", "✓".green(), "128 GB".bold());

    // Check CPU
    print!("  {} Checking CPU cores... ", "→".blue());
    thread::sleep(Duration::from_millis(100));
    println!("{} {} cores", "✓".green(), "20".bold());

    println!();
    Ok(())
}
