//! Build pipeline execution module
//!
//! This module handles the complete Rust build pipeline including:
//! - Code quality checks (cargo check, clippy, fmt)
//! - Testing (cargo test)
//! - Building (cargo build)
//! - Security auditing (cargo audit)
//! - Docker image building and pushing
//! - Log streaming to Redis
//! - Artifact metadata management

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// Maximum timeout for individual build steps (5 minutes)
const STEP_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// Maximum timeout for entire pipeline (30 minutes)
const PIPELINE_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// Pipeline execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Job ID for tracking
    pub job_id: String,
    /// Repository path on disk
    pub repo_path: PathBuf,
    /// Enable sccache for build optimization
    pub use_sccache: bool,
    /// Docker registry URL for image pushing
    pub registry_url: Option<String>,
    /// Docker image tag
    pub image_tag: Option<String>,
    /// Redis connection for log streaming
    pub redis_url: Option<String>,
}

/// Individual build step
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildStep {
    /// Check code compilation without building
    Check,
    /// Run clippy lints
    Clippy,
    /// Check code formatting
    Format,
    /// Run tests
    Test,
    /// Build release binary
    Build,
    /// Run security audit
    Audit,
    /// Build Docker image
    DockerBuild,
    /// Push Docker image to registry
    DockerPush,
}

impl BuildStep {
    /// Get step name for logging
    pub fn name(&self) -> &str {
        match self {
            BuildStep::Check => "check",
            BuildStep::Clippy => "clippy",
            BuildStep::Format => "format",
            BuildStep::Test => "test",
            BuildStep::Build => "build",
            BuildStep::Audit => "audit",
            BuildStep::DockerBuild => "docker-build",
            BuildStep::DockerPush => "docker-push",
        }
    }

    /// Get step description
    pub fn description(&self) -> &str {
        match self {
            BuildStep::Check => "Checking code compilation",
            BuildStep::Clippy => "Running clippy lints",
            BuildStep::Format => "Checking code formatting",
            BuildStep::Test => "Running tests",
            BuildStep::Build => "Building release binary",
            BuildStep::Audit => "Running security audit",
            BuildStep::DockerBuild => "Building Docker image",
            BuildStep::DockerPush => "Pushing Docker image",
        }
    }
}

/// Result of a build step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step name
    pub step: String,
    /// Whether the step succeeded
    pub success: bool,
    /// Exit code
    pub exit_code: Option<i32>,
    /// Duration in seconds
    pub duration_secs: u64,
    /// Captured stdout/stderr (first 10KB)
    pub output: String,
}

/// Complete pipeline execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    /// Job ID
    pub job_id: String,
    /// Overall success
    pub success: bool,
    /// Individual step results
    pub steps: Vec<StepResult>,
    /// Total duration in seconds
    pub total_duration_secs: u64,
    /// Artifact metadata (if build succeeded)
    pub artifacts: Option<ArtifactMetadata>,
}

/// Metadata about build artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    /// Docker image name and tag
    pub image: Option<String>,
    /// Binary artifacts produced
    pub binaries: Vec<String>,
    /// Build timestamp
    pub built_at: String,
}

/// Pipeline executor
pub struct PipelineExecutor {
    config: PipelineConfig,
    redis_client: Option<redis::Client>,
}

impl PipelineExecutor {
    /// Create a new pipeline executor
    pub fn new(config: PipelineConfig) -> Result<Self> {
        let redis_client = if let Some(ref url) = config.redis_url {
            Some(redis::Client::open(url.as_str()).context("Failed to create Redis client")?)
        } else {
            None
        };

        Ok(Self {
            config,
            redis_client,
        })
    }

