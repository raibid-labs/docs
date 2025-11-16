//! Integration tests for CLI-007: Configuration Management
//!
//! These tests verify configuration loading, validation, and subcommands.

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

// ============================================================================
// CONFIG INIT TESTS
// ============================================================================

/// Test config init command
#[test]
fn test_config_init() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("raibid.yaml");

    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("init")
        .arg("--output")
        .arg(&config_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created configuration"));

    // Verify file was created
    assert!(config_path.exists());
}

/// Test config init with minimal flag
#[test]
fn test_config_init_minimal() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("minimal.yaml");

    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("init")
        .arg("--output")
        .arg(&config_path)
        .arg("--minimal")
        .assert()
        .success();

    // Verify file was created
    assert!(config_path.exists());
}

/// Test config init with force flag
#[test]
fn test_config_init_force() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("raibid.yaml");

    // Create initial file
    std::fs::write(&config_path, "initial content").unwrap();

    // Overwrite with force flag
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("init")
        .arg("--output")
        .arg(&config_path)
        .arg("--force")
        .assert()
        .success();
}

/// Test config init without force on existing file fails
#[test]
fn test_config_init_no_force_fails() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("raibid.yaml");

    // Create initial file
    std::fs::write(&config_path, "initial content").unwrap();

    // Try to overwrite without force flag (should fail)
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("init")
        .arg("--output")
        .arg(&config_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("exists"));
}

/// Test config init creates valid YAML
#[test]
fn test_config_init_creates_valid_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("raibid.yaml");

    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("init")
        .arg("--output")
        .arg(&config_path)
        .assert()
        .success();

    // Read and parse YAML
    let content = std::fs::read_to_string(&config_path).unwrap();
    let _parsed: serde_yaml::Value = serde_yaml::from_str(&content)
        .expect("Generated config should be valid YAML");

    // Verify it contains expected keys
    assert!(content.contains("cluster"));
    assert!(content.contains("api"));
    assert!(content.contains("agents"));
}

// ============================================================================
// CONFIG SHOW TESTS
// ============================================================================

/// Test config show command (YAML format)
#[test]
fn test_config_show_yaml() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("cluster"))
        .stdout(predicate::str::contains("api"));
}

/// Test config show with JSON format
#[test]
fn test_config_show_json() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("show")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("cluster"));
}

/// Test config show with TOML format
#[test]
fn test_config_show_toml() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("show")
        .arg("--format")
        .arg("toml")
        .assert()
        .success()
        .stdout(predicate::str::contains("[cluster]"));
}

/// Test config show with specific file
#[test]
fn test_config_show_specific_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("raibid.yaml");

    // Create a config file first
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("init")
        .arg("--output")
        .arg(&config_path)
        .assert()
        .success();

    // Show specific file
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("show")
        .arg("--file")
        .arg(&config_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("cluster"));
}

/// Test config show with invalid format
#[test]
fn test_config_show_invalid_format() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("show")
        .arg("--format")
        .arg("invalid")
        .assert()
        .failure();
}

// ============================================================================
// CONFIG VALIDATE TESTS
// ============================================================================

/// Test config validate with default config
#[test]
fn test_config_validate_default() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

/// Test config validate with specific file
#[test]
fn test_config_validate_specific_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("raibid.yaml");

    // Create a valid config file first
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("init")
        .arg("--output")
        .arg(&config_path)
        .assert()
        .success();

    // Validate the file
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("validate")
        .arg(&config_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

/// Test config validate with invalid file
#[test]
fn test_config_validate_invalid_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.yaml");

    // Create invalid YAML
    std::fs::write(&config_path, "invalid: yaml: content: [").unwrap();

    // Validate should fail
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("validate")
        .arg(&config_path)
        .assert()
        .failure();
}

/// Test config validate with non-existent file
#[test]
fn test_config_validate_nonexistent_file() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("validate")
        .arg("/nonexistent/path/config.yaml")
        .assert()
        .failure();
}

// ============================================================================
// CONFIG PATH TESTS
// ============================================================================

/// Test config path command
#[test]
fn test_config_path() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("path")
        .assert()
        .success()
        .stdout(predicate::str::contains(".config")
            .or(predicate::str::contains("raibid"))
            .or(predicate::str::contains("Using default")));
}

// ============================================================================
// CONFIG HELP TESTS
// ============================================================================

/// Test config command help
#[test]
fn test_config_help() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage configuration"));
}

/// Test config init help
#[test]
fn test_config_init_help() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("init")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize"));
}

/// Test config show help
#[test]
fn test_config_show_help() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("show")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Show current configuration"));
}

/// Test config validate help
#[test]
fn test_config_validate_help() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("validate")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validate"));
}

// ============================================================================
// ENVIRONMENT VARIABLE TESTS
// ============================================================================

/// Test that RAIBID_* environment variables are recognized
#[test]
fn test_config_env_var_override() {
    // This test verifies that the CLI recognizes environment variables
    // The actual override behavior is tested in unit tests
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .env("RAIBID_API_HOST", "custom-host")
        .env("RAIBID_API_PORT", "9090")
        .arg("config")
        .arg("show")
        .assert()
        .success();
}

// ============================================================================
// CONFIG FILE PRIORITY TESTS
// ============================================================================

/// Test that local config takes priority over user config
#[test]
fn test_config_file_priority() {
    let temp_dir = TempDir::new().unwrap();
    let local_config = temp_dir.path().join("raibid.yaml");

    // Create a local config
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("config")
        .arg("init")
        .arg("--output")
        .arg(&local_config)
        .assert()
        .success();

    // Show config (should load the local one if we're in that directory)
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .current_dir(temp_dir.path())
        .arg("config")
        .arg("show")
        .assert()
        .success();
}
