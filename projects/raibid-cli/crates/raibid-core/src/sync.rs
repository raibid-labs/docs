//! Repository synchronization engine

use crate::error::Result;
use crate::git::{construct_repo_path, GitOps};
use crate::types::{Repository, SyncOptions, SyncResult, SyncStatus};
use std::path::Path;
use std::time::Instant;
use tracing::{error, info, warn};

/// Synchronize a single repository
pub async fn sync_repository(
    repo: &Repository,
    workspace_root: &Path,
    options: &SyncOptions,
) -> SyncResult {
    let start = Instant::now();
    let repo_path = construct_repo_path(workspace_root, repo);

    info!("Syncing repository: {}", repo.name);

    // Check if repository exists locally
    let local_state = match GitOps::get_local_state(&repo_path) {
        Ok(state) => state,
        Err(e) => {
            error!("Failed to get local state for {}: {}", repo.name, e);
            return SyncResult {
                repository: repo.clone(),
                status: SyncStatus::Failed,
                path: repo_path,
                error: Some(e.to_string()),
                was_cloned: false,
                commits_fetched: 0,
                duration: start.elapsed(),
            };
        }
    };

    // If dry run, just report what would happen
    if options.dry_run {
        let status = if !local_state.exists {
            SyncStatus::Pending // Would clone
        } else {
            SyncStatus::UpToDate // Would sync
        };

        return SyncResult {
            repository: repo.clone(),
            status,
            path: repo_path,
            error: None,
            was_cloned: false,
            commits_fetched: 0,
            duration: start.elapsed(),
        };
    }

    // Clone if doesn't exist
    if !local_state.exists {
        info!("Cloning {} to {}", repo.name, repo_path.display());

        let url = if options.use_ssh {
            &repo.ssh_url
        } else {
            &repo.clone_url
        };

        match GitOps::clone(url, &repo_path, options.depth) {
            Ok(_) => {
                info!("Successfully cloned {}", repo.name);
                SyncResult {
                    repository: repo.clone(),
                    status: SyncStatus::Success,
                    path: repo_path,
                    error: None,
                    was_cloned: true,
                    commits_fetched: 0, // TODO: Track commits
                    duration: start.elapsed(),
                }
            }
            Err(e) => {
                error!("Failed to clone {}: {}", repo.name, e);
                SyncResult {
                    repository: repo.clone(),
                    status: SyncStatus::Failed,
                    path: repo_path,
                    error: Some(e.to_string()),
                    was_cloned: false,
                    commits_fetched: 0,
                    duration: start.elapsed(),
                }
            }
        }
    } else {
        // Repository exists, check if we can pull
        if local_state.has_uncommitted_changes && !options.force {
            warn!(
                "Skipping {} due to uncommitted changes (use --force to override)",
                repo.name
            );
            return SyncResult {
                repository: repo.clone(),
                status: SyncStatus::Skipped,
                path: repo_path,
                error: Some("Uncommitted changes detected".to_string()),
                was_cloned: false,
                commits_fetched: 0,
                duration: start.elapsed(),
            };
        }

        // Pull updates
        match GitOps::pull(&repo_path) {
            Ok(_) => {
                info!("Successfully synced {}", repo.name);
                SyncResult {
                    repository: repo.clone(),
                    status: SyncStatus::Success,
                    path: repo_path,
                    error: None,
                    was_cloned: false,
                    commits_fetched: 0, // TODO: Track commits
                    duration: start.elapsed(),
                }
            }
            Err(e) => {
                error!("Failed to sync {}: {}", repo.name, e);
                SyncResult {
                    repository: repo.clone(),
                    status: SyncStatus::Failed,
                    path: repo_path,
                    error: Some(e.to_string()),
                    was_cloned: false,
                    commits_fetched: 0,
                    duration: start.elapsed(),
                }
            }
        }
    }
}

/// Synchronize multiple repositories concurrently
pub async fn sync_repositories(
    repos: Vec<Repository>,
    workspace_root: &Path,
    options: &SyncOptions,
) -> Vec<SyncResult> {
    use futures::stream::{self, StreamExt};

    info!(
        "Starting sync of {} repositories with concurrency {}",
        repos.len(),
        options.concurrency
    );

    let results: Vec<SyncResult> = stream::iter(repos)
        .map(|repo| {
            let workspace_root = workspace_root.to_path_buf();
            let options = options.clone();
            async move { sync_repository(&repo, &workspace_root, &options).await }
        })
        .buffer_unordered(options.concurrency)
        .collect()
        .await;

    let successful = results.iter().filter(|r| r.status.is_successful()).count();
    let failed = results
        .iter()
        .filter(|r| r.status == SyncStatus::Failed)
        .count();
    let skipped = results
        .iter()
        .filter(|r| r.status == SyncStatus::Skipped)
        .count();

    info!(
        "Sync complete: {} successful, {} failed, {} skipped",
        successful, failed, skipped
    );

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Repository;
    use chrono::Utc;

    fn create_test_repo(name: &str) -> Repository {
        Repository {
            name: name.to_string(),
            full_name: format!("org/{}", name),
            description: None,
            clone_url: format!("https://github.com/org/{}", name),
            ssh_url: format!("git@github.com:org/{}.git", name),
            default_branch: "main".to_string(),
            private: false,
            fork: false,
            archived: false,
            language: Some("Rust".to_string()),
            stargazers_count: 0,
            forks_count: 0,
            updated_at: Utc::now(),
            pushed_at: Utc::now(),
            topics: vec![],
        }
    }

    #[tokio::test]
    async fn test_sync_dry_run() {
        let repo = create_test_repo("test-repo");
        let workspace = tempfile::tempdir().unwrap();
        let options = SyncOptions {
            dry_run: true,
            ..Default::default()
        };

        let result = sync_repository(&repo, workspace.path(), &options).await;

        assert_eq!(result.status, SyncStatus::Pending);
        assert_eq!(result.was_cloned, false);
    }
}