    /// Execute the complete build pipeline
    pub async fn execute(&self) -> Result<PipelineResult> {
        info!(
            job_id = %self.config.job_id,
            "Starting Rust build pipeline"
        );

        let start_time = std::time::Instant::now();
        let mut step_results = Vec::new();
        let mut overall_success = true;

        // Define the build pipeline steps
        let steps = vec![
            BuildStep::Check,
            BuildStep::Format,
            BuildStep::Clippy,
            BuildStep::Test,
            BuildStep::Build,
            BuildStep::Audit,
        ];

        // Execute each step with timeout
        for step in steps {
            match timeout(
                PIPELINE_TIMEOUT - start_time.elapsed(),
                self.execute_step(step),
            )
            .await
            {
                Ok(Ok(result)) => {
                    if !result.success {
                        overall_success = false;
                    }
                    step_results.push(result);

                    // Stop pipeline on failure
                    if !overall_success {
                        warn!(
                            job_id = %self.config.job_id,
                            step = step.name(),
                            "Pipeline stopped due to step failure"
                        );
                        break;
                    }
                }
                Ok(Err(e)) => {
                    error!(
                        job_id = %self.config.job_id,
                        step = step.name(),
                        error = %e,
                        "Step execution failed"
                    );
                    overall_success = false;
                    step_results.push(StepResult {
                        step: step.name().to_string(),
                        success: false,
                        exit_code: None,
                        duration_secs: 0,
                        output: format!("Error: {}", e),
                    });
                    break;
                }
                Err(_) => {
                    error!(
                        job_id = %self.config.job_id,
                        "Pipeline timeout exceeded"
                    );
                    overall_success = false;
                    step_results.push(StepResult {
                        step: "timeout".to_string(),
                        success: false,
                        exit_code: None,
                        duration_secs: start_time.elapsed().as_secs(),
                        output: "Pipeline timeout exceeded".to_string(),
                    });
                    break;
                }
            }
        }

        // Build and push Docker image if build succeeded
        let mut artifacts = None;
        if overall_success && self.config.registry_url.is_some() {
            if let Ok(result) = self.execute_step(BuildStep::DockerBuild).await {
                step_results.push(result.clone());
                if result.success {
                    if let Ok(push_result) = self.execute_step(BuildStep::DockerPush).await {
                        step_results.push(push_result.clone());
                        if push_result.success {
                            artifacts = Some(ArtifactMetadata {
                                image: self.config.image_tag.clone(),
                                binaries: self.find_binaries().await,
                                built_at: Utc::now().to_rfc3339(),
                            });
                        }
                    }
                } else {
                    overall_success = false;
                }
            }
        }

        let total_duration_secs = start_time.elapsed().as_secs();

        let result = PipelineResult {
            job_id: self.config.job_id.clone(),
            success: overall_success,
            steps: step_results,
            total_duration_secs,
            artifacts,
        };

        info!(
            job_id = %self.config.job_id,
            success = overall_success,
            duration_secs = total_duration_secs,
            "Pipeline execution completed"
        );

        Ok(result)
    }

    /// Execute a single build step
    async fn execute_step(&self, step: BuildStep) -> Result<StepResult> {
        info!(
            job_id = %self.config.job_id,
            step = step.name(),
            "Executing build step: {}",
            step.description()
        );

        let start_time = std::time::Instant::now();

        // Send step start log to Redis
        self.log_to_redis(&format!(">>> Starting step: {}", step.description()))
            .await?;

        // Build the command
        let mut cmd = self.build_command(step)?;

        // Execute with timeout and capture output
        let result = timeout(STEP_TIMEOUT, self.run_command(&mut cmd, step)).await;

        let duration_secs = start_time.elapsed().as_secs();

        match result {
            Ok(Ok((exit_code, output))) => {
                let success = exit_code == 0;

                // Send step completion log to Redis
                let status = if success { "SUCCESS" } else { "FAILED" };
                self.log_to_redis(&format!(
                    "<<< Step {} {} (exit code: {}, duration: {}s)",
                    step.name(),
                    status,
                    exit_code,
                    duration_secs
                ))
                .await?;

                Ok(StepResult {
                    step: step.name().to_string(),
                    success,
                    exit_code: Some(exit_code),
                    duration_secs,
                    output,
                })
            }
            Ok(Err(e)) => {
                error!(
                    job_id = %self.config.job_id,
                    step = step.name(),
                    error = %e,
                    "Step failed with error"
                );

                self.log_to_redis(&format!("<<< Step {} FAILED: {}", step.name(), e))
                    .await?;

                Ok(StepResult {
                    step: step.name().to_string(),
                    success: false,
                    exit_code: None,
                    duration_secs,
                    output: format!("Error: {}", e),
                })
            }
            Err(_) => {
                warn!(
                    job_id = %self.config.job_id,
                    step = step.name(),
                    "Step timeout exceeded"
                );

                self.log_to_redis(&format!(
                    "<<< Step {} TIMEOUT after {}s",
                    step.name(),
                    duration_secs
                ))
                .await?;

                Ok(StepResult {
                    step: step.name().to_string(),
                    success: false,
                    exit_code: None,
                    duration_secs,
                    output: format!("Step timeout exceeded ({} seconds)", STEP_TIMEOUT.as_secs()),
                })
            }
        }
    }

