//! Git repository operations

use crate::error::{Error, Result};
use crate::types::{LocalRepoState, Repository};
use git2::{Repository as Git2Repository, StatusOptions};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Git operations handler
pub struct GitOps;

impl GitOps {
    /// Check if a path contains a valid git repository
    pub fn is_git_repo(path: &Path) -> bool {
        Git2Repository::open(path).is_ok()
    }

    /// Get the state of a local repository
    pub fn get_local_state(path: &Path) -> Result<LocalRepoState> {
        let exists = path.exists();
        let is_git_repo = exists && Self::is_git_repo(path);

        if !is_git_repo {
            return Ok(LocalRepoState {
                path: path.to_path_buf(),
                exists,
                is_git_repo: false,
                current_branch: None,
                has_uncommitted_changes: false,
                commits_behind: None,
                commits_ahead: None,
                last_sync: None,
            });
        }

        let repo = Git2Repository::open(path)?;

        // Get current branch
        let head = repo.head()?;
        let current_branch = head
            .shorthand()
            .map(|s| s.to_string());

        // Check for uncommitted changes
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        let statuses = repo.statuses(Some(&mut opts))?;
        let has_uncommitted_changes = !statuses.is_empty();

        Ok(LocalRepoState {
            path: path.to_path_buf(),
            exists: true,
            is_git_repo: true,
            current_branch,
            has_uncommitted_changes,
            commits_behind: None, // TODO: Calculate from remote
            commits_ahead: None,  // TODO: Calculate from remote
            last_sync: None,      // TODO: Track last sync time
        })
    }

    /// Clone a repository
    pub fn clone(url: &str, path: &Path, depth: u32) -> Result<()> {
        info!("Cloning {} to {}", url, path.display());

        let mut builder = git2::build::RepoBuilder::new();

        if depth > 0 {
            let mut fetch_options = git2::FetchOptions::new();
            fetch_options.depth(depth as i32);
            builder.fetch_options(fetch_options);
        }

        builder.clone(url, path)?;

        debug!("Successfully cloned {}", url);
        Ok(())
    }

    /// Pull updates for a repository
    pub fn pull(path: &Path) -> Result<()> {
        info!("Pulling updates for {}", path.display());

        let repo = Git2Repository::open(path)?;

        // TODO: Implement proper pull operation
        // For now, just fetch
        let mut remote = repo.find_remote("origin")?;
        remote.fetch(&["HEAD"], None, None)?;

        debug!("Successfully pulled updates for {}", path.display());
        Ok(())
    }

    /// Get repository URL
    pub fn get_remote_url(path: &Path) -> Result<String> {
        let repo = Git2Repository::open(path)?;
        let remote = repo.find_remote("origin")?;
        remote
            .url()
            .ok_or_else(|| Error::Git(git2::Error::from_str("No URL for remote origin")))
            .map(|s| s.to_string())
    }
}

/// Helper to construct local repository path
pub fn construct_repo_path(workspace_root: &Path, repo: &Repository) -> PathBuf {
    workspace_root.join(&repo.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_git_repo_false() {
        let path = Path::new("/nonexistent/path");
        assert!(!GitOps::is_git_repo(path));
    }

    #[test]
    fn test_construct_repo_path() {
        let workspace = Path::new("/home/user/workspace");
        let mut repo = Repository {
            name: "test-repo".to_string(),
            full_name: "org/test-repo".to_string(),
            description: None,
            clone_url: "https://github.com/org/test-repo".to_string(),
            ssh_url: "git@github.com:org/test-repo.git".to_string(),
            default_branch: "main".to_string(),
            private: false,
            fork: false,
            archived: false,
            language: None,
            stargazers_count: 0,
            forks_count: 0,
            updated_at: chrono::Utc::now(),
            pushed_at: chrono::Utc::now(),
            topics: vec![],
        };

        let path = construct_repo_path(workspace, &repo);
        assert_eq!(path, Path::new("/home/user/workspace/test-repo"));
    }
}
