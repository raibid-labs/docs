//! Mock Builders
//!
//! Builders for creating mock external services used in tests.

use std::path::PathBuf;

/// Mock k3s cluster for testing
///
/// Simulates a k3s cluster without actually running k3s.
pub struct MockCluster {
    pub name: String,
    pub kubeconfig_path: PathBuf,
    pub api_endpoint: String,
}

impl MockCluster {
    /// Create a new mock cluster
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            kubeconfig_path: PathBuf::from("/tmp/mock-kubeconfig"),
            api_endpoint: "https://127.0.0.1:6443".to_string(),
        }
    }

    /// Set custom kubeconfig path
    pub fn with_kubeconfig(mut self, path: PathBuf) -> Self {
        self.kubeconfig_path = path;
        self
    }

    /// Set custom API endpoint
    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.api_endpoint = endpoint.to_string();
        self
    }

    /// Generate a mock kubeconfig file
    pub fn generate_kubeconfig(&self) -> String {
        format!(
            r#"apiVersion: v1
kind: Config
clusters:
- cluster:
    server: {}
  name: {}
contexts:
- context:
    cluster: {}
    user: default
  name: default
current-context: default
users:
- name: default
  user:
    token: mock-token
"#,
            self.api_endpoint, self.name, self.name
        )
    }
}

/// Mock Gitea instance for testing
///
/// Simulates a Gitea server without actually running Gitea.
pub struct MockGitea {
    pub url: String,
    pub admin_user: String,
    pub admin_token: String,
}

impl MockGitea {
    /// Create a new mock Gitea instance
    pub fn new() -> Self {
        Self {
            url: "http://localhost:3000".to_string(),
            admin_user: "admin".to_string(),
            admin_token: "mock-token-1234567890".to_string(),
        }
    }

    /// Set custom URL
    pub fn with_url(mut self, url: &str) -> Self {
        self.url = url.to_string();
        self
    }

    /// Set custom admin credentials
    pub fn with_admin(mut self, user: &str, token: &str) -> Self {
        self.admin_user = user.to_string();
        self.admin_token = token.to_string();
        self
    }

    /// Generate mock API response for repository creation
    pub fn mock_create_repo_response(&self, name: &str) -> String {
        format!(
            r#"{{
  "id": 1,
  "name": "{}",
  "full_name": "{}/{}",
  "owner": {{
    "login": "{}",
    "id": 1
  }},
  "private": false,
  "html_url": "{}/{}/{}",
  "clone_url": "{}/{}/{}.git",
  "ssh_url": "git@localhost:{}/{}.git"
}}"#,
            name,
            self.admin_user,
            name,
            self.admin_user,
            self.url,
            self.admin_user,
            name,
            self.url,
            self.admin_user,
            name,
            self.admin_user,
            name
        )
    }
}

impl Default for MockGitea {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock Redis instance for testing
///
/// Simulates a Redis server without actually running Redis.
pub struct MockRedis {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
}

impl MockRedis {
    /// Create a new mock Redis instance
    pub fn new() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 6379,
            password: None,
        }
    }

    /// Set custom host
    pub fn with_host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    /// Set custom port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set password
    pub fn with_password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }

    /// Generate connection URL
    pub fn connection_url(&self) -> String {
        if let Some(ref password) = self.password {
            format!("redis://:{}@{}:{}", password, self.host, self.port)
        } else {
            format!("redis://{}:{}", self.host, self.port)
        }
    }
}

impl Default for MockRedis {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_cluster_creation() {
        let cluster = MockCluster::new("test-cluster");
        assert_eq!(cluster.name, "test-cluster");
        assert_eq!(cluster.api_endpoint, "https://127.0.0.1:6443");
    }

    #[test]
    fn test_mock_cluster_kubeconfig() {
        let cluster = MockCluster::new("test-cluster");
        let kubeconfig = cluster.generate_kubeconfig();

        assert!(kubeconfig.contains("test-cluster"));
        assert!(kubeconfig.contains("https://127.0.0.1:6443"));
        assert!(kubeconfig.contains("mock-token"));
    }

    #[test]
    fn test_mock_gitea_creation() {
        let gitea = MockGitea::new();
        assert_eq!(gitea.url, "http://localhost:3000");
        assert_eq!(gitea.admin_user, "admin");
    }

    #[test]
    fn test_mock_gitea_repo_response() {
        let gitea = MockGitea::new();
        let response = gitea.mock_create_repo_response("test-repo");

        assert!(response.contains("test-repo"));
        assert!(response.contains("admin/test-repo"));
    }

    #[test]
    fn test_mock_redis_connection_url() {
        let redis = MockRedis::new();
        assert_eq!(redis.connection_url(), "redis://localhost:6379");

        let redis_with_pass = MockRedis::new().with_password("secret");
        assert_eq!(
            redis_with_pass.connection_url(),
            "redis://:secret@localhost:6379"
        );
    }
}
