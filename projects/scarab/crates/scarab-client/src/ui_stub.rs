// Temporary UI stub - Bevy 0.15 migration in progress
// This module provides minimal stubs to allow compilation
// Full UI features will be restored after core integration validated

use bevy::prelude::*;

/// Placeholder for advanced UI plugin (temporarily disabled)
pub struct AdvancedUIPlugin;

impl Plugin for AdvancedUIPlugin {
    fn build(&self, _app: &mut App) {
        warn!("AdvancedUIPlugin temporarily disabled during Bevy 0.15 migration");
        // UI features (link hints, command palette, leader keys) will be re-enabled
        // after core integration is validated
    }
}

// Re-export placeholder types for compatibility
pub use placeholder::*;

mod placeholder {
    use bevy::prelude::*;

    #[derive(Component)]
    pub struct LinkHint;

    #[derive(Resource)]
    pub struct LinkDetector;

    impl Default for LinkDetector {
        fn default() -> Self {
            Self
        }
    }

    #[derive(Resource)]
    pub struct LinkHintsState;

    impl Default for LinkHintsState {
        fn default() -> Self {
            Self
        }
    }

    #[derive(Event)]
    pub struct LinkActivatedEvent;

    pub struct Command;
    pub struct CommandRegistry;
    pub struct LeaderKeyState;
    pub struct KeyBinding;
    pub struct KeyBindingConfig;
    pub struct FadeAnimation;
    pub struct AnimationState;
    pub struct SelectionMode;
    pub struct SelectionRegion;

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
                link_hints_enabled: false, // Disabled during migration
                command_palette_enabled: false,
                leader_key_enabled: false,
                animations_enabled: false,
                leader_key_timeout_ms: 1000,
                fuzzy_search_threshold: 0.3,
            }
        }
    }
}
