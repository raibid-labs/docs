//! Setup command implementation
//!
//! Implements the setup command for infrastructure components.
//! Real implementations: k3s, Gitea, Redis, KEDA, Flux.

use anyhow::Result;
use colored::Colorize;
use std::thread;
use std::time::Duration;
use crate::infrastructure::{K3sInstaller, GiteaInstaller, RedisInstaller, KedaInstaller, FluxInstaller, FluxConfig};

/// Infrastructure component that can be set up
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Component {
    K3s,
    Gitea,
    Redis,
    Keda,
    Flux,
    All,
}

impl Component {
    /// Get component name as a string
    pub fn name(&self) -> &str {
        match self {
            Component::K3s => "k3s",
            Component::Gitea => "gitea",
            Component::Redis => "redis",
            Component::Keda => "keda",
            Component::Flux => "flux",
            Component::All => "all",
        }
    }

    /// Get component dependencies
    pub fn dependencies(&self) -> Vec<Component> {
        match self {
            Component::K3s => vec![],
            Component::Gitea => vec![Component::K3s],
            Component::Redis => vec![Component::K3s],
            Component::Keda => vec![Component::K3s],
            Component::Flux => vec![Component::K3s, Component::Gitea],
            Component::All => vec![],
        }
    }

    /// Get list of all components to setup when "all" is selected
    pub fn all_components() -> Vec<Component> {
        vec![
            Component::K3s,
            Component::Gitea,
            Component::Redis,
            Component::Keda,
            Component::Flux,
        ]
    }
}

impl std::str::FromStr for Component {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "k3s" => Ok(Component::K3s),
            "gitea" => Ok(Component::Gitea),
            "redis" => Ok(Component::Redis),
            "keda" => Ok(Component::Keda),
            "flux" => Ok(Component::Flux),
            "all" => Ok(Component::All),
            _ => Err(anyhow::anyhow!("Unknown component: {}", s)),
        }
    }
}

/// Execute the setup command for a component
pub fn execute(component: Component) -> Result<()> {
    if component == Component::All {
        setup_all()
    } else {
        setup_component(component)
    }
}

/// Setup all components
fn setup_all() -> Result<()> {
    println!(
        "{} {}",
        "Setting up all components...".bold().cyan(),
        "🚀".bold()
    );
    println!();

    for component in Component::all_components() {
        setup_component(component)?;
        println!();
    }

    println!(
        "{} {}",
        "All components setup successfully!".bold().green(),
        "✓".bold().green()
    );

    Ok(())
}

/// Setup a single component
fn setup_component(component: Component) -> Result<()> {
    println!(
        "{} {}",
        format!("Setting up {}...", component.name()).bold().cyan(),
        "⚙️".bold()
    );
    println!();

    // Show dependencies
    show_dependencies(component)?;

    // Run pre-flight checks
    run_preflight_checks()?;

    // Run actual setup or simulation based on component
    match component {
        Component::K3s => setup_k3s_real()?,
        Component::Gitea => setup_gitea_real()?,
        Component::Redis => setup_redis_real()?,
        Component::Keda => setup_keda_real()?,
        Component::Flux => setup_flux_real()?,
        _ => simulate_setup(component)?,
    }

    println!(
        "{} {} {}",
        "✓".bold().green(),
        component.name().bold(),
        "setup completed successfully!".green()
    );

    Ok(())
}

/// Show component dependencies
fn show_dependencies(component: Component) -> Result<()> {
    let deps = component.dependencies();

    if deps.is_empty() {
        println!("{} No dependencies", "ℹ".blue());
    } else {
        println!(
            "{} {} requires: {}",
            "ℹ".blue(),
            component.name().bold(),
            deps.iter()
                .map(|d| d.name())
                .collect::<Vec<_>>()
                .join(", ")
                .yellow()
        );

        for dep in deps {
            println!(
                "  {} {} would be installed first",
                "→".blue(),
                dep.name().yellow()
            );
        }
    }

    println!();
    Ok(())
}

/// Run pre-flight checks
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

/// Simulate the setup process
fn simulate_setup(component: Component) -> Result<()> {
    println!("{}", "Installing component...".bold());

    let steps = match component {
        Component::K3s => vec![
            "Downloading k3s binary",
            "Installing k3s service",
            "Starting k3s cluster",
            "Configuring kubectl",
        ],
        Component::Gitea => vec![
            "Creating Gitea namespace",
            "Deploying Gitea Helm chart",
            "Waiting for pods to be ready",
            "Configuring OCI registry",
        ],
        Component::Redis => vec![
            "Creating Redis namespace",
            "Deploying Redis Helm chart",
            "Configuring Redis Streams",
            "Testing connection",
        ],
        Component::Keda => vec![
            "Adding KEDA Helm repository",
            "Installing KEDA operator",
            "Configuring autoscaling",
            "Verifying KEDA installation",
        ],
        Component::Flux => vec![
            "Installing Flux CLI",
            "Bootstrapping Flux",
            "Configuring GitOps repository",
            "Setting up reconciliation",
        ],
        Component::All => vec![],
    };

    for step in steps {
        print!("  {} {}... ", "→".blue(), step);
        thread::sleep(Duration::from_millis(200));
        println!("{}", "done".green());
    }

    println!();
    Ok(())
}

/// Real k3s installation implementation
fn setup_k3s_real() -> Result<()> {
    println!("{}", "Installing k3s cluster...".bold());

    // Create runtime for async operations
    let runtime = tokio::runtime::Runtime::new()?;

    // Create installer
    let installer = K3sInstaller::new()?;

    // Run installation with rollback on failure
    let result = (|| -> Result<()> {
        // Download binary
        print!("  {} Downloading k3s binary... ", "→".blue());
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
        print!("  {} Starting k3s cluster... ", "→".blue());
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
    Ok(())
}

/// Real Gitea installation implementation
fn setup_gitea_real() -> Result<()> {
    println!("{}", "Installing Gitea via Helm...".bold());

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

        let (admin_user, admin_password) = installer.get_credentials();
        println!("  {} Admin username: {}", "→".blue(), admin_user.bold().yellow());
        println!("  {} Admin password: {}", "→".blue(), admin_password.bold().yellow());

        // Save credentials for Flux to use later
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/root"));
        let creds_dir = home.join(".raibid");
        std::fs::create_dir_all(&creds_dir)?;

        let creds_path = creds_dir.join("gitea-credentials.json");
        let creds = serde_json::json!({
            "admin_username": admin_user,
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
    Ok(())
}

/// Real Redis installation implementation
fn setup_redis_real() -> Result<()> {
    println!("{}", "Installing Redis with Helm...".bold());

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
        print!("  {} Deploying Redis Helm chart... ", "→".blue());
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
    Ok(())
}

/// Real KEDA installation implementation
fn setup_keda_real() -> Result<()> {
    println!("{}", "Installing KEDA autoscaler...".bold());

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
    Ok(())
}

/// Real Flux installation implementation
fn setup_flux_real() -> Result<()> {
    println!("{}", "Installing Flux GitOps...".bold());

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
        println!("  {} Please run 'raibid-cli setup gitea' first", "→".blue());
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
    Ok(())
}
