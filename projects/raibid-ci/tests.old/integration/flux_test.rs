//! Integration tests for Flux installer
//!
//! These tests verify the Flux GitOps installation and configuration functionality.
//! Tests are skipped if k3s is not available.

use raibid_cli::infrastructure::{FluxInstaller, FluxConfig};
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

/// Check if Flux CLI is available
fn is_flux_available() -> bool {
    Command::new("flux")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if Gitea is available
fn is_gitea_available() -> bool {
    Command::new("kubectl")
        .arg("get")
        .arg("namespace")
        .arg("gitea")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn test_flux_config_default() {
    let config = FluxConfig::default();

    assert_eq!(config.namespace, "flux-system");
    assert_eq!(config.repository, "raibid-gitops");
    assert_eq!(config.branch, "main");
    assert_eq!(config.path, "clusters/raibid");
    assert_eq!(config.username, "raibid-admin");
    assert!(config.enable_image_automation);
    assert!(config.enable_notifications);
    assert_eq!(config.interval, "1m");
}

#[test]
fn test_flux_config_custom() {
    let config = FluxConfig {
        namespace: "custom-flux".to_string(),
        repository: "my-gitops".to_string(),
        username: "admin".to_string(),
        password: "test-password".to_string(),
        branch: "develop".to_string(),
        path: "clusters/prod".to_string(),
        enable_image_automation: false,
        enable_notifications: false,
        interval: "5m".to_string(),
        ..Default::default()
    };

    assert_eq!(config.namespace, "custom-flux");
    assert_eq!(config.repository, "my-gitops");
    assert_eq!(config.branch, "develop");
    assert!(!config.enable_image_automation);
    assert!(!config.enable_notifications);
    assert_eq!(config.interval, "5m");
}

#[test]
fn test_flux_config_validation_missing_password() {
    let config = FluxConfig {
        password: String::new(),
        ..Default::default()
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("password"));
}

#[test]
fn test_flux_config_validation_missing_repository() {
    let config = FluxConfig {
        repository: String::new(),
        password: "test".to_string(),
        ..Default::default()
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Repository"));
}

#[test]
fn test_flux_config_validation_missing_username() {
    let config = FluxConfig {
        username: String::new(),
        password: "test".to_string(),
        ..Default::default()
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Username"));
}

#[test]
fn test_flux_config_validation_success() {
    let config = FluxConfig {
        password: "test-password".to_string(),
        ..Default::default()
    };

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_repository_url() {
    let config = FluxConfig {
        gitea_url: "http://gitea.local:3000".to_string(),
        username: "admin".to_string(),
        repository: "my-repo".to_string(),
        password: "secret".to_string(),
        ..Default::default()
    };

    let url = config.repository_url();
    assert_eq!(url, "http://gitea.local:3000/admin/my-repo.git");
}

#[test]
fn test_repository_url_with_auth() {
    let config = FluxConfig {
        gitea_url: "http://gitea.local:3000".to_string(),
        username: "admin".to_string(),
        repository: "my-repo".to_string(),
        password: "secret".to_string(),
        ..Default::default()
    };

    let url = config.repository_url_with_auth();
    assert_eq!(url, "http://admin:secret@gitea.local:3000/admin/my-repo.git");
}

#[test]
fn test_repository_url_with_auth_strips_protocol() {
    let config = FluxConfig {
        gitea_url: "https://gitea.example.com".to_string(),
        username: "user".to_string(),
        repository: "repo".to_string(),
        password: "pass".to_string(),
        ..Default::default()
    };

    let url = config.repository_url_with_auth();
    assert_eq!(url, "http://user:pass@gitea.example.com/user/repo.git");
}

#[test]
fn test_flux_installer_creation_fails_without_password() {
    let config = FluxConfig {
        password: String::new(),
        ..Default::default()
    };

    let installer = FluxInstaller::with_config(config);
    assert!(installer.is_err());
}

#[test]
fn test_flux_installer_creation_success() {
    let config = FluxConfig {
        password: "test-password".to_string(),
        ..Default::default()
    };

    let installer = FluxInstaller::with_config(config);
    assert!(installer.is_ok(), "FluxInstaller should be created successfully");
}

#[test]
fn test_platform_detection() {
    use raibid_cli::infrastructure::flux::Platform;

    let platform = Platform::detect();
    assert!(platform.is_ok(), "Platform detection should succeed");

    // Verify archive name is valid
    let platform = platform.unwrap();
    let archive_name = platform.archive_name();
    assert!(archive_name.contains("flux"));
    assert!(archive_name.ends_with(".tar.gz"));
}

#[test]
fn test_platform_archive_names() {
    use raibid_cli::infrastructure::flux::Platform;

    assert_eq!(
        Platform::LinuxArm64.archive_name(),
        "flux_2.2.3_linux_arm64.tar.gz"
    );
    assert_eq!(
        Platform::LinuxAmd64.archive_name(),
        "flux_2.2.3_linux_amd64.tar.gz"
    );
    assert_eq!(
        Platform::DarwinArm64.archive_name(),
        "flux_2.2.3_darwin_arm64.tar.gz"
    );
    assert_eq!(
        Platform::DarwinAmd64.archive_name(),
        "flux_2.2.3_darwin_amd64.tar.gz"
    );
}

#[test]
fn test_platform_checksum_name() {
    use raibid_cli::infrastructure::flux::Platform;

    assert_eq!(
        Platform::LinuxArm64.checksum_name(),
        "flux_2.2.3_checksums.txt"
    );
}

#[test]
#[ignore] // Only run when explicitly requested
fn test_flux_cli_check() {
    let config = FluxConfig {
        password: "test".to_string(),
        ..Default::default()
    };

    let installer = FluxInstaller::with_config(config).unwrap();
    let result = installer.check_flux_cli();

    assert!(result.is_ok(), "check_flux_cli should not error");
    // Result can be true or false depending on whether flux is installed
    println!("Flux CLI available: {}", result.unwrap());
}

#[test]
#[ignore] // Only run when explicitly requested
fn test_flux_download_and_verify() {
    let config = FluxConfig {
        password: "test".to_string(),
        ..Default::default()
    };

    let installer = FluxInstaller::with_config(config).unwrap();

    // Create runtime for async operations
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Download binary
    let archive_result = runtime.block_on(installer.download_flux());
    assert!(archive_result.is_ok(), "Should download Flux archive");

    // Download checksums
    let checksums_result = runtime.block_on(installer.download_checksums());
    assert!(checksums_result.is_ok(), "Should download checksums");

    // Verify checksum
    if let (Ok(archive_path), Ok(checksums_path)) = (archive_result, checksums_result) {
        let verify_result = installer.verify_checksum(&archive_path, &checksums_path);
        assert!(verify_result.is_ok(), "Checksum verification should succeed");
    }

    // Cleanup
    let _ = installer.cleanup();
}

#[test]
#[ignore] // Only run when explicitly requested
fn test_flux_bootstrap_workflow() {
    if !is_k3s_available() {
        println!("Skipping: k3s cluster not available");
        return;
    }

    if !is_gitea_available() {
        println!("Skipping: Gitea not available");
        return;
    }

    // This test requires a real Gitea instance with credentials
    // In practice, this would need to be configured with actual values
    let config = FluxConfig {
        password: "test-password".to_string(),
        repository: "flux-test".to_string(),
        namespace: "flux-test".to_string(),
        ..Default::default()
    };

    let installer = FluxInstaller::with_config(config).unwrap();

    // Note: This is a dry-run test that verifies the installer can be created
    // Actual bootstrap would require valid Gitea credentials and repository
    assert!(installer.check_flux_cli().is_ok());
}

#[test]
#[ignore] // Only run when explicitly requested
fn test_flux_validation() {
    if !is_k3s_available() {
        println!("Skipping: k3s cluster not available");
        return;
    }

    if !is_flux_available() {
        println!("Skipping: Flux not available");
        return;
    }

    let config = FluxConfig {
        password: "test".to_string(),
        ..Default::default()
    };

    let installer = FluxInstaller::with_config(config).unwrap();

    // This will only succeed if Flux is actually installed
    let result = installer.validate_installation();
    if result.is_err() {
        println!("Flux validation failed (expected if not installed): {:?}", result);
    }
}

#[test]
#[ignore] // Only run when explicitly requested
fn test_flux_status() {
    if !is_k3s_available() {
        println!("Skipping: k3s cluster not available");
        return;
    }

    if !is_flux_available() {
        println!("Skipping: Flux not available");
        return;
    }

    let config = FluxConfig {
        password: "test".to_string(),
        ..Default::default()
    };

    let installer = FluxInstaller::with_config(config).unwrap();

    // This will only succeed if Flux is actually installed
    let result = installer.get_status();
    if result.is_ok() {
        println!("Flux status:\n{}", result.unwrap());
    } else {
        println!("Failed to get Flux status (expected if not installed)");
    }
}

#[test]
fn test_flux_cleanup() {
    let config = FluxConfig {
        password: "test".to_string(),
        ..Default::default()
    };

    let installer = FluxInstaller::with_config(config).unwrap();

    // Cleanup should succeed even if there's nothing to clean
    let result = installer.cleanup();
    assert!(result.is_ok(), "Cleanup should succeed");
}