    /// Build the command for a specific step
    fn build_command(&self, step: BuildStep) -> Result<Command> {
        let mut cmd = match step {
            BuildStep::Check => {
                let mut c = Command::new("cargo");
                c.args(["check", "--all-features"]);
                c
            }
            BuildStep::Clippy => {
                let mut c = Command::new("cargo");
                c.args(["clippy", "--all-features", "--", "-D", "warnings"]);
                c
            }
            BuildStep::Format => {
                let mut c = Command::new("cargo");
                c.args(["fmt", "--", "--check"]);
                c
            }
            BuildStep::Test => {
                let mut c = Command::new("cargo");
                c.args(["test", "--all-features"]);
                c
            }
            BuildStep::Build => {
                let mut c = Command::new("cargo");
                c.args(["build", "--release"]);
                c
            }
            BuildStep::Audit => {
                let mut c = Command::new("cargo");
                c.args(["audit"]);
                c
            }
            BuildStep::DockerBuild => {
                let mut c = Command::new("docker");
                c.arg("build");
                if let Some(ref tag) = self.config.image_tag {
                    c.args(["-t", tag]);
                }
                c.arg(".");
                c
            }
            BuildStep::DockerPush => {
                let mut c = Command::new("docker");
                c.arg("push");
                if let Some(ref tag) = self.config.image_tag {
                    c.arg(tag);
                }
                c
            }
        };

        // Set working directory
        cmd.current_dir(&self.config.repo_path);

        // Set environment for sccache if enabled
        if self.config.use_sccache
            && matches!(step, BuildStep::Build | BuildStep::Check | BuildStep::Test)
        {
            cmd.env("RUSTC_WRAPPER", "sccache");
        }

        // Ensure output is captured
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        Ok(cmd)
    }

