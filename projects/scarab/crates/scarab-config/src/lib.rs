//! Provides Fusabi-based configuration with:
//! - .fsx script loading
//! - Direct host function integration
//! - Type-safe configuration structs
//! - Bevy plugin for asset-based hot-reloading

pub mod config;
pub mod error;
pub mod fusabi_loader;
pub mod loader;
pub mod validation;
pub mod watcher;
pub mod registry;
pub mod plugin;

pub use config::{
    ScarabConfig, TerminalConfig, FontConfig, ColorConfig, ColorPalette,
    KeyBindings, UiConfig, TabPosition, CursorStyle, PluginConfig, SessionConfig
};
pub use error::{ConfigError, Result};
pub use fusabi_loader::FusabiConfigLoader;
pub use loader::ConfigLoader;
pub use validation::ConfigValidator;
pub use watcher::ConfigWatcher;
pub use registry::{RegistryManager, PluginFilter};
pub use plugin::{ScarabConfigPlugin, ConfigHandle};

pub mod prelude {
    pub use crate::config::*;
    pub use crate::error::*;
    pub use crate::fusabi_loader::*;
    pub use crate::loader::*;
    pub use crate::validation::*;
    pub use crate::watcher::*;
    pub use crate::registry::*;
    pub use crate::plugin::*;
}