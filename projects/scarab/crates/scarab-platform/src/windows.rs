//! Windows platform implementation

use crate::{GraphicsBackend, Platform};
use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct WindowsPlatform;

impl Platform for WindowsPlatform {
    fn socket_path() -> Result<PathBuf> {
        // Windows uses named pipes instead of Unix sockets
        // Format: \\.\pipe\scarab
        Ok(PathBuf::from(r"\\.\pipe\scarab"))
    }

    fn config_dir() -> Result<PathBuf> {
        dirs::config_dir()
            .map(|p| p.join("Scarab"))
            .context("Failed to get config directory")
    }

    fn data_dir() -> Result<PathBuf> {
        dirs::data_local_dir()
            .map(|p| p.join("Scarab"))
            .context("Failed to get data directory")
    }

    fn cache_dir() -> Result<PathBuf> {
        dirs::cache_dir()
            .map(|p| p.join("Scarab"))
            .context("Failed to get cache directory")
    }

    fn runtime_dir() -> Result<PathBuf> {
        std::env::var("TEMP")
            .or_else(|_| std::env::var("TMP"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(r"C:\Windows\Temp"))
            .join("Scarab")
            .into()
    }

    fn platform_name() -> &'static str {
        if cfg!(target_arch = "aarch64") {
            "Windows (ARM64)"
        } else {
            "Windows (x64)"
        }
    }

    fn is_virtualized() -> bool {
        utils::is_wsl_host() || utils::is_hyperv() || utils::is_vm()
    }

    fn graphics_backend() -> GraphicsBackend {
        // Prefer DirectX 12 on Windows, fallback to Vulkan
        if utils::has_dx12_support() {
            GraphicsBackend::DirectX12
        } else if utils::has_vulkan_support() {
            GraphicsBackend::Vulkan
        } else {
            GraphicsBackend::OpenGL
        }
    }

    fn init() -> Result<()> {
        // Create necessary directories
        let dirs = vec![
            Self::config_dir()?,
            Self::data_dir()?,
            Self::cache_dir()?,
            Self::runtime_dir()?,
        ];

        for dir in dirs {
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create directory: {:?}", dir))?;
        }

        Ok(())
    }
}

/// Windows-specific utilities
pub mod utils {
    use std::ptr::null_mut;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::sysinfoapi::{GetSystemInfo, SYSTEM_INFO};
    use winapi::um::winbase::GetComputerNameW;
    use winapi::um::winnt::LPCWSTR;

    /// Check if running under WSL host
    pub fn is_wsl_host() -> bool {
        std::env::var("WSL_DISTRO_NAME").is_ok()
    }

    /// Check if running under Hyper-V
    pub fn is_hyperv() -> bool {
        // Check for Hyper-V by looking at system info
        // This is a simplified check
        std::process::Command::new("wmic")
            .args(&["computersystem", "get", "model"])
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .to_lowercase()
                    .contains("virtual")
            })
            .unwrap_or(false)
    }

    /// Check if running in a VM
    pub fn is_vm() -> bool {
        std::process::Command::new("wmic")
            .args(&["computersystem", "get", "manufacturer"])
            .output()
            .map(|o| {
                let output = String::from_utf8_lossy(&o.stdout).to_lowercase();
                output.contains("vmware")
                    || output.contains("virtualbox")
                    || output.contains("qemu")
                    || output.contains("microsoft corporation")
            })
            .unwrap_or(false)
    }

    /// Check if DirectX 12 is available
    pub fn has_dx12_support() -> bool {
        // Check for dxgi.dll and d3d12.dll
        std::path::Path::new(r"C:\Windows\System32\d3d12.dll").exists()
            && std::path::Path::new(r"C:\Windows\System32\dxgi.dll").exists()
    }

    /// Check if Vulkan is available
    pub fn has_vulkan_support() -> bool {
        // Check for vulkan-1.dll
        std::path::Path::new(r"C:\Windows\System32\vulkan-1.dll").exists()
            || std::env::var("VK_ICD_FILENAMES").is_ok()
    }

    /// Get Windows version
    pub fn windows_version() -> Option<String> {
        std::process::Command::new("wmic")
            .args(&["os", "get", "Caption"])
            .output()
            .ok()
            .and_then(|o| {
                String::from_utf8(o.stdout)
                    .ok()
                    .and_then(|s| s.lines().nth(1).map(|l| l.trim().to_string()))
            })
    }
}
