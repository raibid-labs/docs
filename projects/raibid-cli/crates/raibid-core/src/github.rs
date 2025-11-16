//! GitHub API integration

use crate::error::{Error, Result};
use crate::types::Repository;
use serde::Deserialize;
use std::process::Command;

/// GitHub API client
pub struct GitHubClient {
    org: String,
}

impl GitHubClient {
    /// Create a new GitHub client for the specified organization
    pub fn new(org: String) -> Self {
        Self { org }
    }

    /// List all repositories in the organization using gh CLI
    pub async fn list_repositories(&self) -> Result<Vec<Repository>> {
        // Use gh CLI to fetch repositories
        let output = Command::new("gh")
            .args([
                "api",
                &format!("/orgs/{}/repos", self.org),
                "--paginate",
                "--jq",
                ".",
            ])
            .output()
            .map_err(|e| Error::GitHub(format!("Failed to execute gh command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::GitHub(format!("gh command failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let gh_repos: Vec<GitHubRepo> = serde_json::from_str(&stdout)?;

        Ok(gh_repos.into_iter().map(|r| r.into()).collect())
    }
}

/// GitHub API repository response
#[derive(Debug, Deserialize)]
struct GitHubRepo {
    name: String,
    full_name: String,
    description: Option<String>,
    clone_url: String,
    ssh_url: String,
    default_branch: String,
    private: bool,
    fork: bool,
    archived: bool,
    language: Option<String>,
    stargazers_count: u32,
    forks_count: u32,
    updated_at: String,
    pushed_at: String,
    topics: Vec<String>,
}

impl From<GitHubRepo> for Repository {
    fn from(gh: GitHubRepo) -> Self {
        use chrono::DateTime;

        Self {
            name: gh.name,
            full_name: gh.full_name,
            description: gh.description,
            clone_url: gh.clone_url,
            ssh_url: gh.ssh_url,
            default_branch: gh.default_branch,
            private: gh.private,
            fork: gh.fork,
            archived: gh.archived,
            language: gh.language,
            stargazers_count: gh.stargazers_count,
            forks_count: gh.forks_count,
            updated_at: DateTime::parse_from_rfc3339(&gh.updated_at)
                .unwrap()
                .into(),
            pushed_at: DateTime::parse_from_rfc3339(&gh.pushed_at)
                .unwrap()
                .into(),
            topics: gh.topics,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_client_creation() {
        let client = GitHubClient::new("raibid-labs".to_string());
        assert_eq!(client.org, "raibid-labs");
    }
}
