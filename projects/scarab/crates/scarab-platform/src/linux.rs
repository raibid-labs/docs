//! Linux platform implementation

use crate::{detect, GraphicsBackend, Platform};
use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct LinuxPlatform;

impl Platform for LinuxPlatform {
    fn socket_path(&self) -> Result<PathBuf> {
        let runtime_dir = self.runtime_dir()?;
        Ok(runtime_dir.join("scarab.sock"))
    }

    fn config_dir(&self) -> Result<PathBuf> {
        std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|_| {
                dirs::config_dir()
                    .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))
            })
            .map(|p| p.join("scarab"))
            .context("Failed to get config directory")
    }

    fn data_dir(&self) -> Result<PathBuf> {
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|_| {
                dirs::data_dir()
                    .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))
            })
            .map(|p| p.join("scarab"))
            .context("Failed to get data directory")
    }

    fn cache_dir(&self) -> Result<PathBuf> {
        std::env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|_| {
                dirs::cache_dir()
                    .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))
            })
            .map(|p| p.join("scarab"))
            .context("Failed to get cache directory")
    }

    fn runtime_dir(&self) -> Result<PathBuf> {
        let path = std::env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .or_else(|_| std::env::var("TMPDIR").map(PathBuf::from))
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
            .join(format!("scarab-{}", users::get_current_uid()));

        Ok(path)
    }

    fn platform_name(&self) -> &'static str {
        if detect::is_wsl() {
            "Linux (WSL)"
        } else if detect::is_wayland() {
            "Linux (Wayland)"
        } else if detect::is_x11() {
            "Linux (X11)"
        } else {
            "Linux"
        }
    }

    fn is_virtualized(&self) -> bool {
        detect::is_wsl() || utils::is_docker() || utils::is_vm()
    }

    fn graphics_backend(&self) -> GraphicsBackend {
        // Prefer Vulkan on Linux, fallback to OpenGL
        if utils::has_vulkan_support() {
            GraphicsBackend::Vulkan
        } else {
            GraphicsBackend::OpenGL
        }
    }

    fn init(&self) -> Result<()> {
        // Create necessary directories with proper permissions
        let dirs = vec![
            self.config_dir()?,
            self.data_dir()?,
            self.cache_dir()?,
            self.runtime_dir()?,
        ];

        for dir in dirs {
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create directory: {:?}", dir))?;
        }

        // Set proper permissions for runtime directory (700)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let runtime_dir = self.runtime_dir()?;
            let metadata = std::fs::metadata(&runtime_dir)?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o700);
            std::fs::set_permissions(&runtime_dir, permissions)?;
        }

        Ok(())
    }
}

/// Linux-specific utilities
pub mod utils {
    use std::path::Path;

    /// Check if running in Docker
    pub fn is_docker() -> bool {
        Path::new("/.dockerenv").exists()
            || std::fs::read_to_string("/proc/1/cgroup")
                .map(|s| s.contains("docker"))
                .unwrap_or(false)
    }

    /// Check if running in a VM
    pub fn is_vm() -> bool {
        std::fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
            .map(|s| {
                let s = s.to_lowercase();
                s.contains("virtualbox")
                    || s.contains("vmware")
                    || s.contains("qemu")
                    || s.contains("kvm")
            })
            .unwrap_or(false)
    }

    /// Check if Vulkan is available
    pub fn has_vulkan_support() -> bool {
        // Simple check for Vulkan ICD files
        Path::new("/usr/share/vulkan/icd.d").exists()
            || Path::new("/etc/vulkan/icd.d").exists()
            || std::env::var("VK_ICD_FILENAMES").is_ok()
    }

    /// Get distribution info from /etc/os-release
    /// Currently unused but available for debugging/logging purposes
    #[allow(dead_code)]
    pub fn distro_info() -> Option<String> {
        std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|line| line.starts_with("PRETTY_NAME="))
                    .map(|line| {
                        line.trim_start_matches("PRETTY_NAME=")
                            .trim_matches('"')
                            .to_string()
                    })
            })
    }
}

// Add users dependency for Linux
mod users {
    pub fn get_current_uid() -> u32 {
        #[cfg(unix)]
        unsafe {
            libc::getuid()
        }
        #[cfg(not(unix))]
        0
    }
}
