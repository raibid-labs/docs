//! Bevy plugin for Fusabi-based configuration with hot-reloading
//!
//! This plugin integrates with bevy-fusabi's asset system to provide
//! automatic loading and hot-reloading of configuration files.

use bevy::prelude::*;
use bevy_fusabi::prelude::*;
use fusabi_vm::{Value, Vm};
use crate::{
    config::*,
    error::{ConfigError, Result},
};
use std::collections::HashMap;

/// Resource to hold the handle to the config script asset
#[derive(Resource, Clone)]
pub struct ConfigHandle(pub Handle<FusabiScript>);

/// Bevy plugin for Fusabi-based configuration
pub struct ScarabConfigPlugin {
    /// Path to the config file (relative to assets directory)
    pub config_path: String,
}

impl Default for ScarabConfigPlugin {
    fn default() -> Self {
        Self {
            config_path: "config.fsx".to_string(),
        }
    }
}

impl ScarabConfigPlugin {
    pub fn new(config_path: impl Into<String>) -> Self {
        Self {
            config_path: config_path.into(),
        }
    }
}

impl Plugin for ScarabConfigPlugin {
    fn build(&self, app: &mut App) {
        let config_path = self.config_path.clone();

        app.add_plugins(FusabiPlugin)
           .init_resource::<ScarabConfig>()
           .add_systems(Startup, move |commands: Commands, asset_server: Res<AssetServer>| {
               load_config(commands, asset_server, &config_path)
           })
           .add_systems(Update, apply_config_updates);
    }
}

/// Load the config file on startup
fn load_config(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config_path: &str,
) {
    info!("Loading configuration from: {}", config_path);
    let handle = asset_server.load(config_path);
    commands.insert_resource(ConfigHandle(handle));
}

