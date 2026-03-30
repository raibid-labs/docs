// Configurable key bindings system
// Allows users to customize keyboard shortcuts

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use std::collections::HashMap;

/// Plugin for key bindings functionality
pub struct KeybindingsPlugin;

impl Plugin for KeybindingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KeyBindingConfig>()
            .add_event::<KeyBindingTriggeredEvent>()
            .add_systems(Update, handle_keybindings_system);
    }
}

/// A single key binding
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
}

impl KeyBinding {
    pub fn new(key: KeyCode) -> Self {
        Self {
            key,
            ctrl: false,
            alt: false,
            shift: false,
            super_key: false,
        }
    }

    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn with_super(mut self) -> Self {
        self.super_key = true;
        self
    }

    pub fn matches(&self, keyboard: &ButtonInput<KeyCode>) -> bool {
        if !keyboard.pressed(self.key) {
            return false;
        }

        let ctrl_pressed = keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
        let alt_pressed = keyboard.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
        let shift_pressed = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        let super_pressed = keyboard.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);

        self.ctrl == ctrl_pressed
            && self.alt == alt_pressed
            && self.shift == shift_pressed
            && self.super_key == super_pressed
    }

    pub fn to_string(&self) -> String {
        let mut parts = Vec::new();

        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.super_key {
            parts.push("Super");
        }

        let key_str = format!("{:?}", self.key);
        parts.push(&key_str);

        parts.join("+")
    }

    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('+').collect();
        if parts.is_empty() {
            return None;
        }

        let mut binding = KeyBinding::new(KeyCode::KeyA); // Default, will be overwritten
        let mut key_set = false;

        for part in parts {
            match part {
                "Ctrl" => binding.ctrl = true,
                "Alt" => binding.alt = true,
                "Shift" => binding.shift = true,
                "Super" => binding.super_key = true,
                key_str => {
                    // Parse key code
                    if let Some(key) = parse_keycode(key_str) {
                        binding.key = key;
                        key_set = true;
                    }
                }
            }
        }

        if key_set {
            Some(binding)
        } else {
            None
        }
    }
}

/// Configuration for all key bindings
#[derive(Resource)]
pub struct KeyBindingConfig {
    bindings: HashMap<KeyBinding, String>,
}

impl Default for KeyBindingConfig {
    fn default() -> Self {
        let mut config = Self {
            bindings: HashMap::new(),
        };
        config.register_defaults();
        config
    }
}

impl KeyBindingConfig {
    pub fn bind(&mut self, binding: KeyBinding, action: &str) {
        self.bindings.insert(binding, action.to_string());
    }

    pub fn unbind(&mut self, binding: &KeyBinding) {
        self.bindings.remove(binding);
    }

    pub fn get_action(&self, binding: &KeyBinding) -> Option<&str> {
        self.bindings.get(binding).map(|s| s.as_str())
    }

    pub fn find_binding(&self, action: &str) -> Option<&KeyBinding> {
        self.bindings
            .iter()
            .find(|(_, a)| a.as_str() == action)
            .map(|(k, _)| k)
    }

    pub fn all_bindings(&self) -> &HashMap<KeyBinding, String> {
        &self.bindings
    }

    fn register_defaults(&mut self) {
        // Copy/Paste
        self.bind(KeyBinding::new(KeyCode::KeyC).with_ctrl(), "edit.copy");
        self.bind(KeyBinding::new(KeyCode::KeyV).with_ctrl(), "edit.paste");
        self.bind(KeyBinding::new(KeyCode::KeyX).with_ctrl(), "edit.cut");

        // Undo/Redo
        self.bind(KeyBinding::new(KeyCode::KeyZ).with_ctrl(), "edit.undo");
        self.bind(KeyBinding::new(KeyCode::KeyY).with_ctrl(), "edit.redo");

        // Search
        self.bind(KeyBinding::new(KeyCode::KeyF).with_ctrl(), "search.find");
        self.bind(KeyBinding::new(KeyCode::KeyH).with_ctrl(), "search.replace");

        // Terminal operations
        self.bind(KeyBinding::new(KeyCode::KeyL).with_ctrl(), "terminal.clear");
        self.bind(
            KeyBinding::new(KeyCode::KeyT).with_ctrl(),
            "terminal.new_tab",
        );
        self.bind(
            KeyBinding::new(KeyCode::KeyW).with_ctrl(),
            "terminal.close_tab",
        );

        // Window management
        self.bind(
            KeyBinding::new(KeyCode::Backslash).with_ctrl(),
            "window.split_vertical",
        );
        self.bind(
            KeyBinding::new(KeyCode::Minus).with_ctrl(),
            "window.split_horizontal",
        );

        // Navigation
        self.bind(
            KeyBinding::new(KeyCode::Tab).with_ctrl(),
            "navigation.next_pane",
        );
        self.bind(
            KeyBinding::new(KeyCode::Tab).with_ctrl().with_shift(),
            "navigation.prev_pane",
        );

        // Command palette
        self.bind(KeyBinding::new(KeyCode::KeyP).with_ctrl(), "palette.open");

        // Link hints
        self.bind(
            KeyBinding::new(KeyCode::KeyK).with_ctrl(),
            "links.show_hints",
        );
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;

        for (binding, action) in &self.bindings {
            writeln!(file, "{}={}", binding.to_string(), action)?;
        }

        Ok(())
    }

