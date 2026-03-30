//! Client-side Fusabi scripting system for hot-reloadable UI customization
//!
//! This module provides a complete scripting infrastructure using fusabi-frontend
//! for interpreting .fsx scripts that can customize the UI at runtime.
//!
//! Features:
//! - Hot-reload support with file watching
//! - Script error handling with UI display
//! - Access to Bevy resources (colors, fonts, window)
//! - Custom overlay/widget registration
//! - Event handling for daemon messages

pub mod api;
pub mod context;
pub mod error;
pub mod loader;
pub mod manager;
pub mod runtime;
pub mod watcher;

pub use api::{ScriptApi, ScriptContext, ScriptEvent};
pub use context::RuntimeContext;
pub use error::{ScriptError, ScriptResult};
pub use loader::ScriptLoader;
pub use manager::ScriptManager;
pub use runtime::ScriptRuntime;
pub use watcher::ScriptWatcher;

use bevy::prelude::*;

/// Main plugin that integrates scripting into the client
pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ScriptingSystemPlugin)
            .add_event::<ScriptEvent>()
            .add_systems(Startup, (context::initialize_context, initialize_scripting))
            .add_systems(
                Update,
                (
                    check_script_reloads,
                    execute_pending_scripts,
                    handle_script_events,
                    display_script_errors,
                ),
            );
    }
}

/// Internal plugin for scripting systems
struct ScriptingSystemPlugin;

impl Plugin for ScriptingSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScriptManager>()
            .init_resource::<ScriptErrorDisplay>();
    }
}

/// Initialize the scripting system on startup
fn initialize_scripting(
    mut manager: ResMut<ScriptManager>,
    config: Res<scarab_config::ScarabConfig>,
) {
    let scripts_dir = get_scripts_directory(&config);

    match manager.initialize(&scripts_dir) {
        Ok(count) => {
            info!("Scripting system initialized: {} scripts loaded", count);
        }
        Err(e) => {
            error!("Failed to initialize scripting system: {}", e);
        }
    }
}

/// Check for script file changes and trigger reloads
fn check_script_reloads(mut manager: ResMut<ScriptManager>) {
    if let Err(e) = manager.check_reloads() {
        error!("Script reload check failed: {}", e);
    }
}

/// Execute any pending scripts in the queue
fn execute_pending_scripts(
    mut manager: ResMut<ScriptManager>,
    context: Res<RuntimeContext>,
    mut commands: Commands,
) {
    manager.execute_pending(&context, &mut commands);
}

/// Handle script-generated events
fn handle_script_events(
    mut events: EventReader<ScriptEvent>,
    mut manager: ResMut<ScriptManager>,
) {
    for event in events.read() {
        if let Err(e) = manager.handle_event(event) {
            error!("Failed to handle script event: {}", e);
        }
    }
}

/// Display script errors in the UI
fn display_script_errors(
    errors: Res<ScriptErrorDisplay>,
    window: Query<&Window>,
) {
    if !errors.visible {
        return;
    }

    // This is a stub - in a real implementation, you'd use egui or a custom UI
    // to display the error overlay with formatted error messages
    if let Ok(_window) = window.get_single() {
        // Draw error indicator (simplified - would be a proper UI in production)
        for error in &errors.errors {
            error!("Script Error: {}", error.message);
        }
    }
}

/// Get the scripts directory from config or use default
fn get_scripts_directory(config: &scarab_config::ScarabConfig) -> std::path::PathBuf {
    // Check if config has a custom scripts path
    if let Some(ref custom_path) = config.plugins.config.get("scripts_directory") {
        if let Some(path_str) = custom_path.as_str() {
            return std::path::PathBuf::from(path_str);
        }
    }

    // Default to ~/.config/scarab/scripts
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(home)
        .join(".config")
        .join("scarab")
        .join("scripts")
}

/// Resource for displaying script errors in the UI
#[derive(Resource, Default)]
pub struct ScriptErrorDisplay {
    pub visible: bool,
    pub errors: Vec<DisplayedError>,
}

#[derive(Clone)]
pub struct DisplayedError {
    pub script_name: String,
    pub message: String,
    pub timestamp: std::time::Instant,
}

impl ScriptErrorDisplay {
    pub fn add_error(&mut self, script_name: String, message: String) {
        self.errors.push(DisplayedError {
            script_name,
            message,
            timestamp: std::time::Instant::now(),
        });
        self.visible = true;

        // Keep only the last 10 errors
        if self.errors.len() > 10 {
            self.errors.remove(0);
        }
    }

    pub fn clear(&mut self) {
        self.errors.clear();
        self.visible = false;
    }
}
