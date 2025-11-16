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
    /// Manage CI/CD jobs
    Jobs(JobsCommand),
    // Placeholder for future subcommands:
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

/// Jobs management commands
#[derive(Args, Debug)]
pub struct JobsCommand {
    #[command(subcommand)]
    pub command: JobsSubcommand,
}

/// Jobs subcommands
#[derive(Subcommand, Debug)]
pub enum JobsSubcommand {
    /// List jobs with optional filters
    List {
        /// Filter by status (pending, running, success, failed, cancelled)
        #[arg(short, long)]
        status: Option<String>,

        /// Filter by repository name
        #[arg(short, long)]
        repo: Option<String>,

        /// Filter by branch name
        #[arg(short, long)]
        branch: Option<String>,

        /// Maximum number of jobs to return
        #[arg(short, long, default_value = "25")]
        limit: Option<usize>,

        /// Offset for pagination
        #[arg(short, long, default_value = "0")]
        offset: Option<usize>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show detailed information about a specific job
    Show {
        /// Job ID to show
        job_id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show logs for a specific job
    Logs {
        /// Job ID to show logs for
        job_id: String,

        /// Follow log output (stream new logs in real-time)
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show from the end
        #[arg(short, long)]
        tail: Option<usize>,
    },

    /// Trigger a new job
    Trigger {
        /// Repository to build
        #[arg(short, long)]
        repo: String,

        /// Branch to build
        #[arg(short, long)]
        branch: String,

        /// Commit SHA to build (optional, defaults to latest)
        #[arg(short, long)]
        commit: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Cancel a running or pending job
    Cancel {
        /// Job ID to cancel
        job_id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}
