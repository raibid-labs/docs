//! Scripting API - the interface exposed to .fsx scripts
//!
//! This defines what scripts can do:
//! - Access colors, fonts, window properties
//! - Register custom overlays and widgets
//! - React to daemon events
//! - Customize UI elements

use super::error::{ScriptError, ScriptResult};
use bevy::prelude::*;
use std::collections::HashMap;

/// Events that scripts can generate
#[derive(Event, Debug, Clone)]
pub enum ScriptEvent {
    /// Script wants to customize theme colors
    SetColor {
        name: String,
        color: Color,
    },

    /// Script wants to change font
    SetFont {
        family: String,
        size: f32,
    },

    /// Script wants to add a custom overlay
    AddOverlay {
        name: String,
        position: OverlayPosition,
        content: OverlayContent,
    },

    /// Script wants to remove an overlay
    RemoveOverlay {
        name: String,
    },

    /// Script wants to set window title
    SetWindowTitle {
        title: String,
    },

    /// Script wants to register a command
    RegisterCommand {
        name: String,
        description: String,
        keybinding: Option<String>,
    },

    /// Script encountered an error
    Error {
        script_name: String,
        message: String,
    },

    /// Custom event data
    Custom {
        event_type: String,
        data: HashMap<String, ScriptValue>,
    },
}

/// Positions where overlays can be placed
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverlayPosition {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// Content types for overlays
#[derive(Debug, Clone)]
pub enum OverlayContent {
    Text {
        text: String,
        size: f32,
        color: Color,
    },
    Box {
        width: f32,
        height: f32,
        color: Color,
        border_color: Option<Color>,
        border_width: f32,
    },
    Custom {
        widget_type: String,
        properties: HashMap<String, ScriptValue>,
    },
}

/// Values that can be passed between scripts and Rust
#[derive(Debug, Clone)]
pub enum ScriptValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Color(Color),
    List(Vec<ScriptValue>),
    Map(HashMap<String, ScriptValue>),
}

