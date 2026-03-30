// Advanced UI/UX module for Scarab terminal emulator
// Provides power-user features: link hints, command palette, leader keys, etc.

pub mod animations;
pub mod command_palette;
pub mod grid_utils;
pub mod keybindings;
pub mod leader_key;
pub mod link_hints;
pub mod overlays;
pub mod scroll_indicator;
pub mod scrollback_selection;
pub mod search_overlay;
pub mod visual_selection;

pub use animations::{AnimationState, AnimationsPlugin, FadeAnimation};
pub use command_palette::{Command, CommandPalettePlugin, CommandRegistry};
pub use grid_utils::{grid_to_pixel, grid_to_pixel_with_renderer, pixel_to_grid, grid_cell_bounds, grid_cell_center, grid_region_bounds};
pub use keybindings::{KeyBinding, KeyBindingConfig, KeybindingsPlugin};
pub use leader_key::{LeaderKeyPlugin, LeaderKeyState};
pub use link_hints::{LinkDetector, LinkHint, LinkHintsPlugin};
pub use overlays::RemoteUiPlugin;
pub use scroll_indicator::{ScrollIndicatorConfig, ScrollIndicatorPlugin};
pub use scrollback_selection::{ScrollbackSelectionPlugin, ScrollbackSelectionState};
pub use search_overlay::{SearchOverlayConfig, SearchOverlayPlugin};
pub use visual_selection::{SelectionMode, SelectionRegion, VisualSelectionPlugin};

use bevy::prelude::*;

/// Main plugin that bundles all UI features
pub struct AdvancedUIPlugin;

impl Plugin for AdvancedUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            LinkHintsPlugin,
            CommandPalettePlugin,
            LeaderKeyPlugin,
            KeybindingsPlugin,
            AnimationsPlugin,
            VisualSelectionPlugin,
            RemoteUiPlugin,
            ScrollIndicatorPlugin,
            ScrollbackSelectionPlugin,
            SearchOverlayPlugin,
        ))
        .insert_resource(UIConfig::default());
    }
}

/// Global UI configuration
#[derive(Resource, Clone)]
pub struct UIConfig {
    pub link_hints_enabled: bool,
    pub command_palette_enabled: bool,
    pub leader_key_enabled: bool,
    pub animations_enabled: bool,
    pub leader_key_timeout_ms: u64,
    pub fuzzy_search_threshold: f64,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            link_hints_enabled: true,
            command_palette_enabled: true,
            leader_key_enabled: true,
            animations_enabled: true,
            leader_key_timeout_ms: 1000,
            fuzzy_search_threshold: 0.3,
        }
    }
}
