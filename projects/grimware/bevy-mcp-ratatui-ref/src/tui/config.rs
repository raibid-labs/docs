//! TUI configuration structures

use bevy::prelude::*;

/// TUI rendering mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum TuiRenderMode {
    /// ASCII character density rendering
    Ascii,
    /// Unicode block characters with color
    Color,
    /// Edge detection wireframe
    Edge,
    /// Automatically select based on terminal capabilities
    Auto,
}

impl Default for TuiRenderMode {
    fn default() -> Self {
        Self::Auto
    }
}

/// TUI rendering configuration
#[derive(Resource, Debug, Clone, Reflect)]
pub struct TuiConfig {
    /// Enable terminal output
    pub enabled: bool,
    /// Rendering mode
    pub render_mode: TuiRenderMode,
    /// Target frame rate for terminal rendering
    pub target_fps: u32,
    /// Terminal width (auto-detected if 0)
    pub width: u32,
    /// Terminal height (auto-detected if 0)
    pub height: u32,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            render_mode: TuiRenderMode::default(),
            target_fps: 30,
            width: 0,  // Auto-detect
            height: 0, // Auto-detect
        }
    }
}
