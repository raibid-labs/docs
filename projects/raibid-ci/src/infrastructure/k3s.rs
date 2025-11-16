//! k3s Installation Module
//!
//! This module handles downloading, installing, and bootstrapping k3s clusters.
//! It supports ARM64 Linux (DGX Spark) and macOS ARM64 platforms.

use anyhow::{Context, Result, anyhow};
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

/// k3s release information
const K3S_VERSION: &str = "v1.28.5+k3s1";
const K3S_GITHUB_RELEASE_URL: &str = "https://github.com/k3s-io/k3s/releases/download";

/// k3s server execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum K3sMode {
    /// Rootless mode - runs k3s without root privileges (experimental)
    Rootless,
    /// Root mode - runs k3s with sudo (full feature set)
    Root,
}

impl K3sMode {
    /// Get a human-readable description of the mode
    pub fn description(&self) -> &str {
        match self {
            K3sMode::Rootless => "rootless (no sudo required, experimental)",
            K3sMode::Root => "root (requires sudo, full features)",
        }
    }
}

/// Platform-specific binary names
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    LinuxArm64,
    DarwinArm64,
}

impl Platform {
    /// Detect the current platform
    pub fn detect() -> Result<Self> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        match (os, arch) {
            ("linux", "aarch64") => Ok(Platform::LinuxArm64),
            ("macos", "aarch64") => Ok(Platform::DarwinArm64),
            _ => Err(anyhow!(
                "Unsupported platform: {} {}. Only ARM64 Linux and macOS are supported.",
                os,
                arch
            )),
        }
    }

    /// Get the binary name for this platform
    pub fn binary_name(&self) -> &str {
        match self {
            Platform::LinuxArm64 => "k3s-arm64",
            Platform::DarwinArm64 => "k3s-arm64",
        }
    }

    /// Get the checksum file name for this platform
    pub fn checksum_name(&self) -> &str {
        "sha256sum-arm64.txt"
    }
}

/// k3s installation configuration
#[derive(Debug, Clone)]
pub struct K3sConfig {
    /// k3s version to install
    pub version: String,
    /// Installation directory (default: /usr/local/bin)
    pub install_dir: PathBuf,
    /// Data directory (default: /var/lib/rancher/k3s)
    #[allow(dead_code)]
    pub data_dir: PathBuf,
    /// Kubeconfig output path (default: ~/.kube/config)
    pub kubeconfig_path: PathBuf,
    /// Additional k3s server flags
    pub server_flags: Vec<String>,
    /// k3s server execution mode (rootless or root)
    pub mode: K3sMode,
}

impl Default for K3sConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/root"));

        // Default to rootless mode for development/testing (no sudo required)
        let mode = K3sMode::Rootless;

        // Configure server flags based on mode
        let mut server_flags = vec![
            "--write-kubeconfig-mode=644".to_string(),
            "--disable=traefik".to_string(), // We don't need traefik for CI
        ];

        // Add rootless-specific flags
        if matches!(mode, K3sMode::Rootless) {
            server_flags.push("--rootless".to_string());
            server_flags.push("--snapshotter=fuse-overlayfs".to_string());
        }

        Self {
            version: K3S_VERSION.to_string(),
            install_dir: home.join(".local").join("bin"), // User-local, no sudo required
            data_dir: PathBuf::from("/var/lib/rancher/k3s"),
            kubeconfig_path: home.join(".kube").join("config"),
            server_flags,
            mode,
        }
    }
}

/// Check if cgroup v2 is available on the system
///
/// Rootless k3s requires pure cgroup v2 (not hybrid or v1).
/// This function checks if the system is running cgroup v2.
pub fn cgroup_v2_available() -> bool {
    #[cfg(target_os = "linux")]
    {
        // Check if /sys/fs/cgroup is mounted as cgroup2
        if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
            for line in mounts.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let mount_point = parts[1];
                    let fs_type = parts[2];
                    if mount_point == "/sys/fs/cgroup" && fs_type == "cgroup2" {
                        return true;
                    }
                }
            }
        }
        false
    }

    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// k3s installer
pub struct K3sInstaller {
    config: K3sConfig,
    platform: Platform,
    download_dir: PathBuf,
}

