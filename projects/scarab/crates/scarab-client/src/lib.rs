// Scarab terminal emulator client library
// Re-exports UI and rendering modules for use in binary and tests

pub mod terminal;
pub mod ui;
pub mod ui_stub;

pub mod integration;
pub mod ipc;
pub mod rendering;
pub mod scripting;
pub mod tutorial;

#[cfg(feature = "plugin-inspector")]
pub mod plugin_inspector;

pub use rendering::*;

// Re-export commonly used integration types
pub use integration::{extract_grid_text, get_cell_at, IntegrationPlugin, SharedMemoryReader};

// Re-export terminal types
pub use terminal::scrollback::{ScrollbackBuffer, ScrollbackLine, ScrollbackPlugin, ScrollbackState};

// Re-export UI plugin
pub use ui_stub::AdvancedUIPlugin;

// Re-export scripting system
pub use scripting::{ScriptingPlugin, ScriptManager, ScriptEvent, RuntimeContext};

// Re-export tutorial system
pub use tutorial::{TutorialPlugin, TutorialSystem, TutorialEvent, TutorialState};

// Re-export plugin inspector (feature-gated)
#[cfg(feature = "plugin-inspector")]
pub use plugin_inspector::{PluginInspectorPlugin, PluginInspectorState};