    pub fn load_from_file(&mut self, path: &std::path::Path) -> Result<(), std::io::Error> {
        use std::io::BufRead;
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some((binding_str, action)) = line.split_once('=') {
                if let Some(binding) = KeyBinding::from_string(binding_str) {
                    self.bind(binding, action);
                }
            }
        }

        Ok(())
    }
}

/// Event fired when key binding is triggered
#[derive(Event)]
pub struct KeyBindingTriggeredEvent {
    pub action: String,
}

/// Handle key binding inputs
fn handle_keybindings_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<KeyBindingConfig>,
    mut event_writer: EventWriter<KeyBindingTriggeredEvent>,
) {
    // Check all registered bindings
    for (binding, action) in config.all_bindings() {
        if keyboard.just_pressed(binding.key) && binding.matches(&keyboard) {
            info!(
                "Key binding triggered: {} -> {}",
                binding.to_string(),
                action
            );
            event_writer.send(KeyBindingTriggeredEvent {
                action: action.clone(),
            });
        }
    }
}

/// Parse string to KeyCode
fn parse_keycode(s: &str) -> Option<KeyCode> {
    match s {
        "KeyA" => Some(KeyCode::KeyA),
        "KeyB" => Some(KeyCode::KeyB),
        "KeyC" => Some(KeyCode::KeyC),
        "KeyD" => Some(KeyCode::KeyD),
        "KeyE" => Some(KeyCode::KeyE),
        "KeyF" => Some(KeyCode::KeyF),
        "KeyG" => Some(KeyCode::KeyG),
        "KeyH" => Some(KeyCode::KeyH),
        "KeyI" => Some(KeyCode::KeyI),
        "KeyJ" => Some(KeyCode::KeyJ),
        "KeyK" => Some(KeyCode::KeyK),
        "KeyL" => Some(KeyCode::KeyL),
        "KeyM" => Some(KeyCode::KeyM),
        "KeyN" => Some(KeyCode::KeyN),
        "KeyO" => Some(KeyCode::KeyO),
        "KeyP" => Some(KeyCode::KeyP),
        "KeyQ" => Some(KeyCode::KeyQ),
        "KeyR" => Some(KeyCode::KeyR),
        "KeyS" => Some(KeyCode::KeyS),
        "KeyT" => Some(KeyCode::KeyT),
        "KeyU" => Some(KeyCode::KeyU),
        "KeyV" => Some(KeyCode::KeyV),
        "KeyW" => Some(KeyCode::KeyW),
        "KeyX" => Some(KeyCode::KeyX),
        "KeyY" => Some(KeyCode::KeyY),
        "KeyZ" => Some(KeyCode::KeyZ),
        "Escape" => Some(KeyCode::Escape),
        "Enter" => Some(KeyCode::Enter),
        "Space" => Some(KeyCode::Space),
        "Backspace" => Some(KeyCode::Backspace),
        "Tab" => Some(KeyCode::Tab),
        "Minus" => Some(KeyCode::Minus),
        "Backslash" => Some(KeyCode::Backslash),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_binding_string_conversion() {
        let binding = KeyBinding::new(KeyCode::KeyC).with_ctrl();
        assert_eq!(binding.to_string(), "Ctrl+KeyC");

        let parsed = KeyBinding::from_string("Ctrl+KeyC").unwrap();
        assert_eq!(parsed, binding);
    }

    #[test]
    fn test_keybinding_config() {
        let mut config = KeyBindingConfig::default();

        let binding = KeyBinding::new(KeyCode::KeyS).with_ctrl();
        config.bind(binding.clone(), "test.save");

        assert_eq!(config.get_action(&binding), Some("test.save"));
        assert_eq!(config.find_binding("test.save").unwrap(), &binding);
    }
}
