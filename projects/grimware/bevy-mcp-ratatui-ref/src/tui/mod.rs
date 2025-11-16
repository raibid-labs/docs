//! TUI (Terminal User Interface) rendering module
//!
//! Integrates bevy_ratatui_camera to render 3D Bevy scenes to the terminal.

pub mod config;
pub mod plugin;

pub use config::{TuiConfig, TuiRenderMode};
pub use plugin::BevyMcpTuiPlugin;