/// Watch for config file changes and apply updates
fn apply_config_updates(
    mut events: EventReader<AssetEvent<FusabiScript>>,
    config_handle: Option<Res<ConfigHandle>>,
    scripts: Res<Assets<FusabiScript>>,
    mut config_store: ResMut<ScarabConfig>,
) {
    let Some(config_handle) = config_handle else { return };

    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                // Only process events for our config file
                if *id != config_handle.0.id() {
                    continue;
                }

                if let Some(script) = scripts.get(*id) {
                    info!("Reloading configuration: {}", script.name);
                    if let Err(e) = apply_script(script, &mut config_store) {
                        error!("Failed to apply config: {:?}", e);
                    } else {
                        info!("Configuration reloaded successfully");
                    }
                }
            }
            AssetEvent::LoadedWithDependencies { id } => {
                if *id != config_handle.0.id() {
                    continue;
                }

                if let Some(script) = scripts.get(*id) {
                    info!("Configuration loaded: {}", script.name);
                    if let Err(e) = apply_script(script, &mut config_store) {
                        error!("Failed to apply initial config: {:?}", e);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Execute the script and extract configuration values
fn apply_script(script: &FusabiScript, config: &mut ScarabConfig) -> Result<()> {
    // 1. Deserialize bytecode
    let chunk = script.to_chunk()
        .map_err(|e| ConfigError::FusabiRuntimeError(format!("Failed to deserialize chunk: {}", e)))?;

    // 2. Execute VM
    let mut vm = Vm::new();
    let result = vm.execute(chunk)
        .map_err(|e| ConfigError::FusabiRuntimeError(format!("Failed to execute config: {:?}", e)))?;

    // 3. Extract configuration from VM globals and result
    update_config_from_vm(&vm, &result, config)?;

    Ok(())
}

/// Extract configuration from VM execution result
fn update_config_from_vm(vm: &Vm, result: &Value, config: &mut ScarabConfig) -> Result<()> {
    // Helper to get a value from either VM globals or result record
    let get_global = |name: &str| -> Option<Value> {
        // First check VM globals
        if let Some(v) = vm.globals.get(name) {
            return Some(v.clone());
        }

        // Then check if the result is a Record containing the name
        if let Value::Record(map) = result {
            if let Some(v) = map.borrow().get(name) {
                return Some(v.clone());
            }
        }

        None
    };

    // Extract configuration sections
    if let Some(val) = get_global("terminal") {
        if let Ok(c) = extract_terminal_config(&val) {
            config.terminal = c;
        }
    }

    if let Some(val) = get_global("font") {
        if let Ok(c) = extract_font_config(&val) {
            config.font = c;
        }
    }

    if let Some(val) = get_global("colors") {
        if let Ok(c) = extract_color_config(&val) {
            config.colors = c;
        }
    }

    if let Some(val) = get_global("keybindings") {
        if let Ok(c) = extract_keybindings(&val) {
            config.keybindings = c;
        }
    }

    if let Some(val) = get_global("ui") {
        if let Ok(c) = extract_ui_config(&val) {
            config.ui = c;
        }
    }

    if let Some(val) = get_global("plugins") {
        if let Ok(c) = extract_plugin_config(&val) {
            config.plugins = c;
        }
    }

    if let Some(val) = get_global("sessions") {
        if let Ok(c) = extract_session_config(&val) {
            config.sessions = c;
        }
    }

    Ok(())
}

// --- Extraction functions (refactored to work with &Value) ---

fn extract_terminal_config(val: &Value) -> Result<TerminalConfig> {
    let mut config = TerminalConfig::default();

    if let Value::Record(map) = val {
        let map = map.borrow();
        if let Some(s) = get_string(&map, "DefaultShell") { config.default_shell = s; }
        if let Some(i) = get_int(&map, "ScrollbackLines") { config.scrollback_lines = i as u32; }
        if let Some(b) = get_bool(&map, "AltScreen") { config.alt_screen = b; }
        if let Some(f) = get_float(&map, "ScrollMultiplier") { config.scroll_multiplier = f as f32; }
        if let Some(b) = get_bool(&map, "AutoScroll") { config.auto_scroll = b; }
        if let Some(i) = get_int(&map, "Columns") { config.columns = i as u16; }
        if let Some(i) = get_int(&map, "Rows") { config.rows = i as u16; }
    }

    Ok(config)
}

fn extract_font_config(val: &Value) -> Result<FontConfig> {
    let mut config = FontConfig::default();

    if let Value::Record(map) = val {
        let map = map.borrow();
        if let Some(s) = get_string(&map, "Family") { config.family = s; }
        if let Some(f) = get_float(&map, "Size") { config.size = f as f32; }
        if let Some(f) = get_float(&map, "LineHeight") { config.line_height = f as f32; }
        if let Some(b) = get_bool(&map, "BoldIsBright") { config.bold_is_bright = b; }
        if let Some(b) = get_bool(&map, "UseThinStrokes") { config.use_thin_strokes = b; }

        if let Some(Value::Tuple(vec)) = map.get("Fallback") {
            let mut fallback = Vec::new();
            for v in vec {
                if let Value::Str(s) = v {
                    fallback.push(s.to_string());
                }
            }
            if !fallback.is_empty() {
                config.fallback = fallback;
            }
        }
    }

    Ok(config)
}

fn extract_color_config(val: &Value) -> Result<ColorConfig> {
    let mut config = ColorConfig::default();

    if let Value::Record(map) = val {
        let map = map.borrow();
        if let Some(s) = get_string(&map, "Theme") { config.theme = Some(s); }
        if let Some(f) = get_float(&map, "Opacity") { config.opacity = f as f32; }
        if let Some(f) = get_float(&map, "DimOpacity") { config.dim_opacity = f as f32; }

        // Optional overrides
        if let Some(s) = get_string(&map, "Foreground") { config.foreground = Some(s); }
        if let Some(s) = get_string(&map, "Background") { config.background = Some(s); }
        if let Some(s) = get_string(&map, "Cursor") { config.cursor = Some(s); }
        if let Some(s) = get_string(&map, "SelectionBackground") { config.selection_background = Some(s); }
        if let Some(s) = get_string(&map, "SelectionForeground") { config.selection_foreground = Some(s); }

        // Palette (nested record)
        if let Some(Value::Record(palette_map)) = map.get("Palette") {
            let p_map = palette_map.borrow();
            let p = &mut config.palette;

            if let Some(s) = get_string(&p_map, "Black") { p.black = s; }
            if let Some(s) = get_string(&p_map, "Red") { p.red = s; }
            if let Some(s) = get_string(&p_map, "Green") { p.green = s; }
            if let Some(s) = get_string(&p_map, "Yellow") { p.yellow = s; }
            if let Some(s) = get_string(&p_map, "Blue") { p.blue = s; }
            if let Some(s) = get_string(&p_map, "Magenta") { p.magenta = s; }
            if let Some(s) = get_string(&p_map, "Cyan") { p.cyan = s; }
            if let Some(s) = get_string(&p_map, "White") { p.white = s; }

            if let Some(s) = get_string(&p_map, "BrightBlack") { p.bright_black = s; }
            if let Some(s) = get_string(&p_map, "BrightRed") { p.bright_red = s; }
            if let Some(s) = get_string(&p_map, "BrightGreen") { p.bright_green = s; }
            if let Some(s) = get_string(&p_map, "BrightYellow") { p.bright_yellow = s; }
            if let Some(s) = get_string(&p_map, "BrightBlue") { p.bright_blue = s; }
            if let Some(s) = get_string(&p_map, "BrightMagenta") { p.bright_magenta = s; }
            if let Some(s) = get_string(&p_map, "BrightCyan") { p.bright_cyan = s; }
            if let Some(s) = get_string(&p_map, "BrightWhite") { p.bright_white = s; }
        }
    }

    Ok(config)
}

fn extract_keybindings(val: &Value) -> Result<KeyBindings> {
    let mut config = KeyBindings::default();

    if let Value::Record(map) = val {
        let map = map.borrow();
        if let Some(s) = get_string(&map, "LeaderKey") { config.leader_key = s; }

        // Custom bindings
        if let Some(Value::Map(custom_map)) = map.get("Custom") {
            let c_map = custom_map.borrow();
            for (k, v) in c_map.iter() {
                if let Value::Str(cmd) = v {
                    config.custom.insert(k.clone(), cmd.to_string());
                }
            }
        }
    }

    Ok(config)
}

fn extract_ui_config(val: &Value) -> Result<UiConfig> {
    let mut config = UiConfig::default();

    if let Value::Record(map) = val {
        let map = map.borrow();
        if let Some(b) = get_bool(&map, "LinkHints") { config.link_hints = b; }
        if let Some(b) = get_bool(&map, "CommandPalette") { config.command_palette = b; }
        if let Some(b) = get_bool(&map, "Animations") { config.animations = b; }
        if let Some(b) = get_bool(&map, "SmoothScroll") { config.smooth_scroll = b; }
        if let Some(b) = get_bool(&map, "ShowTabs") { config.show_tabs = b; }
        if let Some(b) = get_bool(&map, "CursorBlink") { config.cursor_blink = b; }
        if let Some(i) = get_int(&map, "CursorBlinkInterval") { config.cursor_blink_interval = i as u32; }
        if let Some(s) = get_string(&map, "WindowIcon") { config.window_icon = Some(s); }
    }

    Ok(config)
}

fn extract_plugin_config(val: &Value) -> Result<PluginConfig> {
    let mut config = PluginConfig::default();

    if let Value::Record(map) = val {
        let map = map.borrow();

        if let Some(Value::Tuple(vec)) = map.get("Enabled") {
            for v in vec {
                if let Value::Str(s) = v {
                    config.enabled.push(s.to_string());
                }
            }
        }
    }

    Ok(config)
}

fn extract_session_config(val: &Value) -> Result<SessionConfig> {
    let mut config = SessionConfig::default();

    if let Value::Record(map) = val {
        let map = map.borrow();
        if let Some(b) = get_bool(&map, "RestoreOnStartup") { config.restore_on_startup = b; }
        if let Some(i) = get_int(&map, "AutoSaveInterval") { config.auto_save_interval = i as u32; }
        if let Some(b) = get_bool(&map, "SaveScrollback") { config.save_scrollback = b; }
        if let Some(s) = get_string(&map, "WorkingDirectory") { config.working_directory = Some(s); }
    }

    Ok(config)
}

// --- Helper functions ---

fn get_string(map: &HashMap<String, Value>, key: &str) -> Option<String> {
    map.get(key).and_then(|v| match v {
        Value::Str(s) => Some(s.to_string()),
        _ => None,
    })
}

fn get_int(map: &HashMap<String, Value>, key: &str) -> Option<i64> {
    map.get(key).and_then(|v| match v {
        Value::Int(i) => Some(*i),
        _ => None,
    })
}

fn get_float(map: &HashMap<String, Value>, key: &str) -> Option<f64> {
    map.get(key).and_then(|v| match v {
        Value::Float(f) => Some(*f),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    })
}

fn get_bool(map: &HashMap<String, Value>, key: &str) -> Option<bool> {
    map.get(key).and_then(|v| match v {
        Value::Bool(b) => Some(*b),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_functions() {
        // Test basic Value extraction
        use fusabi_vm::Value;
        use std::collections::HashMap;
        use std::cell::RefCell;
        use std::rc::Rc;

        let mut terminal_map = HashMap::new();
        terminal_map.insert("DefaultShell".to_string(), Value::Str("/bin/zsh".into()));
        terminal_map.insert("ScrollbackLines".to_string(), Value::Int(10000));
        terminal_map.insert("AltScreen".to_string(), Value::Bool(true));

        let val = Value::Record(Rc::new(RefCell::new(terminal_map)));
        let config = extract_terminal_config(&val).unwrap();

        assert_eq!(config.default_shell, "/bin/zsh");
        assert_eq!(config.scrollback_lines, 10000);
        assert_eq!(config.alt_screen, true);
    }
}
