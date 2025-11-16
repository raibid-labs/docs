//! Flux GitOps Installation Module
//!
//! This module handles installing Flux CLI and bootstrapping Flux controllers
//! in the k3s cluster with Gitea as the Git source for GitOps workflows.

use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

/// Flux version to install
const FLUX_VERSION: &str = "v2.2.3";
const FLUX_GITHUB_RELEASE_URL: &str = "https://github.com/fluxcd/flux2/releases/download";

/// Platform-specific binary names
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    LinuxArm64,
    LinuxAmd64,
    DarwinArm64,
    DarwinAmd64,
}

impl Platform {
    /// Detect the current platform
    pub fn detect() -> Result<Self> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        match (os, arch) {
            ("linux", "aarch64") => Ok(Platform::LinuxArm64),
            ("linux", "x86_64") => Ok(Platform::LinuxAmd64),
            ("macos", "aarch64") => Ok(Platform::DarwinArm64),
            ("macos", "x86_64") => Ok(Platform::DarwinAmd64),
            _ => Err(anyhow!("Unsupported platform: {} {}", os, arch)),
        }
    }

    /// Get the archive name for this platform
    pub fn archive_name(&self) -> &str {
        match self {
            Platform::LinuxArm64 => "flux_2.2.3_linux_arm64.tar.gz",
            Platform::LinuxAmd64 => "flux_2.2.3_linux_amd64.tar.gz",
            Platform::DarwinArm64 => "flux_2.2.3_darwin_arm64.tar.gz",
            Platform::DarwinAmd64 => "flux_2.2.3_darwin_amd64.tar.gz",
        }
    }

    /// Get the checksum file name
    pub fn checksum_name(&self) -> &str {
        "flux_2.2.3_checksums.txt"
    }
}

/// Flux GitOps configuration
#[derive(Debug, Clone)]
pub struct FluxConfig {
    /// Flux version to install
    pub version: String,
    /// Installation directory (default: /usr/local/bin)
    pub install_dir: PathBuf,
    /// Namespace for Flux controllers (default: flux-system)
    pub namespace: String,
    /// Gitea repository URL
    pub gitea_url: String,
    /// Gitea repository name (e.g., "raibid-gitops")
    pub repository: String,
    /// Gitea username
    pub username: String,
    /// Gitea password or token
    pub password: String,
    /// Branch to track (default: main)
    pub branch: String,
    /// Path within repository for manifests (default: clusters/raibid)
    pub path: String,
    /// Kubeconfig path
    pub kubeconfig_path: PathBuf,
    /// Enable image automation
    pub enable_image_automation: bool,
    /// Enable notification controller
    pub enable_notifications: bool,
    /// Reconciliation interval (default: 1m)
    pub interval: String,
}

impl Default for FluxConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/root"));
        Self {
            version: FLUX_VERSION.to_string(),
            install_dir: home.join(".local").join("bin"), // User-local, no sudo required
            namespace: "flux-system".to_string(),
            gitea_url: "http://gitea.gitea.svc.cluster.local:3000".to_string(),
            repository: "raibid-gitops".to_string(),
            username: "raibid-admin".to_string(),
            password: String::new(), // Must be provided
            branch: "main".to_string(),
            path: "clusters/raibid".to_string(),
            kubeconfig_path: home.join(".kube").join("config"),
            enable_image_automation: true,
            enable_notifications: true,
            interval: "1m".to_string(),
        }
    }
}

impl FluxConfig {
    /// Get the full repository URL
    #[allow(dead_code)]
    pub fn repository_url(&self) -> String {
        format!(
            "{}/{}/{}.git",
            self.gitea_url, self.username, self.repository
        )
    }