    /// Run a command and capture output, streaming logs to Redis
    async fn run_command(&self, cmd: &mut Command, step: BuildStep) -> Result<(i32, String)> {
        let mut child = cmd
            .spawn()
            .context(format!("Failed to spawn {} command", step.name()))?;

        let stdout = child.stdout.take().context("Failed to capture stdout")?;
        let stderr = child.stderr.take().context("Failed to capture stderr")?;

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let mut output_buffer = String::new();
        const MAX_OUTPUT_SIZE: usize = 10 * 1024; // 10KB

        // Stream output lines
        loop {
            tokio::select! {
                line = stdout_reader.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            // Add to buffer (truncate if too large)
                            if output_buffer.len() < MAX_OUTPUT_SIZE {
                                output_buffer.push_str(&line);
                                output_buffer.push('\n');
                            }
                            // Stream to Redis
                            self.log_to_redis(&line).await.ok();
                            debug!(step = step.name(), "stdout: {}", line);
                        }
                        Ok(None) => {}
                        Err(e) => {
                            warn!(step = step.name(), error = %e, "Error reading stdout");
                        }
                    }
                }
                line = stderr_reader.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            // Add to buffer (truncate if too large)
                            if output_buffer.len() < MAX_OUTPUT_SIZE {
                                output_buffer.push_str(&line);
                                output_buffer.push('\n');
                            }
                            // Stream to Redis
                            self.log_to_redis(&line).await.ok();
                            debug!(step = step.name(), "stderr: {}", line);
                        }
                        Ok(None) => {}
                        Err(e) => {
                            warn!(step = step.name(), error = %e, "Error reading stderr");
                        }
                    }
                }
                else => break,
            }
        }

        // Wait for process to exit
        let status = child
            .wait()
            .await
            .context("Failed to wait for child process")?;
        let exit_code = status.code().unwrap_or(-1);

        Ok((exit_code, output_buffer))
    }

    /// Stream a log line to Redis
    async fn log_to_redis(&self, message: &str) -> Result<()> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client
                .get_multiplexed_async_connection()
                .await
                .context("Failed to get Redis connection")?;

            let stream_key = format!("raibid:logs:{}", self.config.job_id);
            let timestamp = Utc::now().to_rfc3339();

            redis::cmd("XADD")
                .arg(&stream_key)
                .arg("*") // Auto-generate ID
                .arg("timestamp")
                .arg(&timestamp)
                .arg("message")
                .arg(message)
                .query_async::<_, ()>(&mut conn)
                .await
                .context("Failed to write log to Redis")?;
        }

        Ok(())
    }

    /// Find built binaries in target/release
    async fn find_binaries(&self) -> Vec<String> {
        let release_dir = self.config.repo_path.join("target/release");
        let mut binaries = Vec::new();

        if let Ok(mut entries) = tokio::fs::read_dir(&release_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(metadata) = entry.metadata().await {
                    if metadata.is_file() {
                        // Check if it's executable (on Unix)
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            if metadata.permissions().mode() & 0o111 != 0 {
                                if let Some(name) = entry.file_name().to_str() {
                                    // Exclude files with extensions
                                    if !name.contains('.') {
                                        binaries.push(name.to_string());
                                    }
                                }
                            }
                        }

                        #[cfg(not(unix))]
                        {
                            if let Some(name) = entry.file_name().to_str() {
                                if name.ends_with(".exe") {
                                    binaries.push(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        binaries
    }

    /// Update job status in Redis
    pub async fn update_job_status(&self, status: &str, exit_code: Option<i32>) -> Result<()> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client
                .get_multiplexed_async_connection()
                .await
                .context("Failed to get Redis connection")?;

            let job_key = format!("raibid:job:{}", self.config.job_id);
            let timestamp = Utc::now().to_rfc3339();

            redis::cmd("HSET")
                .arg(&job_key)
                .arg("status")
                .arg(status)
                .arg("updated_at")
                .arg(&timestamp)
                .query_async::<_, ()>(&mut conn)
                .await
                .context("Failed to update job status")?;

            if let Some(code) = exit_code {
                redis::cmd("HSET")
                    .arg(&job_key)
                    .arg("exit_code")
                    .arg(code)
                    .query_async::<_, ()>(&mut conn)
                    .await
                    .context("Failed to update exit code")?;
            }

            info!(
                job_id = %self.config.job_id,
                status = status,
                exit_code = ?exit_code,
                "Updated job status in Redis"
            );
        }

        Ok(())
    }

    /// Store artifact metadata in Redis
    pub async fn store_artifacts(&self, artifacts: &ArtifactMetadata) -> Result<()> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client
                .get_multiplexed_async_connection()
                .await
                .context("Failed to get Redis connection")?;

            let artifacts_key = format!("raibid:artifacts:{}", self.config.job_id);
            let artifacts_json = serde_json::to_string(artifacts)?;

            redis::cmd("SET")
                .arg(&artifacts_key)
                .arg(&artifacts_json)
                .arg("EX")
                .arg(86400 * 7) // Expire after 7 days
                .query_async::<_, ()>(&mut conn)
                .await
                .context("Failed to store artifacts metadata")?;

            info!(
                job_id = %self.config.job_id,
                "Stored artifact metadata in Redis"
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    /// Helper to create a minimal Rust project for testing
    async fn create_test_project(dir: &Path) -> Result<()> {
        // Create Cargo.toml
        let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
        tokio::fs::write(dir.join("Cargo.toml"), cargo_toml).await?;

        // Create src directory
        tokio::fs::create_dir(dir.join("src")).await?;

        // Create main.rs
        let main_rs = r#"
fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic() {
        assert_eq!(2 + 2, 4);
    }
}
"#;
        tokio::fs::write(dir.join("src/main.rs"), main_rs).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_build_step_names() {
        assert_eq!(BuildStep::Check.name(), "check");
        assert_eq!(BuildStep::Clippy.name(), "clippy");
        assert_eq!(BuildStep::Format.name(), "format");
        assert_eq!(BuildStep::Test.name(), "test");
        assert_eq!(BuildStep::Build.name(), "build");
        assert_eq!(BuildStep::Audit.name(), "audit");
        assert_eq!(BuildStep::DockerBuild.name(), "docker-build");
        assert_eq!(BuildStep::DockerPush.name(), "docker-push");
    }

    #[tokio::test]
    async fn test_pipeline_config_creation() {
        let temp_dir = TempDir::new().unwrap();

        let config = PipelineConfig {
            job_id: "test-job-123".to_string(),
            repo_path: temp_dir.path().to_path_buf(),
            use_sccache: true,
            registry_url: Some("https://gitea.example.com".to_string()),
            image_tag: Some("test:latest".to_string()),
            redis_url: None,
        };

        assert_eq!(config.job_id, "test-job-123");
        assert!(config.use_sccache);
    }

    #[tokio::test]
    async fn test_executor_creation() {
        let temp_dir = TempDir::new().unwrap();

        let config = PipelineConfig {
            job_id: "test-job-456".to_string(),
            repo_path: temp_dir.path().to_path_buf(),
            use_sccache: false,
            registry_url: None,
            image_tag: None,
            redis_url: None,
        };

        let executor = PipelineExecutor::new(config);
        assert!(executor.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires cargo to be installed
    async fn test_check_step() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path()).await.unwrap();

        let config = PipelineConfig {
            job_id: "test-check".to_string(),
            repo_path: temp_dir.path().to_path_buf(),
            use_sccache: false,
            registry_url: None,
            image_tag: None,
            redis_url: None,
        };

        let executor = PipelineExecutor::new(config).unwrap();
        let result = executor.execute_step(BuildStep::Check).await;

        assert!(result.is_ok());
        let step_result = result.unwrap();
        assert!(step_result.success);
        assert_eq!(step_result.step, "check");
    }

    #[tokio::test]
    #[ignore] // Requires cargo to be installed
    async fn test_test_step() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path()).await.unwrap();

        let config = PipelineConfig {
            job_id: "test-test".to_string(),
            repo_path: temp_dir.path().to_path_buf(),
            use_sccache: false,
            registry_url: None,
            image_tag: None,
            redis_url: None,
        };

        let executor = PipelineExecutor::new(config).unwrap();
        let result = executor.execute_step(BuildStep::Test).await;

        assert!(result.is_ok());
        let step_result = result.unwrap();
        assert!(step_result.success);
        assert_eq!(step_result.step, "test");
    }

    #[tokio::test]
    async fn test_pipeline_result_serialization() {
        let result = PipelineResult {
            job_id: "job-789".to_string(),
            success: true,
            steps: vec![StepResult {
                step: "check".to_string(),
                success: true,
                exit_code: Some(0),
                duration_secs: 5,
                output: "Checking complete".to_string(),
            }],
            total_duration_secs: 10,
            artifacts: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("job-789"));

        let deserialized: PipelineResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.job_id, "job-789");
        assert!(deserialized.success);
    }

    #[tokio::test]
    async fn test_artifact_metadata() {
        let metadata = ArtifactMetadata {
            image: Some("raibid/test:latest".to_string()),
            binaries: vec!["test-project".to_string()],
            built_at: Utc::now().to_rfc3339(),
        };

        assert_eq!(metadata.image.as_ref().unwrap(), "raibid/test:latest");
        assert_eq!(metadata.binaries.len(), 1);
    }
}
