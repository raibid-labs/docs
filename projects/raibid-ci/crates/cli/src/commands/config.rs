//! Configuration management commands
//!
//! Provides subcommands for managing raibid-cli configuration:
//! - init: Create an example configuration file
//! - show: Display current configuration
//! - validate: Validate a configuration file
//! - path: Show configuration file location

use crate::cli::ConfigCommand;
use anyhow::{Context, Result};
use colored::Colorize;
use raibid_common::config::{
    discover_config_files, load_config, load_config_file, validate_config,
};
use std::fs;
use std::path::PathBuf;

/// Handle config command and its subcommands
pub fn handle(cmd: &ConfigCommand) -> Result<()> {
    match &cmd.command {
        crate::cli::ConfigSubcommand::Init {
            output,
            minimal,
            force,
        } => init_config(output.as_ref(), *minimal, *force),
        crate::cli::ConfigSubcommand::Show { format, file } => show_config(format, file.as_ref()),
        crate::cli::ConfigSubcommand::Validate { file } => validate_config_file(file.as_ref()),
        crate::cli::ConfigSubcommand::Path => show_config_path(),
    }
}

/// Initialize a new configuration file
fn init_config(output: Option<&PathBuf>, minimal: bool, force: bool) -> Result<()> {
    // Determine output path
    let output_path = if let Some(path) = output {
        path.clone()
    } else {
        PathBuf::from("./raibid.yaml")
    };

    // Check if file exists and force flag not set
    if output_path.exists() && !force {
        anyhow::bail!(
            "Configuration file already exists at {}. Use --force to overwrite.",
            output_path.display()
        );
    }

    // Get example content
    let content = if minimal {
        get_minimal_config()
    } else {
        get_example_config()
    };

    // Write file
    fs::write(&output_path, content)
        .with_context(|| format!("Failed to write config file: {}", output_path.display()))?;

    println!(
        "{} Created {} configuration file at: {}",
        "✓".green().bold(),
        if minimal { "minimal" } else { "example" },
        output_path.display().to_string().cyan()
    );

    Ok(())
}

/// Show current configuration
fn show_config(format: &str, file: Option<&PathBuf>) -> Result<()> {
    let config = if let Some(path) = file {
        load_config_file(path)?
    } else {
        load_config()?
    };

    match format {
        "yaml" => {
            let yaml =
                serde_yaml::to_string(&config).context("Failed to serialize config to YAML")?;
            println!("{}", yaml);
        }
        "json" => {
            let json = serde_json::to_string_pretty(&config)
                .context("Failed to serialize config to JSON")?;
            println!("{}", json);
        }
        "toml" => {
            let toml_str =
                toml::to_string_pretty(&config).context("Failed to serialize config to TOML")?;
            println!("{}", toml_str);
        }
        _ => {
            anyhow::bail!("Unsupported format: {}. Use yaml, json, or toml", format);
        }
    }

    Ok(())
}

/// Validate a configuration file
fn validate_config_file(file: Option<&PathBuf>) -> Result<()> {
    let config = if let Some(path) = file {
        println!("Validating config file: {}", path.display());
        load_config_file(path)?
    } else {
        println!("Validating merged configuration from all sources...");
        load_config()?
    };

    validate_config(&config)?;

    println!("{} Configuration is valid!", "✓".green().bold());
    Ok(())
}

/// Show configuration file path
fn show_config_path() -> Result<()> {
    let config_files = discover_config_files();

    if config_files.is_empty() {
        println!("{} No configuration files found.", "ℹ".blue().bold());
        println!("\nSearched locations:");
        println!("  - ./raibid.yaml");
        if let Some(config_dir) = dirs::config_dir() {
            println!(
                "  - {}",
                config_dir.join("raibid").join("config.yaml").display()
            );
        }
        println!("  - /etc/raibid/config.yaml");
        println!(
            "\nUse {} to create a new configuration file.",
            "raibid-cli config init".cyan()
        );
    } else {
        println!("{} Found configuration files:", "✓".green().bold());
        for (i, path) in config_files.iter().enumerate() {
            let priority = config_files.len() - i;
            println!(
                "  [Priority {}] {}",
                priority.to_string().yellow(),
                path.display().to_string().cyan()
            );
        }
    }

    Ok(())
}

/// Get minimal configuration example
fn get_minimal_config() -> String {
    r#"# Minimal raibid-cli configuration
# Only includes settings you're likely to change from defaults

cluster:
  name: raibid-ci
  namespace: raibid-ci

agents:
  max_agents: 10
  types:
    - rust
"#
    .to_string()
}

/// Get full example configuration with documentation
fn get_example_config() -> String {
    r#"# raibid-cli configuration file
# All settings shown with their default values

# Cluster (k3s) configuration
cluster:
  # Cluster name
  name: raibid-ci

  # Kubernetes API server port
  api_port: 6443

  # Path to kubeconfig file (~ expands to home directory)
  kubeconfig_path: ~/.kube/config

  # Namespace for raibid resources
  namespace: raibid-ci

  # CPU cores reserved for system (out of 20 total on DGX Spark)
  reserved_cores: 2

  # Memory reserved for system in GB (out of 128 GB total on DGX Spark)
  reserved_memory_gb: 8

# API server configuration
api:
  # API server host
  host: 127.0.0.1

  # API server port
  port: 8080

  # Enable TLS (requires tls_cert_path and tls_key_path)
  tls_enabled: false

  # Path to TLS certificate (only if tls_enabled: true)
  # tls_cert_path: /path/to/cert.pem

  # Path to TLS private key (only if tls_enabled: true)
  # tls_key_path: /path/to/key.pem

# Agent configuration
agents:
  # Agent types to enable (currently only 'rust' is supported in MVP)
  types:
    - rust

  # Minimum number of agents (0 = fully ephemeral)
  min_agents: 0

  # Maximum number of agents
  max_agents: 10

  # Agent idle timeout in seconds (agents terminate after this period of inactivity)
  idle_timeout_seconds: 300

  # Scale-down delay in seconds (wait period before removing idle agents)
  scaledown_delay_seconds: 60

  # Cache volume size in GB per agent
  cache_size_gb: 50

# Gitea configuration
gitea:
  # Gitea URL (cluster-internal by default)
  url: http://gitea.raibid-ci.svc.cluster.local:3000

  # Gitea admin username
  admin_user: admin

  # Gitea admin password (use environment variable: ${GITEA_ADMIN_PASSWORD})
  # admin_password: ${GITEA_ADMIN_PASSWORD}

  # Enable OCI registry
  registry_enabled: true

  # OCI registry port
  registry_port: 5000

# Redis configuration
redis:
  # Redis host (cluster-internal by default)
  host: redis.raibid-ci.svc.cluster.local

  # Redis port
  port: 6379

  # Redis password (use environment variable: ${REDIS_PASSWORD})
  # password: ${REDIS_PASSWORD}

  # Redis database number
  database: 0

  # Job queue stream name
  job_stream: raibid:jobs

# UI configuration
ui:
  # Enable TUI (terminal user interface)
  tui_enabled: true

  # TUI refresh rate in milliseconds
  refresh_rate_ms: 250

  # Color scheme: dark, light, or auto
  color_scheme: dark

  # Enable unicode characters in TUI
  unicode_enabled: true
"#
    .to_string()
}
