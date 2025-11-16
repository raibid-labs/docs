//! Tests for installation permission handling
//!
//! These tests verify that the installers handle permissions correctly,
//! default to user-local directories, and provide helpful error messages.

use raibid_cli::infrastructure::{K3sConfig, K3sInstaller, FluxConfig, FluxInstaller};
use std::fs;
use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;

/// Test that K3sConfig defaults to user-local directory
#[test]
fn test_k3s_config_defaults_to_user_local() {
    let config = K3sConfig::default();
    let home = dirs::home_dir().expect("Should have home directory");
    let expected = home.join(".local").join("bin");

    assert_eq!(
        config.install_dir,
        expected,
        "K3s should default to ~/.local/bin"
    );
}

/// Test that FluxConfig defaults to user-local directory
#[test]
fn test_flux_config_defaults_to_user_local() {
    let config = FluxConfig::default();
    let home = dirs::home_dir().expect("Should have home directory");
    let expected = home.join(".local").join("bin");

    assert_eq!(
        config.install_dir,
        expected,
        "Flux should default to ~/.local/bin"
    );
}

/// Test that user-local directory is created if it doesn't exist
#[test]
fn test_k3s_creates_user_local_directory() {
    let test_dir = std::env::temp_dir().join("raibid-test-k3s-install");

    // Clean up if exists
    let _ = fs::remove_dir_all(&test_dir);

    let config = K3sConfig {
        install_dir: test_dir.clone(),
        ..Default::default()
    };

    let installer = K3sInstaller::with_config(config).expect("Should create installer");

    // Create a dummy binary to install
    let test_binary = std::env::temp_dir().join("test-k3s-binary");
    fs::write(&test_binary, b"#!/bin/sh\necho test").expect("Should create test binary");

    // This should create the install directory
    let result = installer.install_binary(&test_binary);

    // Clean up
    let _ = fs::remove_file(&test_binary);
    let _ = fs::remove_dir_all(&test_dir);

    assert!(result.is_ok(), "Should create install directory if it doesn't exist");
}

/// Test permission check for directory
#[test]
fn test_directory_permission_check() {
    use raibid_cli::infrastructure::utils::check_directory_writable;

    // Test with temp directory (should be writable)
    let temp_dir = std::env::temp_dir();
    let result = check_directory_writable(&temp_dir);
    assert!(result.is_ok(), "Temp directory should be writable");

    // Test with a directory that doesn't exist yet
    let new_dir = temp_dir.join("raibid-test-new-dir");
    let _ = fs::remove_dir_all(&new_dir);

    // Should be able to create parent directory
    let result = check_directory_writable(&new_dir);
    assert!(result.is_ok(), "Should be able to create new directory");

    // Clean up
    let _ = fs::remove_dir_all(&new_dir);
}

/// Test that read-only directory is detected
#[test]
#[cfg(unix)]
fn test_readonly_directory_detected() {
    use raibid_cli::infrastructure::utils::check_directory_writable;

    let test_dir = std::env::temp_dir().join("raibid-test-readonly");

    // Create directory
    fs::create_dir_all(&test_dir).expect("Should create test directory");

    // Make it read-only
    let mut perms = fs::metadata(&test_dir).unwrap().permissions();
    perms.set_mode(0o444); // Read-only
    fs::set_permissions(&test_dir, perms).expect("Should set permissions");

    let result = check_directory_writable(&test_dir);

    // Restore permissions for cleanup
    let mut perms = fs::metadata(&test_dir).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&test_dir, perms).expect("Should restore permissions");

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);

    assert!(result.is_err(), "Should detect read-only directory");
}

/// Test PATH detection
#[test]
fn test_path_detection() {
    use raibid_cli::infrastructure::utils::is_directory_in_path;

    // Test with a directory that's definitely in PATH
    let path_env = std::env::var("PATH").expect("PATH should be set");
    let first_dir = path_env.split(':').next().expect("PATH should have at least one entry");

    let result = is_directory_in_path(&PathBuf::from(first_dir));
    assert!(result, "First PATH directory should be detected as in PATH");
}

/// Test PATH detection for directory not in PATH
#[test]
fn test_path_detection_not_in_path() {
    use raibid_cli::infrastructure::utils::is_directory_in_path;

    let random_dir = PathBuf::from("/tmp/definitely-not-in-path-123456789");
    let result = is_directory_in_path(&random_dir);
    assert!(!result, "Random directory should not be in PATH");
}

/// Test helpful error message for permission denied
#[test]
fn test_permission_denied_error_message() {
    use raibid_cli::infrastructure::utils::permission_denied_help;

    let system_dir = PathBuf::from("/usr/local/bin");
    let message = permission_denied_help(&system_dir);

    assert!(message.contains("Permission denied"), "Should mention permission denied");
    assert!(message.contains("sudo") || message.contains("different directory"),
            "Should suggest sudo or alternative directory");
}

/// Test custom install directory configuration
#[test]
fn test_k3s_custom_install_directory() {
    let custom_dir = PathBuf::from("/tmp/custom-k3s-bin");

    let config = K3sConfig {
        install_dir: custom_dir.clone(),
        ..Default::default()
    };

    assert_eq!(config.install_dir, custom_dir, "Should use custom install directory");
}

/// Test custom install directory configuration for Flux
#[test]
fn test_flux_custom_install_directory() {
    let custom_dir = PathBuf::from("/tmp/custom-flux-bin");

    let config = FluxConfig {
        install_dir: custom_dir.clone(),
        password: "test".to_string(),
        ..Default::default()
    };

    assert_eq!(config.install_dir, custom_dir, "Should use custom install directory");
}

/// Test that installer validates directory permissions before proceeding
#[test]
fn test_k3s_validates_permissions_before_install() {
    // This test ensures that permission check happens early
    let readonly_dir = PathBuf::from("/dev/null/cannot-write-here");

    let config = K3sConfig {
        install_dir: readonly_dir,
        ..Default::default()
    };

    let installer = K3sInstaller::with_config(config).expect("Should create installer");

    // Create a test binary
    let test_binary = std::env::temp_dir().join("test-k3s-binary-2");
    fs::write(&test_binary, b"#!/bin/sh\necho test").expect("Should create test binary");

    // This should fail with a helpful error message
    let result = installer.install_binary(&test_binary);

    // Clean up
    let _ = fs::remove_file(&test_binary);

    assert!(result.is_err(), "Should fail when directory is not writable");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("permission") || error_msg.contains("directory"),
        "Error message should mention permission or directory issue"
    );
}
