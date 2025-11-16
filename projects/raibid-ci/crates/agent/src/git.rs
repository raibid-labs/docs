//! Git repository cloning and management

use crate::error::{AgentError, AgentResult};
use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks, Repository};
use std::path::PathBuf;
use tracing::{debug, info};

/// Git repository manager
#[derive(Clone)]
pub struct GitManager {
    workspace_dir: PathBuf,
}

impl GitManager {
    /// Create a new git manager
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self { workspace_dir }
    }

    /// Clone a repository to the workspace
    ///
    /// # Arguments
    /// * `repo_url` - Repository URL (e.g., "https://github.com/user/repo.git")
    /// * `branch` - Branch name to checkout
    /// * `commit` - Optional commit SHA to checkout
    ///
    /// # Returns
    /// Path to the cloned repository
    pub fn clone_repository(
        &self,
        repo_url: &str,
        branch: &str,
        commit: Option<&str>,
    ) -> AgentResult<PathBuf> {
        info!("Cloning repository: {} (branch: {})", repo_url, branch);

        // Create workspace directory if it doesn't exist
        std::fs::create_dir_all(&self.workspace_dir)?;

        // Extract repo name from URL
        let repo_name = Self::extract_repo_name(repo_url)?;
        let repo_path = self.workspace_dir.join(&repo_name);

        // Remove existing directory if it exists
        if repo_path.exists() {
            debug!("Removing existing repository at {:?}", repo_path);
            std::fs::remove_dir_all(&repo_path)?;
        }

        // Setup callbacks for authentication
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            // Try SSH key first, then default credentials
            if let Some(username) = username_from_url {
                Cred::ssh_key_from_agent(username)
            } else {
                Cred::default()
            }
        });

        // Setup fetch options
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Clone the repository
        let mut builder = RepoBuilder::new();
        builder.branch(branch);
        builder.fetch_options(fetch_options);

        let repo = builder.clone(repo_url, &repo_path)?;
        info!("Repository cloned to {:?}", repo_path);

        // Checkout specific commit if provided
        if let Some(commit_sha) = commit {
            self.checkout_commit(&repo, commit_sha)?;
        }

        Ok(repo_path)
    }

    /// Checkout a specific commit
    fn checkout_commit(&self, repo: &Repository, commit_sha: &str) -> AgentResult<()> {
        debug!("Checking out commit: {}", commit_sha);

        let oid = git2::Oid::from_str(commit_sha).map_err(AgentError::Git)?;

        let commit = repo.find_commit(oid)?;

        // Detach HEAD and checkout the commit
        repo.set_head_detached(commit.id())?;

        // Checkout the commit
        let mut checkout_builder = git2::build::CheckoutBuilder::new();
        checkout_builder.force();

        repo.checkout_head(Some(&mut checkout_builder))?;

        info!("Checked out commit: {}", commit_sha);

        Ok(())
    }

    /// Extract repository name from URL
    fn extract_repo_name(repo_url: &str) -> AgentResult<String> {
        // Handle URLs like:
        // - https://github.com/user/repo.git
        // - git@github.com:user/repo.git
        // - https://github.com/user/repo

        let url = repo_url.trim_end_matches('/');

        // Remove .git suffix if present
        let url = url.trim_end_matches(".git");

        // Extract the last path component
        let name = url
            .rsplit('/')
            .next()
            .or_else(|| url.rsplit(':').next())
            .ok_or_else(|| {
                AgentError::Configuration(format!("Invalid repository URL: {}", repo_url))
            })?;

        if name.is_empty() {
            return Err(AgentError::Configuration(format!(
                "Invalid repository URL: {}",
                repo_url
            )));
        }

        Ok(name.to_string())
    }

    /// Clean up workspace directory
    pub fn cleanup(&self) -> AgentResult<()> {
        if self.workspace_dir.exists() {
            info!("Cleaning up workspace: {:?}", self.workspace_dir);
            std::fs::remove_dir_all(&self.workspace_dir)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_repo_name_https() {
        assert_eq!(
            GitManager::extract_repo_name("https://github.com/user/repo.git").unwrap(),
            "repo"
        );
        assert_eq!(
            GitManager::extract_repo_name("https://github.com/user/repo").unwrap(),
            "repo"
        );
    }

    #[test]
    fn test_extract_repo_name_ssh() {
        assert_eq!(
            GitManager::extract_repo_name("git@github.com:user/repo.git").unwrap(),
            "repo"
        );
    }

    #[test]
    fn test_extract_repo_name_trailing_slash() {
        assert_eq!(
            GitManager::extract_repo_name("https://github.com/user/repo/").unwrap(),
            "repo"
        );
    }

    #[test]
    fn test_extract_repo_name_invalid() {
        assert!(GitManager::extract_repo_name("").is_err());
    }

    #[test]
    fn test_git_manager_creation() {
        let workspace = PathBuf::from("/tmp/test-workspace");
        let manager = GitManager::new(workspace.clone());
        assert_eq!(manager.workspace_dir, workspace);
    }
}
