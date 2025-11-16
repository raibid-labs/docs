//! Bevy MCP Ratatui Reference Implementation
//!
//! This library provides AI-controlled 3D game development in the terminal.
//! It integrates Bevy game engine with terminal rendering via bevy_ratatui_camera
//! and enables AI interaction through the Bevy Remote Protocol (BRP).
//!
//! # Features
//!
//! - `tui`: Terminal UI rendering with bevy_ratatui_camera
//! - `brp`: Bevy Remote Protocol for MCP integration
//! - `full`: Both TUI and BRP enabled
//!
//! # Example - Terminal-Only Rendering
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_mcp_ratatui_ref::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         // Terminal-only rendering (no window)
//!         .add_plugins(RatatuiPlugins::default())
//!         .add_plugins(BevyMcpTuiPlugin::default())
//!         .run();
//! }
//! ```
//!
//! # Example - Windowed + Terminal Dual Mode
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_mcp_ratatui_ref::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         // Windowed mode with terminal rendering
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(BevyMcpTuiPlugin::default())
//!         .run();
//! }
//! ```

pub mod systems;

#[cfg(feature = "tui")]
pub mod tui;

#[cfg(feature = "brp")]
pub mod brp;

/// Prelude module for convenient imports
pub mod prelude {
    #[cfg(feature = "tui")]
    pub use crate::tui::{BevyMcpTuiPlugin, TuiConfig, TuiRenderMode};

    #[cfg(feature = "tui")]
    pub use bevy_ratatui::RatatuiPlugins;

    #[cfg(feature = "brp")]
    pub use crate::brp::{BrpConfig, CustomBrpPlugin};

    pub use crate::systems::*;
}
