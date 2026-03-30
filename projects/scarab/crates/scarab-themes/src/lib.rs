//! Theme System Plugin for Scarab Terminal
//!
//! This plugin provides:
//! - 10+ built-in professional themes
//! - Theme manager with preview/apply functionality
//! - Import/export themes in multiple formats (TOML, JSON, Base16)
//! - Command palette integration
//! - Hot-reload support (no restart required)
//! - Custom theme creation
//!
//! ## Architecture
//!
//! - `ThemePlugin`: Main plugin implementation
//! - `ThemeManager`: Core theme management logic
//! - `themes/`: Built-in theme definitions
//! - `format/`: Import/export format handlers
//!
//! ## Usage
//!
//! The plugin integrates with the command palette:
//! - "Theme: Select" - Choose from available themes
//! - "Theme: Preview" - Live preview without applying
//! - "Theme: Import" - Import theme from file
//! - "Theme: Export" - Export current theme
//! - "Theme: Create Custom" - Create theme from current colors

pub mod error;
pub mod format;
pub mod manager;
pub mod plugin;
pub mod theme;
pub mod themes;

pub use error::{ThemeError, ThemeResult};
pub use manager::ThemeManager;
pub use plugin::ThemePlugin;
pub use theme::{Theme, ThemeMetadata};

// Re-export common types
pub use scarab_config::ColorPalette;
