//! Workspace Integration Test
//!
//! This test demonstrates how to use the helpers module and test fixtures
//! for integration testing across workspace crates.

use assert_cmd::prelude::*;
use std::process::Command;

// Include the helpers module
// In a real workspace, we'd import from multiple crates
mod helpers {
    include!("../helpers/mod.rs");
}

use helpers::{TestEnv, generate_test_config, assert_success_output, load_fixture};

/// Test configuration initialization with generated config
#[test]
fn test_config_init_with_generated_data() {
    let mut env = TestEnv::new();

    // Generate test configuration
    let config_content = generate_test_config();
    let config_path = env.create_config(&config_content);

    // Verify file was created and is valid
    assert!(config_path.exists());

    // Use the CLI to validate the config
    let output = Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("config")
        .arg("validate")
        .arg(&config_path)
        .output()
        .expect("Failed to validate config");

    assert_success_output(&output, "valid");
}

/// Test configuration loading from fixture
#[test]
fn test_config_show_with_fixture() {
    let fixture_content = load_fixture("sample_config.yaml");

    let mut env = TestEnv::new();
    let config_path = env.create_config(&fixture_content);

    // Show the config
    let output = Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("config")
        .arg("show")
        .arg("--file")
        .arg(&config_path)
        .output()
        .expect("Failed to show config");

    assert_success_output(&output, "dgx-spark-dev");
    assert_success_output(&output, "raibid-ci");
}

/// Test minimal configuration with fixture
#[test]
fn test_minimal_config_fixture() {
    let fixture_content = load_fixture("minimal_config.yaml");

    let mut env = TestEnv::new();
    let config_path = env.create_config(&fixture_content);

    // Validate minimal config
    let output = Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("config")
        .arg("validate")
        .arg(&config_path)
        .output()
        .expect("Failed to validate minimal config");

    assert_success_output(&output, "valid");
}

/// Test environment isolation
#[test]
fn test_environment_isolation() {
    // Create two separate test environments
    let mut env1 = TestEnv::new();
    let mut env2 = TestEnv::new();

    // Create different configs in each
    let config1 = env1.create_config("cluster:\n  name: env1");
    let config2 = env2.create_config("cluster:\n  name: env2");

    // Verify they're in different directories
    assert_ne!(env1.path(), env2.path());
    assert_ne!(config1, config2);

    // Verify each has its own content
    let content1 = std::fs::read_to_string(&config1).unwrap();
    let content2 = std::fs::read_to_string(&config2).unwrap();

    assert!(content1.contains("env1"));
    assert!(content2.contains("env2"));
    assert!(!content1.contains("env2"));
    assert!(!content2.contains("env1"));
}

/// Test subdirectory creation in test environment
#[test]
fn test_test_env_subdirectories() {
    let env = TestEnv::new();

    // Create nested directory structure
    let cache_dir = env.create_dir("cache");
    let builds_dir = env.create_dir("builds");
    let logs_dir = env.create_dir("logs");

    assert!(cache_dir.exists());
    assert!(builds_dir.exists());
    assert!(logs_dir.exists());

    // Verify they're all in the test environment
    assert!(cache_dir.starts_with(env.path()));
    assert!(builds_dir.starts_with(env.path()));
    assert!(logs_dir.starts_with(env.path()));
}

/// Test file creation in test environment
#[test]
fn test_test_env_file_creation() {
    let env = TestEnv::new();

    // Create test files
    let file1 = env.create_file("test1.txt", "content1");
    let file2 = env.create_file("test2.txt", "content2");

    assert!(file1.exists());
    assert!(file2.exists());

    // Verify contents
    let content1 = std::fs::read_to_string(&file1).unwrap();
    let content2 = std::fs::read_to_string(&file2).unwrap();

    assert_eq!(content1, "content1");
    assert_eq!(content2, "content2");
}

/// Test that uses mock builders (placeholder - would need actual implementation)
#[test]
#[ignore = "requires mock server implementation"]
fn test_with_mock_gitea() {
    use helpers::MockGitea;

    let mock_gitea = MockGitea::new()
        .with_url("http://localhost:3000")
        .with_admin("admin", "token123");

    // In a real test, we'd start a mock HTTP server here
    // and verify that the CLI can interact with it

    assert_eq!(mock_gitea.url, "http://localhost:3000");
    assert_eq!(mock_gitea.admin_user, "admin");
}

/// Test that uses mock Redis (placeholder)
#[test]
#[ignore = "requires mock Redis implementation"]
fn test_with_mock_redis() {
    use helpers::MockRedis;

    let mock_redis = MockRedis::new()
        .with_host("localhost")
        .with_port(6379);

    // In a real test, we'd start a mock Redis server here

    assert_eq!(mock_redis.connection_url(), "redis://localhost:6379");
}

/// Test YAML validation helper
#[test]
fn test_yaml_validation_helper() {
    use helpers::assertions::assert_valid_yaml;

    let valid_yaml = load_fixture("sample_config.yaml");
    assert_valid_yaml(&valid_yaml);

    let minimal_yaml = load_fixture("minimal_config.yaml");
    assert_valid_yaml(&minimal_yaml);
}

/// Test fixture loading
#[test]
fn test_fixture_loading() {
    // Load sample config
    let sample_config = load_fixture("sample_config.yaml");
    assert!(!sample_config.is_empty());
    assert!(sample_config.contains("cluster:"));

    // Load minimal config
    let minimal_config = load_fixture("minimal_config.yaml");
    assert!(!minimal_config.is_empty());
    assert!(minimal_config.contains("cluster:"));

    // Load Kubernetes manifest
    let manifest = load_fixture("rust_agent_deployment.yaml");
    assert!(!manifest.is_empty());
    assert!(manifest.contains("kind: Deployment"));
}

/// Test custom assertions
#[test]
fn test_custom_assertions() {
    use helpers::assertions::{assert_valid_yaml, assert_valid_json};

    // Test YAML validation
    let yaml = "key: value\nlist:\n  - item1\n  - item2";
    assert_valid_yaml(yaml);

    // Test JSON validation
    let json = r#"{"key": "value", "list": [1, 2, 3]}"#;
    assert_valid_json(json);
}

/// Test regex matching assertion
#[test]
fn test_regex_assertion() {
    use helpers::assertions::assert_matches_regex;

    let output = Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("--version")
        .output()
        .expect("Failed to get version");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify version format
    assert_matches_regex(&stdout, r"raibid-cli \d+\.\d+\.\d+");
}
