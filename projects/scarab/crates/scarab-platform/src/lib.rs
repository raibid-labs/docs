//! Platform abstraction layer for Scarab terminal emulator
//!
//! This module provides platform-specific implementations for:
//! - File paths (config, data, cache)
//! - IPC mechanisms (Unix sockets vs Named Pipes)
//! - Graphics backend selection
//! - System integration

use anyhow::Result;
use std::path::PathBuf;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub mod ipc;

/// Platform-specific behavior trait
pub trait Platform {
    /// Get the default socket/pipe path for IPC
    fn socket_path(&self) -> Result<PathBuf>;

    /// Get the configuration directory path
    fn config_dir(&self) -> Result<PathBuf>;

    /// Get the data directory path
    fn data_dir(&self) -> Result<PathBuf>;

    /// Get the cache directory path
    fn cache_dir(&self) -> Result<PathBuf>;

    /// Get the runtime directory path (for temporary files)
    fn runtime_dir(&self) -> Result<PathBuf>;

    /// Get platform name for display
    fn platform_name(&self) -> &'static str;

    /// Check if running in a container/VM
    fn is_virtualized(&self) -> bool {
        false
    }

    /// Get recommended graphics backend
    fn graphics_backend(&self) -> GraphicsBackend;

    /// Platform-specific initialization
    fn init(&self) -> Result<()>
    where
        Self: Sized,
    {
        Ok(())
    }
}

/// Graphics backend options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsBackend {
    Metal,
    Vulkan,
    DirectX12,
    OpenGL,
    Auto,
}

/// Current platform implementation
#[cfg(target_os = "macos")]
pub type CurrentPlatform = macos::MacPlatform;

#[cfg(target_os = "linux")]
pub type CurrentPlatform = linux::LinuxPlatform;

#[cfg(target_os = "windows")]
pub type CurrentPlatform = windows::WindowsPlatform;

/// Platform instance holder
pub struct PlatformInstance;

#[cfg(target_os = "macos")]
impl Platform for PlatformInstance {
    fn socket_path(&self) -> Result<PathBuf> {
        let platform = macos::MacPlatform;
        platform.socket_path()
    }
    fn config_dir(&self) -> Result<PathBuf> {
        let platform = macos::MacPlatform;
        platform.config_dir()
    }
    fn data_dir(&self) -> Result<PathBuf> {
        let platform = macos::MacPlatform;
        platform.data_dir()
    }
    fn cache_dir(&self) -> Result<PathBuf> {
        let platform = macos::MacPlatform;
        platform.cache_dir()
    }
    fn runtime_dir(&self) -> Result<PathBuf> {
        let platform = macos::MacPlatform;
        platform.runtime_dir()
    }
    fn platform_name(&self) -> &'static str {
        let platform = macos::MacPlatform;
        platform.platform_name()
    }
    fn graphics_backend(&self) -> GraphicsBackend {
        let platform = macos::MacPlatform;
        platform.graphics_backend()
    }
    fn init(&self) -> Result<()> {
        let platform = macos::MacPlatform;
        platform.init()
    }
}

#[cfg(target_os = "linux")]
impl Platform for PlatformInstance {
    fn socket_path(&self) -> Result<PathBuf> {
        linux::LinuxPlatform.socket_path()
    }
    fn config_dir(&self) -> Result<PathBuf> {
        linux::LinuxPlatform.config_dir()
    }
    fn data_dir(&self) -> Result<PathBuf> {
        linux::LinuxPlatform.data_dir()
    }
    fn cache_dir(&self) -> Result<PathBuf> {
        linux::LinuxPlatform.cache_dir()
    }
    fn runtime_dir(&self) -> Result<PathBuf> {
        linux::LinuxPlatform.runtime_dir()
    }
    fn platform_name(&self) -> &'static str {
        linux::LinuxPlatform.platform_name()
    }
    fn graphics_backend(&self) -> GraphicsBackend {
        linux::LinuxPlatform.graphics_backend()
    }
    fn init(&self) -> Result<()> {
        linux::LinuxPlatform.init()
    }
}

#[cfg(target_os = "windows")]
impl Platform for PlatformInstance {
    fn socket_path(&self) -> Result<PathBuf> {
        windows::WindowsPlatform::socket_path()
    }
    fn config_dir(&self) -> Result<PathBuf> {
        windows::WindowsPlatform::config_dir()
    }
    fn data_dir(&self) -> Result<PathBuf> {
        windows::WindowsPlatform::data_dir()
    }
    fn cache_dir(&self) -> Result<PathBuf> {
        windows::WindowsPlatform::cache_dir()
    }
    fn runtime_dir(&self) -> Result<PathBuf> {
        windows::WindowsPlatform::runtime_dir()
    }
    fn platform_name(&self) -> &'static str {
        windows::WindowsPlatform::platform_name()
    }
    fn graphics_backend(&self) -> GraphicsBackend {
        windows::WindowsPlatform::graphics_backend()
    }
    fn init(&self) -> Result<()> {
        windows::WindowsPlatform::init()
    }
}

/// Get the current platform
pub fn current_platform() -> &'static dyn Platform {
    &PlatformInstance
}

/// Platform detection utilities
pub mod detect {

    /// Detect if running under WSL
    #[cfg(target_os = "linux")]
    pub fn is_wsl() -> bool {
        std::path::Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists()
    }

    #[cfg(not(target_os = "linux"))]
    pub fn is_wsl() -> bool {
        false
    }

    /// Detect if running under X11
    #[cfg(target_os = "linux")]
    pub fn is_x11() -> bool {
        std::env::var("DISPLAY").is_ok()
    }

    #[cfg(not(target_os = "linux"))]
    pub fn is_x11() -> bool {
        false
    }

    /// Detect if running under Wayland
    #[cfg(target_os = "linux")]
    pub fn is_wayland() -> bool {
        std::env::var("WAYLAND_DISPLAY").is_ok()
    }

    #[cfg(not(target_os = "linux"))]
    pub fn is_wayland() -> bool {
        false
    }
}
