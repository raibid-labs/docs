//! Common types used throughout raibid-core

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Repository metadata from GitHub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Repository name
    pub name: String,

    /// Full name (org/repo)
    pub full_name: String,

    /// Description
    pub description: Option<String>,

    /// Clone URL (HTTPS)
    pub clone_url: String,

    /// SSH URL
    pub ssh_url: String,

    /// Default branch
    pub default_branch: String,

    /// Is private?
    pub private: bool,

    /// Is fork?
    pub fork: bool,

    /// Is archived?
    pub archived: bool,

    /// Primary language
    pub language: Option<String>,

    /// Star count
    pub stargazers_count: u32,

    /// Fork count
    pub forks_count: u32,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Last pushed timestamp
    pub pushed_at: DateTime<Utc>,

    /// Topics/tags
    pub topics: Vec<String>,
}

/// Repository synchronization status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    /// Not yet processed
    Pending,

    /// Currently being processed
    InProgress,

    /// Successfully synced
    Success,

    /// Sync failed
    Failed,

    /// Skipped (e.g., uncommitted changes)
    Skipped,

    /// Already up to date
    UpToDate,
}

impl SyncStatus {
    /// Returns true if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            SyncStatus::Success
                | SyncStatus::Failed
                | SyncStatus::Skipped
                | SyncStatus::UpToDate
        )
    }

    /// Returns true if this status represents a successful outcome
    pub fn is_successful(&self) -> bool {
        matches!(self, SyncStatus::Success | SyncStatus::UpToDate)
    }
}

/// Local repository state
#[derive(Debug, Clone)]
pub struct LocalRepoState {
    /// Local path
    pub path: PathBuf,

    /// Does the repository exist locally?
    pub exists: bool,

    /// Is it a valid git repository?
    pub is_git_repo: bool,

    /// Current branch
    pub current_branch: Option<String>,

    /// Has uncommitted changes?
    pub has_uncommitted_changes: bool,

    /// Commits behind remote
    pub commits_behind: Option<usize>,

    /// Commits ahead of remote
    pub commits_ahead: Option<usize>,

    /// Last sync timestamp
    pub last_sync: Option<DateTime<Utc>>,
}

/// Synchronization result for a single repository
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Repository information
    pub repository: Repository,

    /// Synchronization status
    pub status: SyncStatus,

    /// Local path
    pub path: PathBuf,

    /// Error message if failed
    pub error: Option<String>,

    /// Was this a clone or an update?
    pub was_cloned: bool,

    /// Number of commits fetched
    pub commits_fetched: usize,

    /// Duration of operation
    pub duration: std::time::Duration,
}

/// Filter criteria for repositories
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterCriteria {
    /// Include patterns (glob)
    pub include: Vec<String>,

    /// Exclude patterns (glob)
    pub exclude: Vec<String>,

    /// Exclude archived repositories
    pub exclude_archived: bool,

    /// Exclude forks
    pub exclude_forks: bool,

    /// Filter by language
    pub language: Option<String>,

    /// Minimum stars
    pub min_stars: Option<u32>,

    /// Updated after date
    pub updated_after: Option<DateTime<Utc>>,
}

/// Sync operation options
#[derive(Debug, Clone)]
pub struct SyncOptions {
    /// Number of concurrent operations
    pub concurrency: usize,

    /// Perform dry run (no actual changes)
    pub dry_run: bool,

    /// Force sync even with uncommitted changes
    pub force: bool,

    /// Clone depth (0 = full clone)
    pub depth: u32,

    /// Use SSH instead of HTTPS
    pub use_ssh: bool,

    /// Repositories to sync (if empty, sync all)
    pub repositories: Vec<String>,

    /// Filter criteria
    pub filter: Option<FilterCriteria>,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            concurrency: 5,
            dry_run: false,
            force: false,
            depth: 0,
            use_ssh: true,
            repositories: Vec::new(),
            filter: None,
        }
    }
}