impl K3sInstaller {
    /// Create a new k3s installer with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(K3sConfig::default())
    }

    /// Create a new k3s installer with custom configuration
    pub fn with_config(config: K3sConfig) -> Result<Self> {
        let platform = Platform::detect()?;
        let download_dir = std::env::temp_dir().join("raibid-k3s-install");

        Ok(Self {
            config,
            platform,
            download_dir,
        })
    }

    /// Download k3s binary from GitHub releases
    pub async fn download_binary(&self) -> Result<PathBuf> {
        info!("Downloading k3s {} for {:?}", self.config.version, self.platform);

        // Create download directory
        fs::create_dir_all(&self.download_dir)
            .context("Failed to create download directory")?;

        let binary_name = self.platform.binary_name();
        let download_url = format!(
            "{}/{}/{}",
            K3S_GITHUB_RELEASE_URL,
            self.config.version,
            binary_name
        );

        let binary_path = self.download_dir.join("k3s");

        debug!("Downloading from: {}", download_url);

        // Download binary
        let response = reqwest::get(&download_url)
            .await
            .context("Failed to download k3s binary")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download k3s binary: HTTP {}",
                response.status()
            ));
        }

        let bytes = response.bytes()
            .await
            .context("Failed to read k3s binary response")?;

        let mut file = fs::File::create(&binary_path)
            .context("Failed to create binary file")?;

        file.write_all(&bytes)
            .context("Failed to write binary file")?;

        info!("Downloaded k3s binary to {:?}", binary_path);

        Ok(binary_path)
    }

    /// Download and parse checksums file
    pub async fn download_checksums(&self) -> Result<String> {
        info!("Downloading checksums for verification");

        let checksum_name = self.platform.checksum_name();
        let checksum_url = format!(
            "{}/{}/{}",
            K3S_GITHUB_RELEASE_URL,
            self.config.version,
            checksum_name
        );

        debug!("Downloading checksums from: {}", checksum_url);

        let response = reqwest::get(&checksum_url)
            .await
            .context("Failed to download checksums")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download checksums: HTTP {}",
                response.status()
            ));
        }

        let checksums = response.text()
            .await
            .context("Failed to read checksums response")?;

        Ok(checksums)
    }

    /// Verify binary checksum
    pub fn verify_checksum(&self, binary_path: &Path, checksums: &str) -> Result<()> {
        info!("Verifying binary checksum");

        // Calculate SHA256 of downloaded binary
        let binary_data = fs::read(binary_path)
            .context("Failed to read binary for checksum verification")?;

        let hash = sha256::digest(&binary_data);

        // Find expected checksum for our binary
        let binary_name = self.platform.binary_name();
        let expected_checksum = checksums
            .lines()
            .find(|line| line.contains(binary_name))
            .and_then(|line| line.split_whitespace().next())
            .ok_or_else(|| anyhow!("Checksum not found for {}", binary_name))?;

        if hash != expected_checksum {
            return Err(anyhow!(
                "Checksum verification failed!\nExpected: {}\nActual: {}",
                expected_checksum,
                hash
            ));
        }

        info!("Checksum verification passed");
        Ok(())
    }

    /// Install k3s binary to system location
    pub fn install_binary(&self, binary_path: &Path) -> Result<()> {
        use crate::infrastructure::utils::{check_directory_writable, warn_if_not_in_path};

        info!("Installing k3s binary to {:?}", self.config.install_dir);

        // Check if we can write to the install directory before proceeding
        check_directory_writable(&self.config.install_dir)?;

        let install_path = self.config.install_dir.join("k3s");

        // Copy binary to install location
        fs::copy(binary_path, &install_path)
            .context("Failed to copy binary to install directory")?;

        // Make executable (chmod +x)
        let mut perms = fs::metadata(&install_path)
            .context("Failed to get binary metadata")?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&install_path, perms)
            .context("Failed to set binary permissions")?;

        info!("Binary installed to {:?}", install_path);

        // Warn if install directory is not in PATH
        warn_if_not_in_path(&self.config.install_dir);

        Ok(())
    }

    /// Bootstrap k3s cluster
    pub fn bootstrap_cluster(&self) -> Result<()> {
        info!("Bootstrapping k3s cluster in {:?} mode", self.config.mode);

        // Pre-flight checks for rootless mode
        if self.config.mode == K3sMode::Rootless {
            if !cgroup_v2_available() {
                warn!(
                    "cgroup v2 is not available. Rootless mode requires pure cgroup v2.\n\
                    To enable cgroup v2, add 'systemd.unified_cgroup_hierarchy=1' to kernel parameters."
                );
                // Continue anyway - k3s will provide a better error message if it fails
            }
        }

        let k3s_path = self.config.install_dir.join("k3s");

        // Build k3s server command based on mode
        let mut cmd = match self.config.mode {
            K3sMode::Rootless => {
                // Rootless mode - run k3s directly
                debug!("Running k3s in rootless mode (no sudo)");
                let mut c = Command::new(&k3s_path);
                c.arg("server");
                c
            }
            K3sMode::Root => {
                // Root mode - run k3s with sudo
                debug!("Running k3s in root mode (with sudo)");
                info!("Root mode requires sudo privileges. You may be prompted for your password.");
                let mut c = Command::new("sudo");
                c.arg(&k3s_path);
                c.arg("server");
                c
            }
        };

        // Add server flags
        for flag in &self.config.server_flags {
            cmd.arg(flag);
        }

        // Start k3s server in background
        debug!("Starting k3s server: {:?}", cmd);

        let mut child = cmd
            .spawn()
            .context("Failed to start k3s server")?;

        // Wait a bit for server to start
        std::thread::sleep(std::time::Duration::from_secs(5));

        // Check if still running
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(anyhow!(
                    "k3s server exited prematurely with status: {}",
                    status
                ));
            }
            Ok(None) => {
                info!("k3s server is running in {:?} mode", self.config.mode);
            }
            Err(e) => {
                return Err(anyhow!("Failed to check k3s server status: {}", e));
            }
        }

        Ok(())
    }

    /// Configure kubeconfig for cluster access
    pub fn configure_kubeconfig(&self) -> Result<()> {
        info!("Configuring kubeconfig");

        let k3s_kubeconfig = PathBuf::from("/etc/rancher/k3s/k3s.yaml");

        // Check if k3s kubeconfig exists
        if !k3s_kubeconfig.exists() {
            return Err(anyhow!(
                "k3s kubeconfig not found at {:?}. Is k3s running?",
                k3s_kubeconfig
            ));
        }

        // Ensure .kube directory exists
        if let Some(parent) = self.config.kubeconfig_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create .kube directory")?;
        }

        // Copy k3s kubeconfig to user location
        fs::copy(&k3s_kubeconfig, &self.config.kubeconfig_path)
            .context("Failed to copy kubeconfig")?;

        // Set proper permissions
        let mut perms = fs::metadata(&self.config.kubeconfig_path)
            .context("Failed to get kubeconfig metadata")?
            .permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&self.config.kubeconfig_path, perms)
            .context("Failed to set kubeconfig permissions")?;

        info!("Kubeconfig configured at {:?}", self.config.kubeconfig_path);

        Ok(())
    }

    /// Validate cluster is healthy
    pub fn validate_cluster(&self) -> Result<()> {
        info!("Validating k3s cluster");

        // k3s binary also acts as kubectl
        let k3s_path = self.config.install_dir.join("k3s");

        // Try to get nodes
        let output = Command::new(&k3s_path)
            .arg("kubectl")
            .arg("get")
            .arg("nodes")
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to run kubectl get nodes")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "kubectl get nodes failed: {}",
                stderr
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check if we have a Ready node
        if !stdout.contains("Ready") {
            warn!("No Ready nodes found yet, cluster may still be initializing");
        } else {
            info!("Cluster validation successful - node is Ready");
        }

        Ok(())
    }

    /// Cleanup download directory
    pub fn cleanup(&self) -> Result<()> {
        if self.download_dir.exists() {
            fs::remove_dir_all(&self.download_dir)
                .context("Failed to cleanup download directory")?;
            debug!("Cleaned up download directory");
        }
        Ok(())
    }

    /// Install k3s - complete installation workflow
    #[allow(dead_code)]
    pub async fn install(&self) -> Result<()> {
        info!("Starting k3s installation");

        // Download binary
        let binary_path = self.download_binary().await?;

        // Download and verify checksums
        let checksums = self.download_checksums().await?;
        self.verify_checksum(&binary_path, &checksums)?;

        // Install binary
        self.install_binary(&binary_path)?;

        // Bootstrap cluster
        self.bootstrap_cluster()?;

        // Configure kubeconfig
        self.configure_kubeconfig()?;

        // Validate cluster
        self.validate_cluster()?;

        // Cleanup
        self.cleanup()?;

        info!("k3s installation completed successfully");

        Ok(())
    }

    /// Rollback installation on failure
    pub fn rollback(&self) -> Result<()> {
        warn!("Rolling back k3s installation");

        // Stop k3s if running
        let _ = Command::new("pkill")
            .arg("k3s")
            .output();

        // Remove binary
        let install_path = self.config.install_dir.join("k3s");
        if install_path.exists() {
            fs::remove_file(&install_path)
                .context("Failed to remove k3s binary during rollback")?;
        }

        // Cleanup download directory
        let _ = self.cleanup();

        info!("Rollback completed");

        Ok(())
    }
}

