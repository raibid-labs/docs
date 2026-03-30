//! Fusabi-based configuration loader
//!
//! Loads configuration from `.fsx` files using the Fusabi frontend.
//! This replaces the TOML-based configuration system with a programmable
//! F# DSL that allows dynamic configuration, hooks, and validation.

use crate::{config::*, error::*};
use fusabi_frontend::{Lexer, Parser, Compiler};
use fusabi_vm::{Value, Vm};
use std::collections::HashMap;
use std::path::Path;

/// Fusabi configuration loader
pub struct FusabiConfigLoader;

impl FusabiConfigLoader {
    /// Load configuration from a Fusabi script file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<ScarabConfig> {
        let source = std::fs::read_to_string(path.as_ref())?;
        Self::from_source(&source)
    }

    /// Load configuration from Fusabi source code
    pub fn from_source(source: &str) -> Result<ScarabConfig> {
        // Compile the Fusabi source manually to ensure proper program structure
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|e| {
            ConfigError::FusabiCompileError(format!("Lexer error: {:?}", e))
        })?;

        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().map_err(|e| {
            ConfigError::FusabiCompileError(format!("Parser error: {:?}", e))
        })?;

        let chunk = Compiler::compile_program(&program).map_err(|e| {
            ConfigError::FusabiCompileError(format!("Compiler error: {:?}", e))
        })?;

        // Execute the compiled config
        let mut vm = Vm::new();
        let result = vm.execute(chunk).map_err(|e| {
            ConfigError::FusabiRuntimeError(format!("Failed to execute config: {:?}", e))
        })?;

        let module = FusabiModule { vm, result };
        let mut config = ScarabConfig::default();

        // Extract configuration sections
        // We use a best-effort approach: if a section is defined, we use it;
        // otherwise we keep the default.
        if let Ok(c) = Self::extract_terminal_config(&module) {
            config.terminal = c;
        }
        if let Ok(c) = Self::extract_font_config(&module) {
            config.font = c;
        }
        if let Ok(c) = Self::extract_color_config(&module) {
            config.colors = c;
        }
        if let Ok(c) = Self::extract_keybindings(&module) {
            config.keybindings = c;
        }
        if let Ok(c) = Self::extract_ui_config(&module) {
            config.ui = c;
        }
        if let Ok(c) = Self::extract_plugin_config(&module) {
            config.plugins = c;
        }
        if let Ok(c) = Self::extract_session_config(&module) {
            config.sessions = c;
        }

        println!("✅ Fusabi config loaded successfully");
        Ok(config)
    }

    /// Load configuration with fallback chain:
    /// 1. Try ~/.config/scarab/config.fsx
    /// 2. Fall back to default config
    pub fn load_with_fallback() -> Result<ScarabConfig> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_path = std::path::PathBuf::from(home)
            .join(".config/scarab/config.fsx");

        if config_path.exists() {
            println!("Loading Fusabi config from: {}", config_path.display());
            Self::from_file(config_path)
        } else {
            println!("No Fusabi config found, using defaults");
            println!("Create {} to customize your terminal", config_path.display());
            Ok(ScarabConfig::default())
        }
    }

    /// Extract terminal config from compiled Fusabi module
    fn extract_terminal_config(module: &FusabiModule) -> Result<TerminalConfig> {
        // Debug: print available globals
        // println!("Globals: {:?}", module.vm.globals.keys());
        // println!("Result: {:?}", module.result);

        let val = match module.get_global("terminal") {
            Some(v) => v,
            None => return Ok(TerminalConfig::default()),
        };

        // println!("Terminal value: {:?}", val);

        let mut config = TerminalConfig::default();

        if let Value::Record(map) = val {
            let map = map.borrow();
            // println!("Terminal map keys: {:?}", map.keys());
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

    /// Extract font config from compiled Fusabi module
    fn extract_font_config(module: &FusabiModule) -> Result<FontConfig> {
        let val = match module.get_global("font") {
            Some(v) => v,
            None => return Ok(FontConfig::default()),
        };

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

    /// Extract color config from compiled Fusabi module
    fn extract_color_config(module: &FusabiModule) -> Result<ColorConfig> {
        let val = match module.get_global("colors") {
            Some(v) => v,
            None => return Ok(ColorConfig::default()),
        };

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

    /// Extract keybindings from compiled Fusabi module
    fn extract_keybindings(module: &FusabiModule) -> Result<KeyBindings> {
        let val = match module.get_global("keybindings") {
            Some(v) => v,
            None => return Ok(KeyBindings::default()),
        };
        
        let mut config = KeyBindings::default();
        
        if let Value::Record(map) = val {
            let map = map.borrow();
            if let Some(s) = get_string(&map, "LeaderKey") { config.leader_key = s; }
            // ... other bindings ...
            
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

    /// Extract UI config from compiled Fusabi module
    fn extract_ui_config(module: &FusabiModule) -> Result<UiConfig> {
        let val = match module.get_global("ui") {
            Some(v) => v,
            None => return Ok(UiConfig::default()),
        };
        
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
            
            // Enums would need string parsing or integer mapping
            // For now, skip enums to keep it simple
        }
        
        Ok(config)
    }

    /// Extract plugin config from compiled Fusabi module
    fn extract_plugin_config(module: &FusabiModule) -> Result<PluginConfig> {
        let val = match module.get_global("plugins") {
             Some(v) => v,
             None => return Ok(PluginConfig::default()),
        };
        
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
            
            // Plugin specific config (Map<String, Record>)
            // This is complex because we need to convert Value to serde_json::Value
            // Skipping for now
        }
        
        Ok(config)
    }

    /// Extract session config from compiled Fusabi module
    fn extract_session_config(module: &FusabiModule) -> Result<SessionConfig> {
        let val = match module.get_global("sessions") {
            Some(v) => v,
            None => return Ok(SessionConfig::default()),
        };
        
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
}

/// Wrapper for Fusabi VM
struct FusabiModule {
    vm: Vm,
    result: Value,
}

impl FusabiModule {
    fn get_global(&self, name: &str) -> Option<Value> {
        // First check VM globals
        if let Some(v) = self.vm.globals.get(name) {
            return Some(v.clone());
        }
        
        // Then check if the result is a Record containing the name
        // (e.g. if the script returns a module object)
        if let Value::Record(map) = &self.result {
             if let Some(v) = map.borrow().get(name) {
                 return Some(v.clone());
             }
        }
        
        None
    }
}

// --- Helpers ---

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
        // Handle conversion from Int if needed?
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
    fn test_compile_simple_config() {
        // Note: Syntax depends on what fusabi-frontend supports
        // Assuming it supports records { key = val }
        let source = r#"
let x = 42

let terminal = {
    DefaultShell = "/bin/zsh";
    ScrollbackLines = 12345;
    AltScreen = true;
    Columns = 120
}

let font = {
    Family = "Hack";
    Size = 16.5;
    Fallback = ("Arial", "Symbola")
}

{
    terminal = terminal;
    font = font
}
"#;

        let config = FusabiConfigLoader::from_source(source);
        if let Err(e) = &config {
            println!("Error loading config: {:?}", e);
        }
        assert!(config.is_ok(), "Config should compile successfully");
        let config = config.unwrap();
        
        assert_eq!(config.terminal.default_shell, "/bin/zsh");
        assert_eq!(config.terminal.scrollback_lines, 12345);
        assert_eq!(config.terminal.columns, 120);
        
        assert_eq!(config.font.family, "Hack");
        assert_eq!(config.font.size, 16.5);
        // Tuples in F# might compile to Value::Tuple or Value::Array depending on version
        // Our extractor expects Tuple
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = FusabiConfigLoader::from_file("/nonexistent/config.fsx");
        assert!(result.is_err(), "Should fail on nonexistent file");
    }

    #[test]
    fn test_load_with_fallback() {
        let config = FusabiConfigLoader::load_with_fallback();
        assert!(config.is_ok(), "Fallback should always succeed");
    }
}