impl ScriptValue {
    pub fn as_bool(&self) -> ScriptResult<bool> {
        match self {
            ScriptValue::Bool(b) => Ok(*b),
            _ => Err(ScriptError::TypeError {
                expected: "bool".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    pub fn as_int(&self) -> ScriptResult<i64> {
        match self {
            ScriptValue::Int(i) => Ok(*i),
            _ => Err(ScriptError::TypeError {
                expected: "int".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    pub fn as_float(&self) -> ScriptResult<f64> {
        match self {
            ScriptValue::Float(f) => Ok(*f),
            ScriptValue::Int(i) => Ok(*i as f64),
            _ => Err(ScriptError::TypeError {
                expected: "float".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    pub fn as_string(&self) -> ScriptResult<String> {
        match self {
            ScriptValue::String(s) => Ok(s.clone()),
            _ => Err(ScriptError::TypeError {
                expected: "string".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    pub fn as_color(&self) -> ScriptResult<Color> {
        match self {
            ScriptValue::Color(c) => Ok(*c),
            _ => Err(ScriptError::TypeError {
                expected: "color".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    fn type_name(&self) -> &str {
        match self {
            ScriptValue::Null => "null",
            ScriptValue::Bool(_) => "bool",
            ScriptValue::Int(_) => "int",
            ScriptValue::Float(_) => "float",
            ScriptValue::String(_) => "string",
            ScriptValue::Color(_) => "color",
            ScriptValue::List(_) => "list",
            ScriptValue::Map(_) => "map",
        }
    }
}

/// Script API context passed to scripts
#[derive(Clone)]
pub struct ScriptContext {
    pub colors: ColorContext,
    pub fonts: FontContext,
    pub window: WindowContext,
    pub terminal: TerminalContext,
}

/// Color-related context
#[derive(Clone)]
pub struct ColorContext {
    pub foreground: Color,
    pub background: Color,
    pub cursor: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub palette: Vec<Color>,
}

/// Font-related context
#[derive(Clone)]
pub struct FontContext {
    pub family: String,
    pub size: f32,
    pub line_height: f32,
}

/// Window-related context
#[derive(Clone)]
pub struct WindowContext {
    pub width: f32,
    pub height: f32,
    pub scale_factor: f32,
    pub title: String,
}

/// Terminal-related context
#[derive(Clone)]
pub struct TerminalContext {
    pub rows: u16,
    pub cols: u16,
    pub scrollback_lines: u32,
}

/// The main script API that scripts can call
pub struct ScriptApi {
    event_sender: crossbeam::channel::Sender<ScriptEvent>,
}

impl ScriptApi {
    pub fn new(event_sender: crossbeam::channel::Sender<ScriptEvent>) -> Self {
        Self { event_sender }
    }

    /// Set a color by name
    pub fn set_color(&self, name: &str, color: Color) -> ScriptResult<()> {
        self.send_event(ScriptEvent::SetColor {
            name: name.to_string(),
            color,
        })
    }

    /// Set font properties
    pub fn set_font(&self, family: &str, size: f32) -> ScriptResult<()> {
        self.send_event(ScriptEvent::SetFont {
            family: family.to_string(),
            size,
        })
    }

    /// Add a custom overlay
    pub fn add_overlay(
        &self,
        name: &str,
        position: OverlayPosition,
        content: OverlayContent,
    ) -> ScriptResult<()> {
        self.send_event(ScriptEvent::AddOverlay {
            name: name.to_string(),
            position,
            content,
        })
    }

    /// Remove an overlay
    pub fn remove_overlay(&self, name: &str) -> ScriptResult<()> {
        self.send_event(ScriptEvent::RemoveOverlay {
            name: name.to_string(),
        })
    }

    /// Set window title
    pub fn set_window_title(&self, title: &str) -> ScriptResult<()> {
        self.send_event(ScriptEvent::SetWindowTitle {
            title: title.to_string(),
        })
    }

    /// Register a custom command
    pub fn register_command(
        &self,
        name: &str,
        description: &str,
        keybinding: Option<&str>,
    ) -> ScriptResult<()> {
        self.send_event(ScriptEvent::RegisterCommand {
            name: name.to_string(),
            description: description.to_string(),
            keybinding: keybinding.map(String::from),
        })
    }

    /// Parse a color from hex string
    pub fn parse_color(hex: &str) -> ScriptResult<Color> {
        let hex = hex.trim_start_matches('#');

        if hex.len() != 6 && hex.len() != 8 {
            return Err(ScriptError::ApiError {
                function: "parse_color".to_string(),
                message: format!("Invalid hex color: #{}", hex),
            });
        }

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ScriptError::ApiError {
            function: "parse_color".to_string(),
            message: format!("Invalid red component: {}", &hex[0..2]),
        })?;

        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ScriptError::ApiError {
            function: "parse_color".to_string(),
            message: format!("Invalid green component: {}", &hex[2..4]),
        })?;

        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ScriptError::ApiError {
            function: "parse_color".to_string(),
            message: format!("Invalid blue component: {}", &hex[4..6]),
        })?;

        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).map_err(|_| ScriptError::ApiError {
                function: "parse_color".to_string(),
                message: format!("Invalid alpha component: {}", &hex[6..8]),
            })?
        } else {
            255
        };

        Ok(Color::srgba_u8(r, g, b, a))
    }

    fn send_event(&self, event: ScriptEvent) -> ScriptResult<()> {
        self.event_sender
            .send(event)
            .map_err(|e| ScriptError::RuntimeError {
                script: "api".to_string(),
                message: format!("Failed to send event: {}", e),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color_rgb() {
        let color = ScriptApi::parse_color("#FF5555").unwrap();
        assert_eq!(color, Color::srgb_u8(255, 85, 85));
    }

    #[test]
    fn test_parse_color_rgba() {
        let color = ScriptApi::parse_color("#FF555580").unwrap();
        assert_eq!(color, Color::srgba_u8(255, 85, 85, 128));
    }

    #[test]
    fn test_parse_color_invalid() {
        assert!(ScriptApi::parse_color("#FFF").is_err());
        assert!(ScriptApi::parse_color("not_a_color").is_err());
    }
}
