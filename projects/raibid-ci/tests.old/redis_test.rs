//! Integration tests for Redis installer
//!
//! These tests verify the Redis deployment and configuration functionality.
//! Tests are skipped if k3s is not available.

use raibid_cli::infrastructure::{RedisInstaller, RedisConfig, RedisStreamsConfig};
use std::path::PathBuf;
use std::process::Command;

/// Check if k3s cluster is available
fn is_k3s_available() -> bool {
    Command::new("kubectl")
        .arg("cluster-info")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if Helm is available
fn is_helm_available() -> bool {
    Command::new("helm")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn test_redis_config_default() {
    let config = RedisConfig::default();

    assert_eq!(config.namespace, "raibid-redis");
    assert_eq!(config.release_name, "raibid-redis");
    assert!(config.persistence_enabled);
    assert_eq!(config.persistence_size, "8Gi");
    assert!(config.auth_enabled);
    assert_eq!(config.replica_count, 0);
    assert!(!config.sentinel_enabled);
}

#[test]
fn test_redis_streams_config_default() {
    let config = RedisStreamsConfig::default();

    assert_eq!(config.queue_stream, "raibid:jobs");
    assert_eq!(config.consumer_group, "raibid-workers");
    assert_eq!(config.max_length, 10000);
}

#[test]
fn test_redis_config_custom() {
    let streams_config = RedisStreamsConfig {
        queue_stream: "test:queue".to_string(),
        consumer_group: "test-group".to_string(),
        max_length: 5000,
    };

    let config = RedisConfig {
        namespace: "test-redis".to_string(),
        release_name: "test-release".to_string(),
        persistence_enabled: false,
        auth_enabled: false,
        streams_config,
        ..Default::default()
    };

    assert_eq!(config.namespace, "test-redis");
    assert!(!config.persistence_enabled);
    assert!(!config.auth_enabled);
    assert_eq!(config.streams_config.queue_stream, "test:queue");
}

#[test]
fn test_redis_installer_creation() {
    let installer = RedisInstaller::new();
    assert!(installer.is_ok(), "RedisInstaller should be created successfully");
}

#[test]
fn test_redis_installer_with_custom_config() {
    let config = RedisConfig {
        namespace: "custom-redis".to_string(),
        ..Default::default()
    };

    let installer = RedisInstaller::with_config(config);
    assert!(installer.is_ok(), "RedisInstaller should accept custom config");
}

#[test]
fn test_helm_values_generation() {
    let mut installer = RedisInstaller::new().unwrap();
    let values = installer.generate_helm_values();

    assert!(values.is_ok(), "Helm values should be generated successfully");

    let values_str = values.unwrap();
    assert!(values_str.contains("auth:"), "Should contain auth config");
    assert!(values_str.contains("persistence:"), "Should contain persistence config");
    assert!(values_str.contains("enabled: true"), "Should enable features");
    assert!(values_str.contains("appendonly yes"), "Should configure AOF persistence");
}

#[test]
fn test_helm_values_with_auth_disabled() {
    let config = RedisConfig {
        auth_enabled: false,
        ..Default::default()
    };

    let mut installer = RedisInstaller::with_config(config).unwrap();
    let values = installer.generate_helm_values().unwrap();

    assert!(values.contains("enabled: false"), "Auth should be disabled");
}

#[test]
fn test_password_generation() {
    let mut installer = RedisInstaller::new().unwrap();

    let password1 = installer.get_or_generate_password();
    assert_eq!(password1.len(), 32, "Password should be 32 characters");

    // Verify password is alphanumeric
    assert!(password1.chars().all(|c| c.is_alphanumeric()),
        "Password should be alphanumeric");

    // Verify password is cached
    let password2 = installer.get_or_generate_password();
    assert_eq!(password1, password2, "Password should be cached");
}

#[test]
fn test_password_provided() {
    let config = RedisConfig {
        password: Some("my-secret-password".to_string()),
        ..Default::default()
    };

    let mut installer = RedisInstaller::with_config(config).unwrap();
    let password = installer.get_or_generate_password();

    assert_eq!(password, "my-secret-password", "Should use provided password");
}

#[test]
fn test_connection_info_generation() {
    let installer = RedisInstaller::new().unwrap();
    let conn_info = installer.get_connection_info();

    assert!(conn_info.is_ok(), "Connection info should be generated");

    let info = conn_info.unwrap();
    assert!(info.host.contains("raibid-redis-master"), "Host should contain release name");
    assert!(info.host.contains("raibid-redis.svc.cluster.local"), "Host should be cluster DNS");
    assert_eq!(info.port, 6379, "Port should be 6379");
    assert_eq!(info.namespace, "raibid-redis", "Namespace should be raibid-redis");
}

#[test]
fn test_connection_url_with_auth() {
    let info = raibid_cli::infrastructure::RedisConnectionInfo {
        host: "redis-master.namespace.svc.cluster.local".to_string(),
        port: 6379,
        password: Some("secretpass".to_string()),
        namespace: "namespace".to_string(),
    };

    let url = info.connection_url();
    assert_eq!(url, "redis://:secretpass@redis-master.namespace.svc.cluster.local:6379");
}

#[test]
fn test_connection_url_without_auth() {
    let info = raibid_cli::infrastructure::RedisConnectionInfo {
        host: "redis-master.namespace.svc.cluster.local".to_string(),
        port: 6379,
        password: None,
        namespace: "namespace".to_string(),
    };

    let url = info.connection_url();
    assert_eq!(url, "redis://redis-master.namespace.svc.cluster.local:6379");
}

#[test]
#[ignore] // Only run when explicitly requested
fn test_redis_installation_workflow() {
    if !is_k3s_available() {
        println!("Skipping: k3s cluster not available");
        return;
    }

    if !is_helm_available() {
        println!("Skipping: helm not available");
        return;
    }

    // Use test namespace to avoid conflicts
    let config = RedisConfig {
        namespace: "raibid-redis-test".to_string(),
        release_name: "raibid-redis-test".to_string(),
        persistence_size: "1Gi".to_string(), // Smaller for testing
        ..Default::default()
    };

    let mut installer = RedisInstaller::with_config(config).unwrap();

    // Test installation
    let result = installer.install();

    // Cleanup
    let _ = installer.uninstall();

    assert!(result.is_ok(), "Installation should succeed: {:?}", result.err());
}

#[test]
#[ignore] // Only run when explicitly requested
fn test_helm_repo_operations() {
    if !is_helm_available() {
        println!("Skipping: helm not available");
        return;
    }

    let installer = RedisInstaller::new().unwrap();

    // Test adding Helm repo
    let result = installer.add_helm_repo();
    assert!(result.is_ok(), "Should add Helm repo successfully: {:?}", result.err());
}

#[test]
#[ignore] // Only run when explicitly requested
fn test_namespace_creation() {
    if !is_k3s_available() {
        println!("Skipping: k3s cluster not available");
        return;
    }

    let config = RedisConfig {
        namespace: "raibid-redis-test-ns".to_string(),
        ..Default::default()
    };

    let installer = RedisInstaller::with_config(config).unwrap();

    // Test namespace creation
    let result = installer.create_namespace();
    assert!(result.is_ok(), "Should create namespace successfully: {:?}", result.err());

    // Cleanup
    let _ = Command::new("kubectl")
        .arg("delete")
        .arg("namespace")
        .arg("raibid-redis-test-ns")
        .output();
}

#[test]
fn test_credential_save() {
    use tempfile::NamedTempFile;

    let installer = RedisInstaller::new().unwrap();
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    let result = installer.save_credentials(temp_path);
    assert!(result.is_ok(), "Should save credentials successfully");

    // Verify file was created and contains JSON
    let content = std::fs::read_to_string(temp_path).unwrap();
    assert!(content.contains("host"), "Should contain host field");
    assert!(content.contains("port"), "Should contain port field");
    assert!(content.contains("raibid:jobs"), "Should contain stream name");
    assert!(content.contains("raibid-workers"), "Should contain consumer group");

    // Verify it's valid JSON
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(json["host"].is_string());
    assert!(json["port"].is_number());
}

#[cfg(unix)]
#[test]
fn test_credential_file_permissions() {
    use std::os::unix::fs::PermissionsExt;
    use tempfile::NamedTempFile;

    let installer = RedisInstaller::new().unwrap();
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path();

    installer.save_credentials(temp_path).unwrap();

    // Check file permissions (should be 600)
    let metadata = std::fs::metadata(temp_path).unwrap();
    let permissions = metadata.permissions();
    let mode = permissions.mode();

    // 0o600 = owner read/write only
    assert_eq!(mode & 0o777, 0o600, "File permissions should be 600");
}
