//! Test Data Generators
//!
//! Functions for generating test data, configurations, and fixtures.

use std::path::PathBuf;

/// Generate a test configuration YAML
///
/// Creates a minimal valid configuration for testing purposes.
pub fn generate_test_config() -> String {
    r#"# Test Configuration
cluster:
  name: test-cluster
  kubeconfig: ~/.kube/config
  namespace: raibid-ci

api:
  host: 0.0.0.0
  port: 8080
  log_level: info

agents:
  rust:
    enabled: true
    max_replicas: 3
    cache_size: 10Gi

redis:
  host: localhost
  port: 6379
  stream: build-jobs

gitea:
  url: http://localhost:3000
  admin_user: admin
  admin_token: test-token
"#
    .to_string()
}

/// Generate a test configuration with custom values
pub fn generate_custom_config(
    cluster_name: &str,
    api_port: u16,
    max_replicas: u32,
) -> String {
    format!(
        r#"cluster:
  name: {}
  kubeconfig: ~/.kube/config
  namespace: raibid-ci

api:
  host: 0.0.0.0
  port: {}
  log_level: info

agents:
  rust:
    enabled: true
    max_replicas: {}
    cache_size: 10Gi

redis:
  host: localhost
  port: 6379
  stream: build-jobs

gitea:
  url: http://localhost:3000
  admin_user: admin
  admin_token: test-token
"#,
        cluster_name, api_port, max_replicas
    )
}

/// Generate a minimal test configuration
pub fn generate_minimal_config() -> String {
    r#"cluster:
  name: test

api:
  port: 8080

agents:
  rust:
    enabled: true
"#
    .to_string()
}

/// Generate a test Kubernetes manifest
///
/// Creates a sample deployment manifest for testing.
pub fn generate_test_manifest(name: &str, image: &str, replicas: i32) -> String {
    format!(
        r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {}
  labels:
    app: {}
spec:
  replicas: {}
  selector:
    matchLabels:
      app: {}
  template:
    metadata:
      labels:
        app: {}
    spec:
      containers:
      - name: {}
        image: {}
        ports:
        - containerPort: 8080
"#,
        name, name, replicas, name, name, name, image
    )
}

/// Generate a test Kubernetes Service manifest
pub fn generate_test_service(name: &str, port: i32, target_port: i32) -> String {
    format!(
        r#"apiVersion: v1
kind: Service
metadata:
  name: {}
spec:
  selector:
    app: {}
  ports:
  - port: {}
    targetPort: {}
  type: ClusterIP
"#,
        name, name, port, target_port
    )
}

/// Generate a test Redis Stream job payload
pub fn generate_test_job(repo: &str, commit: &str, branch: &str) -> String {
    format!(
        r#"{{
  "job_id": "{}",
  "repo": "{}",
  "commit": "{}",
  "branch": "{}",
  "timestamp": "2025-11-01T00:00:00Z",
  "agent_type": "rust",
  "build_config": {{
    "cache_enabled": true,
    "target": "x86_64-unknown-linux-gnu"
  }}
}}"#,
        uuid::Uuid::new_v4(),
        repo,
        commit,
        branch
    )
}

/// Generate a random project name for testing
pub fn generate_project_name() -> String {
    format!("test-project-{}", rand::random::<u32>())
}

/// Generate a random commit hash for testing
pub fn generate_commit_hash() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..40)
        .map(|_| format!("{:x}", rng.gen_range(0..16)))
        .collect()
}

/// Create a temporary Git repository for testing
///
/// Returns the path to the temporary repository.
/// The repository will be automatically cleaned up when the returned TempDir is dropped.
pub fn create_temp_git_repo() -> (tempfile::TempDir, PathBuf) {
    use std::process::Command;

    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    Command::new("git")
        .args(&["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to init git repo");

    // Configure git user
    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to configure git user.name");

    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to configure git user.email");

    // Create initial commit
    std::fs::write(repo_path.join("README.md"), "# Test Repository\n")
        .expect("Failed to create README");

    Command::new("git")
        .args(&["add", "README.md"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to git add");

    Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to git commit");

    (temp_dir, repo_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_test_config() {
        let config = generate_test_config();
        assert!(config.contains("cluster:"));
        assert!(config.contains("api:"));
        assert!(config.contains("agents:"));
    }

    #[test]
    fn test_generate_custom_config() {
        let config = generate_custom_config("my-cluster", 9090, 5);
        assert!(config.contains("my-cluster"));
        assert!(config.contains("9090"));
        assert!(config.contains("max_replicas: 5"));
    }

    #[test]
    fn test_generate_minimal_config() {
        let config = generate_minimal_config();
        assert!(config.contains("cluster:"));
        assert!(config.contains("name: test"));
        // Should be minimal, so shouldn't have all fields
        assert!(!config.contains("kubeconfig:"));
    }

    #[test]
    fn test_generate_test_manifest() {
        let manifest = generate_test_manifest("my-app", "my-image:latest", 3);
        assert!(manifest.contains("name: my-app"));
        assert!(manifest.contains("image: my-image:latest"));
        assert!(manifest.contains("replicas: 3"));
    }

    #[test]
    fn test_generate_commit_hash() {
        let hash = generate_commit_hash();
        assert_eq!(hash.len(), 40);
        // Should only contain hex characters
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_project_name() {
        let name1 = generate_project_name();
        let name2 = generate_project_name();

        assert!(name1.starts_with("test-project-"));
        assert!(name2.starts_with("test-project-"));
        // Should generate unique names
        assert_ne!(name1, name2);
    }

    #[test]
    #[ignore = "requires git command"]
    fn test_create_temp_git_repo() {
        let (_temp_dir, repo_path) = create_temp_git_repo();

        assert!(repo_path.exists());
        assert!(repo_path.join(".git").exists());
        assert!(repo_path.join("README.md").exists());
    }
}
