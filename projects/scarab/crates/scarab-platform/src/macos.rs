//! macOS platform implementation

use crate::{GraphicsBackend, Platform};
use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct MacPlatform;

impl Platform for MacPlatform {
    fn socket_path(&self) -> Result<PathBuf> {
        let runtime_dir = self.runtime_dir()?;
        Ok(runtime_dir.join("scarab.sock"))
    }

    fn config_dir(&self) -> Result<PathBuf> {
        dirs::config_dir()
            .map(|p| p.join("scarab"))
            .context("Failed to get config directory")
    }

    fn data_dir(&self) -> Result<PathBuf> {
        dirs::data_dir()
            .map(|p| p.join("scarab"))
            .context("Failed to get data directory")
    }

    fn cache_dir(&self) -> Result<PathBuf> {
        dirs::cache_dir()
            .map(|p| p.join("scarab"))
            .context("Failed to get cache directory")
    }

    fn runtime_dir(&self) -> Result<PathBuf> {
        let dir = std::env::var("TMPDIR")
            .map(PathBuf::from)
            .or_else(|_| std::env::var("XDG_RUNTIME_DIR").map(PathBuf::from))
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
            .join("scarab");
        Ok(dir)
    }

    fn platform_name(&self) -> &'static str {
        if cfg!(target_arch = "aarch64") {
            "macOS (Apple Silicon)"
        } else {
            "macOS (Intel)"
        }
    }

    fn graphics_backend(&self) -> GraphicsBackend {
        // Always use Metal on macOS for best performance
        GraphicsBackend::Metal
    }

    fn init(&self) -> Result<()> {
        // Create necessary directories
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

        Ok(())
    }
}

/// macOS-specific utilities
pub mod utils {
    use std::process::Command;

    /// Check if running on Apple Silicon
    pub fn is_apple_silicon() -> bool {
        cfg!(target_arch = "aarch64")
    }

    /// Get macOS version
    pub fn macos_version() -> Option<String> {
        Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    }

    /// Check if running under Rosetta 2
    pub fn is_rosetta() -> bool {
        Command::new("sysctl")
            .arg("-n")
            .arg("sysctl.proc_translated")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim() == "1")
            .unwrap_or(false)
    }
}
