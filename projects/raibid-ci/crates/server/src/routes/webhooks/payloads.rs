//! Webhook payload structures for GitHub and Gitea

use serde::{Deserialize, Serialize};

/// Gitea webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiteaWebhookPayload {
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub repository: Repository,
    pub pusher: User,
    pub commits: Option<Vec<Commit>>,
}

/// GitHub webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubWebhookPayload {
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub repository: Repository,
    pub pusher: Pusher,
    pub commits: Option<Vec<Commit>>,
}

/// Repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: Owner,
    pub html_url: String,
    pub clone_url: String,
    pub ssh_url: String,
    pub default_branch: String,
}

/// User/Owner information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

/// GitHub pusher (slightly different from User)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pusher {
    pub name: String,
    pub email: Option<String>,
}

/// Owner information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub id: u64,
    pub login: String,
    pub avatar_url: Option<String>,
}

/// Commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub url: String,
    pub author: CommitAuthor,
    pub committer: CommitAuthor,
    pub timestamp: String,
}

/// Commit author/committer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
    pub username: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitea_payload_deserialization() {
        let json = r#"{
            "ref": "refs/heads/main",
            "before": "abc123",
            "after": "def456",
            "repository": {
                "id": 1,
                "name": "test-repo",
                "full_name": "owner/test-repo",
                "owner": {
                    "id": 1,
                    "login": "owner",
                    "avatar_url": "https://example.com/avatar.png"
                },
                "html_url": "https://git.example.com/owner/test-repo",
                "clone_url": "https://git.example.com/owner/test-repo.git",
                "ssh_url": "git@git.example.com:owner/test-repo.git",
                "default_branch": "main"
            },
            "pusher": {
                "id": 1,
                "username": "testuser",
                "email": "test@example.com",
                "avatar_url": "https://example.com/avatar.png"
            }
        }"#;

        let payload: GiteaWebhookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.ref_name, Some("refs/heads/main".to_string()));
        assert_eq!(payload.repository.full_name, "owner/test-repo");
        assert_eq!(payload.pusher.username, "testuser");
    }

    #[test]
    fn test_github_payload_deserialization() {
        let json = r#"{
            "ref": "refs/heads/main",
            "before": "abc123",
            "after": "def456",
            "repository": {
                "id": 1,
                "name": "test-repo",
                "full_name": "owner/test-repo",
                "owner": {
                    "id": 1,
                    "login": "owner",
                    "avatar_url": "https://example.com/avatar.png"
                },
                "html_url": "https://github.com/owner/test-repo",
                "clone_url": "https://github.com/owner/test-repo.git",
                "ssh_url": "git@github.com:owner/test-repo.git",
                "default_branch": "main"
            },
            "pusher": {
                "name": "testuser",
                "email": "test@example.com"
            }
        }"#;

        let payload: GitHubWebhookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.ref_name, Some("refs/heads/main".to_string()));
        assert_eq!(payload.repository.full_name, "owner/test-repo");
        assert_eq!(payload.pusher.name, "testuser");
    }

    #[test]
    fn test_repository_serialization() {
        let repo = Repository {
            id: 1,
            name: "test-repo".to_string(),
            full_name: "owner/test-repo".to_string(),
            owner: Owner {
                id: 1,
                login: "owner".to_string(),
                avatar_url: None,
            },
            html_url: "https://example.com/repo".to_string(),
            clone_url: "https://example.com/repo.git".to_string(),
            ssh_url: "git@example.com:repo.git".to_string(),
            default_branch: "main".to_string(),
        };

        let json = serde_json::to_string(&repo).unwrap();
        assert!(json.contains("test-repo"));
        assert!(json.contains("owner/test-repo"));
    }
}