    /// Get the repository URL with credentials
    pub fn repository_url_with_auth(&self) -> String {
        let url = self
            .gitea_url
            .trim_start_matches("http://")
            .trim_start_matches("https://");

        format!(
            "http://{}:{}@{}/{}/{}.git",
            self.username, self.password, url, self.username, self.repository
        )
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.password.is_empty() {
            return Err(anyhow!("Gitea password is required"));
        }
        if self.repository.is_empty() {
            return Err(anyhow!("Repository name is required"));
        }
        if self.username.is_empty() {
            return Err(anyhow!("Username is required"));
        }
        Ok(())
    }
}

/// Flux installer
pub struct FluxInstaller {
    config: FluxConfig,
    platform: Platform,
    download_dir: PathBuf,
}

impl FluxInstaller {
    /// Create a new Flux installer with default configuration
    #[allow(dead_code)]
    pub fn new() -> Result<Self> {
        Self::with_config(FluxConfig::default())
    }

    /// Create a new Flux installer with custom configuration
    pub fn with_config(config: FluxConfig) -> Result<Self> {
        config.validate()?;

        let platform = Platform::detect()?;
        let download_dir = std::env::temp_dir().join("flux-install");

        Ok(Self {
            config,
            platform,
            download_dir,
        })
    }

    /// Check if Flux CLI is already installed
    pub fn check_flux_cli(&self) -> Result<bool> {
        let output = Command::new("flux").arg("--version").output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout);
                    debug!("Flux CLI found: {}", version.trim());
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(_) => Ok(false),
        }
    }

    /// Download Flux CLI binary archive
    pub async fn download_flux(&self) -> Result<PathBuf> {
        // Create download directory
        fs::create_dir_all(&self.download_dir).context("Failed to create download directory")?;

        let archive_name = self.platform.archive_name();
        let download_url = format!(
            "{}/{}/{}",
            FLUX_GITHUB_RELEASE_URL, self.config.version, archive_name
        );

        let archive_path = self.download_dir.join(archive_name);

        info!("Downloading Flux from: {}", download_url);

        // Download the archive
        let response = reqwest::get(&download_url)
            .await
            .context("Failed to download Flux")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download Flux: HTTP {}",
                response.status()
            ));
        }

        let bytes = response
            .bytes()
            .await
            .context("Failed to read response bytes")?;

        let mut file = fs::File::create(&archive_path).context("Failed to create archive file")?;
        file.write_all(&bytes).context("Failed to write archive")?;

        info!("Downloaded Flux archive to: {}", archive_path.display());

        Ok(archive_path)
    }

    /// Download checksums file
    pub async fn download_checksums(&self) -> Result<PathBuf> {
        let checksum_name = self.platform.checksum_name();
        let download_url = format!(
            "{}/{}/{}",
            FLUX_GITHUB_RELEASE_URL, self.config.version, checksum_name
        );

        let checksum_path = self.download_dir.join(checksum_name);

        info!("Downloading checksums from: {}", download_url);

        let response = reqwest::get(&download_url)
            .await
            .context("Failed to download checksums")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download checksums: HTTP {}",
                response.status()
            ));
        }

        let bytes = response
            .bytes()
            .await
            .context("Failed to read checksum bytes")?;

        fs::write(&checksum_path, &bytes).context("Failed to write checksum file")?;

        Ok(checksum_path)
    }

    /// Verify archive checksum
    pub fn verify_checksum(&self, archive_path: &Path, checksums_path: &Path) -> Result<()> {
        let checksums =
            fs::read_to_string(checksums_path).context("Failed to read checksums file")?;

        let archive_name = archive_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid archive path"))?;

        // Find the line with our archive
        let expected_checksum = checksums
            .lines()
            .find(|line| line.contains(archive_name))
            .and_then(|line| line.split_whitespace().next())
            .ok_or_else(|| anyhow!("Checksum not found for {}", archive_name))?;

        // Calculate actual checksum
        let archive_bytes =
            fs::read(archive_path).context("Failed to read archive for checksum")?;
        let actual_checksum = sha256::digest(&archive_bytes);

        if actual_checksum != expected_checksum {
            return Err(anyhow!(
                "Checksum mismatch for {}: expected {}, got {}",
                archive_name,
                expected_checksum,
                actual_checksum
            ));
        }

        info!("Checksum verified successfully");
        Ok(())
    }

    /// Extract and install Flux CLI
    pub fn install_flux_cli(&self, archive_path: &Path) -> Result<()> {
        use crate::infrastructure::utils::{check_directory_writable, warn_if_not_in_path};
        use std::os::unix::fs::PermissionsExt;

        info!("Extracting Flux CLI from archive");

        // Extract tar.gz archive
        let output = Command::new("tar")
            .arg("-xzf")
            .arg(archive_path)
            .arg("-C")
            .arg(&self.download_dir)
            .output()
            .context("Failed to extract archive")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to extract archive: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Check if we can write to the install directory before proceeding
        check_directory_writable(&self.config.install_dir)?;

        // Move flux binary to install directory
        let flux_binary = self.download_dir.join("flux");
        let install_path = self.config.install_dir.join("flux");

        // Copy binary to install location (no sudo needed with user-local directory)
        fs::copy(&flux_binary, &install_path)
            .context("Failed to copy Flux binary to install directory")?;

        // Make executable (chmod +x)
        let mut perms = fs::metadata(&install_path)
            .context("Failed to get Flux binary metadata")?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&install_path, perms)
            .context("Failed to set Flux binary permissions")?;

        info!("Flux CLI installed to: {}", install_path.display());

        // Warn if install directory is not in PATH
        warn_if_not_in_path(&self.config.install_dir);

        Ok(())
    }

    /// Bootstrap Flux in the k3s cluster
    pub fn bootstrap_flux(&self) -> Result<()> {
        info!("Bootstrapping Flux with Gitea repository");

        // Prepare bootstrap command
        let mut cmd = Command::new("flux");
        cmd.arg("bootstrap")
            .arg("git")
            .arg("--url")
            .arg(self.config.repository_url_with_auth())
            .arg("--branch")
            .arg(&self.config.branch)
            .arg("--path")
            .arg(&self.config.path)
            .arg("--namespace")
            .arg(&self.config.namespace)
            .arg("--interval")
            .arg(&self.config.interval);

        // Add image automation components if enabled
        if self.config.enable_image_automation {
            cmd.arg("--components-extra")
                .arg("image-reflector-controller,image-automation-controller");
        }

        // Set kubeconfig
        cmd.env("KUBECONFIG", &self.config.kubeconfig_path);

        debug!("Running: {:?}", cmd);

        let output = cmd.output().context("Failed to run flux bootstrap")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Flux bootstrap failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        info!("Flux bootstrapped successfully");
        info!("{}", String::from_utf8_lossy(&output.stdout));

        Ok(())
    }

    /// Create GitRepository resource
    #[allow(dead_code)]
    pub fn create_git_repository(&self, name: &str, url: &str, branch: &str) -> Result<()> {
        info!("Creating GitRepository resource: {}", name);

        let output = Command::new("flux")
            .arg("create")
            .arg("source")
            .arg("git")
            .arg(name)
            .arg("--url")
            .arg(url)
            .arg("--branch")
            .arg(branch)
            .arg("--interval")
            .arg(&self.config.interval)
            .arg("--namespace")
            .arg(&self.config.namespace)
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to create GitRepository")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to create GitRepository: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        info!("GitRepository '{}' created", name);
        Ok(())
    }

    /// Create Kustomization resource
    #[allow(dead_code)]
    pub fn create_kustomization(&self, name: &str, source: &str, path: &str) -> Result<()> {
        info!("Creating Kustomization resource: {}", name);

        let output = Command::new("flux")
            .arg("create")
            .arg("kustomization")
            .arg(name)
            .arg("--source")
            .arg(format!("GitRepository/{}", source))
            .arg("--path")
            .arg(path)
            .arg("--prune")
            .arg("true")
            .arg("--interval")
            .arg(&self.config.interval)
            .arg("--namespace")
            .arg(&self.config.namespace)
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to create Kustomization")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to create Kustomization: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        info!("Kustomization '{}' created", name);
        Ok(())
    }

    /// Configure image automation
    pub fn configure_image_automation(&self) -> Result<()> {
        if !self.config.enable_image_automation {
            info!("Image automation is disabled");
            return Ok(());
        }

        info!("Configuring image automation");

        // Create ImageRepository resource for Gitea OCI registry
        let mut child = Command::new("kubectl")
            .arg("apply")
            .arg("-f")
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .spawn()
            .context("Failed to spawn kubectl")?;

        let manifest = format!(
            r#"apiVersion: image.toolkit.fluxcd.io/v1beta2
kind: ImageRepository
metadata:
  name: raibid-images
  namespace: {}
spec:
  image: gitea.gitea.svc.cluster.local:3000/raibid/images
  interval: 5m
"#,
            self.config.namespace
        );

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(manifest.as_bytes())
                .context("Failed to write ImageRepository manifest")?;
        }

        let output = child
            .wait_with_output()
            .context("Failed to wait for kubectl")?;

        if !output.status.success() {
            warn!(
                "Failed to create ImageRepository: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        } else {
            info!("ImageRepository configured");
        }

        Ok(())
    }

    /// Configure notification controller
    pub fn configure_notifications(&self) -> Result<()> {
        if !self.config.enable_notifications {
            info!("Notifications are disabled");
            return Ok(());
        }

        info!("Configuring notification controller");

        // Create Provider resource for Gitea webhooks
        let mut child = Command::new("kubectl")
            .arg("apply")
            .arg("-f")
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .spawn()
            .context("Failed to spawn kubectl")?;

        let manifest = format!(
            r#"apiVersion: notification.toolkit.fluxcd.io/v1beta3
kind: Provider
metadata:
  name: gitea
  namespace: {}
spec:
  type: gitea
  address: {}
"#,
            self.config.namespace, self.config.gitea_url
        );

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(manifest.as_bytes())
                .context("Failed to write Provider manifest")?;
        }

        let output = child
            .wait_with_output()
            .context("Failed to wait for kubectl")?;

        if !output.status.success() {
            warn!(
                "Failed to create Provider: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        } else {
            info!("Notification Provider configured");
        }

        Ok(())
    }

    /// Validate Flux installation
    pub fn validate_installation(&self) -> Result<()> {
        info!("Validating Flux installation");

        // Check Flux controllers
        let output = Command::new("flux")
            .arg("check")
            .arg("--namespace")
            .arg(&self.config.namespace)
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to run flux check")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Flux validation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        info!("Flux installation validated successfully");
        info!("{}", String::from_utf8_lossy(&output.stdout));

        Ok(())
    }

    /// Get Flux status
    pub fn get_status(&self) -> Result<String> {
        let output = Command::new("flux")
            .arg("get")
            .arg("all")
            .arg("--namespace")
            .arg(&self.config.namespace)
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output()
            .context("Failed to get Flux status")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to get Flux status: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Rollback Flux installation
    pub fn rollback(&self) -> Result<()> {
        warn!("Rolling back Flux installation");

        // Uninstall Flux
        let output = Command::new("flux")
            .arg("uninstall")
            .arg("--namespace")
            .arg(&self.config.namespace)
            .arg("--silent")
            .env("KUBECONFIG", &self.config.kubeconfig_path)
            .output();

        if let Ok(output) = output {
            if !output.status.success() {
                warn!(
                    "Failed to uninstall Flux: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }

        Ok(())
    }

    /// Clean up temporary files
    pub fn cleanup(&self) -> Result<()> {
        if self.download_dir.exists() {
            fs::remove_dir_all(&self.download_dir)
                .context("Failed to remove download directory")?;
            debug!(
                "Cleaned up download directory: {}",
                self.download_dir.display()
            );
        }
        Ok(())
    }
}