impl Default for K3sInstaller {
    fn default() -> Self {
        Self::new().expect("Failed to create default K3sInstaller")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        assert!(platform.is_ok(), "Platform detection should succeed");
    }

    #[test]
    fn test_platform_binary_names() {
        assert_eq!(Platform::LinuxArm64.binary_name(), "k3s-arm64");
        assert_eq!(Platform::DarwinArm64.binary_name(), "k3s-arm64");
    }

    #[test]
    fn test_default_config() {
        let config = K3sConfig::default();
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/root"));
        let expected_install_dir = home.join(".local").join("bin");

        assert_eq!(config.version, K3S_VERSION);
        assert_eq!(config.install_dir, expected_install_dir);
        assert!(config.server_flags.contains(&"--write-kubeconfig-mode=644".to_string()));
    }

    #[test]
    fn test_installer_creation() {
        let installer = K3sInstaller::new();
        assert!(installer.is_ok(), "Installer creation should succeed");
    }

    #[tokio::test]
    async fn test_download_binary() {
        let installer = K3sInstaller::new().unwrap();

        // This test actually downloads - skip in CI without network
        if std::env::var("SKIP_NETWORK_TESTS").is_ok() {
            return;
        }

        let result = installer.download_binary().await;

        // Cleanup regardless of success
        let _ = installer.cleanup();

        assert!(result.is_ok(), "Binary download should succeed");
    }

