//! CLI argument parsing and command definitions
//!
//! This module handles all command-line argument parsing using clap.
//! It defines the CLI structure and routes commands to their implementations.

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// DGX Spark Personal CI Agent Pool
///
/// A TUI-first developer tool for managing self-hosted CI agents
#[derive(Parser, Debug)]
#[command(name = "raibid-cli")]
#[command(version, about, long_about = None)]
#[command(author = "Raibid Labs")]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available CLI subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage configuration
    Config(ConfigCommand),
    /// Launch the TUI dashboard for monitoring and management
    Tui,
    /// Setup infrastructure component
    Setup {
        /// Component to setup (k3s, gitea, redis, keda, flux, all)
        component: String,
    },
    /// Teardown infrastructure component
    Teardown {
        /// Component to teardown (k3s, gitea, redis, keda, flux, all)
        component: String,
    },
    /// Show status of infrastructure component
    Status {
        /// Component to show status for (k3s, gitea, redis, keda, flux, all)
        component: Option<String>,
    },
    // Placeholder for future subcommands
    // These will be added in future issues:
    // - Job
    // - Agent
    // - Mirror
}

/// Configuration management commands
#[derive(Args, Debug)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub command: ConfigSubcommand,
}

/// Configuration subcommands
#[derive(Subcommand, Debug)]
pub enum ConfigSubcommand {
    /// Initialize a new configuration file
    Init {
        /// Output path for the configuration file
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Generate minimal configuration instead of full example
        #[arg(short, long)]
        minimal: bool,

        /// Overwrite existing configuration file
        #[arg(short, long)]
        force: bool,
    },

    /// Show current configuration
    Show {
        /// Output format (yaml, json, toml)
        #[arg(short, long, default_value = "yaml")]
        format: String,

        /// Show configuration from specific file instead of merged config
        #[arg(long, value_name = "FILE")]
        file: Option<PathBuf>,
    },

    /// Validate configuration file
    Validate {
        /// Configuration file to validate (if not provided, validates merged config)
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },

    /// Show configuration file path
    Path,
}
