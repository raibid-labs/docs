//! Job execution logic

use crate::config::AgentConfig;
use crate::error::{AgentError, AgentResult};
use crate::git::GitManager;
use raibid_common::jobs::Job;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Job executor
pub struct JobExecutor {
    config: Arc<AgentConfig>,
    git_manager: GitManager,
}

impl JobExecutor {
    /// Create a new job executor
    pub fn new(config: Arc<AgentConfig>) -> Self {
        let git_manager = GitManager::new(config.workspace_dir.clone());

        Self {
            config,
            git_manager,
        }
    }

    /// Execute a job
    ///
    /// # Returns
    /// Exit code of the build process
    pub async fn execute(&self, job: &Job) -> AgentResult<i32> {
        info!("Executing job: {}", job.id);

        // Step 1: Clone the repository
        let repo_path = self.clone_repository(job).await?;

        // Step 2: Execute the build pipeline
        let exit_code = self.run_build_pipeline(&repo_path, job).await?;

        // Step 3: Clean up (optional - keep for debugging in dev)
        if let Err(e) = self.cleanup(&repo_path) {
            warn!("Failed to cleanup workspace: {}", e);
        }

        Ok(exit_code)
    }

    /// Clone the repository for the job
    async fn clone_repository(&self, job: &Job) -> AgentResult<PathBuf> {
        // Construct repository URL
        // For MVP, assume repositories are in Gitea
        // Format: http://gitea.raibid-gitea.svc.cluster.local:3000/{repo}.git
        let repo_url = self.construct_repo_url(&job.repo);

        info!("Cloning repository: {}", repo_url);

        // Clone in a blocking task since git2 is synchronous
        let git_manager = self.git_manager.clone();
        let repo_url_clone = repo_url.clone();
        let branch = job.branch.clone();
        let commit = if !job.commit.is_empty() {
            Some(job.commit.clone())
        } else {
            None
        };

        let repo_path = tokio::task::spawn_blocking(move || {
            git_manager.clone_repository(&repo_url_clone, &branch, commit.as_deref())
        })
        .await
        .map_err(|e| AgentError::Internal(format!("Task join error: {}", e)))??;

        info!("Repository cloned to: {:?}", repo_path);

        Ok(repo_path)
    }

    /// Construct repository URL from repo name
    fn construct_repo_url(&self, repo_name: &str) -> String {
        // For MVP, use environment variable or default to Gitea in cluster
        let gitea_host = std::env::var("GITEA_HOST")
            .unwrap_or_else(|_| "gitea.raibid-gitea.svc.cluster.local:3000".to_string());

        format!("http://{}/{}.git", gitea_host, repo_name)
    }

    /// Run the build pipeline
    async fn run_build_pipeline(&self, repo_path: &PathBuf, job: &Job) -> AgentResult<i32> {
        info!("Running build pipeline for job: {}", job.id);

        // For MVP, execute a simple Rust build
        // In the future, this should be configurable based on agent type
        let exit_code = self.run_rust_build(repo_path).await?;

        Ok(exit_code)
    }

    /// Run Rust build pipeline
    async fn run_rust_build(&self, repo_path: &PathBuf) -> AgentResult<i32> {
        info!("Running Rust build");

        // Step 1: cargo check
        let check_exit = self.run_command(repo_path, "cargo", &["check"]).await?;
        if check_exit != 0 {
            warn!("cargo check failed with exit code {}", check_exit);
            return Ok(check_exit);
        }

        // Step 2: cargo test
        let test_exit = self.run_command(repo_path, "cargo", &["test"]).await?;
        if test_exit != 0 {
            warn!("cargo test failed with exit code {}", test_exit);
            return Ok(test_exit);
        }

        // Step 3: cargo build --release
        let build_exit = self
            .run_command(repo_path, "cargo", &["build", "--release"])
            .await?;
        if build_exit != 0 {
            warn!("cargo build failed with exit code {}", build_exit);
            return Ok(build_exit);
        }

        info!("Rust build completed successfully");
        Ok(0)
    }

    /// Run a command and stream output
    async fn run_command(
        &self,
        working_dir: &PathBuf,
        program: &str,
        args: &[&str],
    ) -> AgentResult<i32> {
        debug!("Running command: {} {}", program, args.join(" "));

        let mut child = Command::new(program)
            .args(args)
            .current_dir(working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                AgentError::BuildExecution(format!("Failed to spawn {}: {}", program, e))
            })?;

        // Stream stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            tokio::spawn(async move {
                while let Ok(Some(line)) = lines.next_line().await {
                    info!("[stdout] {}", line);
                }
            });
        }

        // Stream stderr
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            tokio::spawn(async move {
                while let Ok(Some(line)) = lines.next_line().await {
                    warn!("[stderr] {}", line);
                }
            });
        }

        // Wait for the command to complete
        let status = child.wait().await.map_err(|e| {
            AgentError::BuildExecution(format!("Failed to wait for {}: {}", program, e))
        })?;

        let exit_code = status.code().unwrap_or(-1);
        debug!("Command {} exited with code {}", program, exit_code);

        Ok(exit_code)
    }

    /// Clean up workspace after job execution
    fn cleanup(&self, repo_path: &PathBuf) -> AgentResult<()> {
        if repo_path.exists() {
            debug!("Cleaning up repository: {:?}", repo_path);
            std::fs::remove_dir_all(repo_path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct_repo_url() {
        let config = Arc::new(AgentConfig::default());
        let executor = JobExecutor::new(config);

        let url = executor.construct_repo_url("user/repo");
        assert!(url.contains("user/repo.git"));
    }

    #[test]
    fn test_construct_repo_url_with_env() {
        std::env::set_var("GITEA_HOST", "gitea.example.com:3000");

        let config = Arc::new(AgentConfig::default());
        let executor = JobExecutor::new(config);

        let url = executor.construct_repo_url("user/repo");
        assert_eq!(url, "http://gitea.example.com:3000/user/repo.git");

        std::env::remove_var("GITEA_HOST");
    }

    #[tokio::test]
    async fn test_run_command_success() {
        let config = Arc::new(AgentConfig::default());
        let executor = JobExecutor::new(config);

        let temp_dir = std::env::temp_dir();
        let exit_code = executor.run_command(&temp_dir, "echo", &["hello"]).await;

        assert!(exit_code.is_ok());
        assert_eq!(exit_code.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_run_command_failure() {
        let config = Arc::new(AgentConfig::default());
        let executor = JobExecutor::new(config);

        let temp_dir = std::env::temp_dir();
        let exit_code = executor.run_command(&temp_dir, "false", &[]).await;

        assert!(exit_code.is_ok());
        assert_ne!(exit_code.unwrap(), 0);
    }
}