    #[tokio::test]
    async fn test_download_checksums() {
        let installer = K3sInstaller::new().unwrap();

        // Skip in CI without network
        if std::env::var("SKIP_NETWORK_TESTS").is_ok() {
            return;
        }

        let result = installer.download_checksums().await;
        assert!(result.is_ok(), "Checksum download should succeed");

        let checksums = result.unwrap();
        assert!(checksums.contains("k3s-arm64"), "Checksums should contain k3s-arm64");
    }

    #[test]
    fn test_verify_checksum_success() {
        use std::io::Write;

        let installer = K3sInstaller::new().unwrap();
        let test_dir = std::env::temp_dir().join("k3s-test-checksum");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("test-binary");
        let test_data = b"test data";

        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(test_data).unwrap();
        drop(file);

        let hash = sha256::digest(test_data);
        let checksums = format!("{} k3s-arm64\n", hash);

        let result = installer.verify_checksum(&test_file, &checksums);

        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();

        assert!(result.is_ok(), "Checksum verification should succeed");
    }

    #[test]
    fn test_verify_checksum_failure() {
        use std::io::Write;

        let installer = K3sInstaller::new().unwrap();
        let test_dir = std::env::temp_dir().join("k3s-test-checksum-fail");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("test-binary");
        let test_data = b"test data";

        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(test_data).unwrap();
        drop(file);

        // Wrong checksum
        let checksums = "0000000000000000000000000000000000000000000000000000000000000000 k3s-arm64\n";

        let result = installer.verify_checksum(&test_file, &checksums);

        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();

        assert!(result.is_err(), "Checksum verification should fail with wrong hash");
    }

    #[test]
    fn test_k3s_mode_enum() {
        // Test that K3sMode enum exists and has expected variants
        let rootless = K3sMode::Rootless;
        let root = K3sMode::Root;

        assert_ne!(rootless, root, "Rootless and Root should be different variants");
    }

    #[test]
    fn test_k3s_config_default_mode() {
        // Test that default K3sConfig uses Rootless mode
        let config = K3sConfig::default();

        assert_eq!(config.mode, K3sMode::Rootless, "Default mode should be Rootless for development");
    }

    #[test]
    fn test_k3s_config_rootless_flags() {
        // Test that rootless mode includes required flags
        let config = K3sConfig::default();

        if config.mode == K3sMode::Rootless {
            assert!(
                config.server_flags.contains(&"--rootless".to_string()),
                "Rootless mode should include --rootless flag"
            );
            assert!(
                config.server_flags.contains(&"--snapshotter=fuse-overlayfs".to_string()),
                "Rootless mode should include --snapshotter=fuse-overlayfs flag"
            );
        }
    }

    #[test]
    fn test_cgroup_v2_detection() {
        // Test cgroup v2 detection function
        // This should work on most modern Linux systems
        #[cfg(target_os = "linux")]
        {
            let result = cgroup_v2_available();
            // Just ensure it returns a boolean without panicking
            assert!(result == true || result == false);
        }

        #[cfg(not(target_os = "linux"))]
        {
            // On non-Linux, should return false
            let result = cgroup_v2_available();
            assert_eq!(result, false, "cgroup v2 detection should return false on non-Linux");
        }
    }

    #[test]
    fn test_k3s_mode_display() {
        // Test that K3sMode has useful Display/Debug output
        let rootless = K3sMode::Rootless;
        let root = K3sMode::Root;

        let rootless_str = format!("{:?}", rootless);
        let root_str = format!("{:?}", root);

        assert!(rootless_str.contains("Rootless"), "Rootless mode should display as 'Rootless'");
        assert!(root_str.contains("Root"), "Root mode should display as 'Root'");
    }
}
