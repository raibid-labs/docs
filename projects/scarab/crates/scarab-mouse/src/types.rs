//! Core types for mouse handling

use serde::{Deserialize, Serialize};

/// Mouse button identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    ScrollUp,
    ScrollDown,
}

/// Mouse event with position and modifiers
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub position: Position,
    pub button: Option<MouseButton>,
    pub modifiers: Modifiers,
}

/// Type of mouse event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseEventKind {
    Press,
    Release,
    Move,
    Scroll,
}

/// Position in terminal grid coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    /// Calculate distance to another position
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = (self.x as f32) - (other.x as f32);
        let dy = (self.y as f32) - (other.y as f32);
        (dx * dx + dy * dy).sqrt()
    }
}

/// Keyboard modifiers held during mouse event
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl Modifiers {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        !self.shift && !self.ctrl && !self.alt && !self.meta
    }
}

/// Click type detected from timing and count
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClickType {
    Single,
    Double,
    Triple,
}

/// Mouse mode - determines who handles mouse events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseMode {
    /// Scarab handles mouse (normal terminal use)
    Normal,
    /// Application handles mouse (vim, tmux, etc)
    Application,
}

impl MouseMode {
    /// Check if application should handle this event
    pub fn is_application(&self) -> bool {
        matches!(self, MouseMode::Application)
    }

    /// Check if Scarab should handle this event
    pub fn is_normal(&self) -> bool {
        matches!(self, MouseMode::Normal)
    }
}

/// Convert bevy mouse button to our type
impl From<bevy::input::mouse::MouseButton> for MouseButton {
    fn from(button: bevy::input::mouse::MouseButton) -> Self {
        match button {
            bevy::input::mouse::MouseButton::Left => MouseButton::Left,
            bevy::input::mouse::MouseButton::Right => MouseButton::Right,
            bevy::input::mouse::MouseButton::Middle => MouseButton::Middle,
            _ => MouseButton::Left, // Default for other buttons
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_distance() {
        let p1 = Position::new(0, 0);
        let p2 = Position::new(3, 4);
        assert_eq!(p1.distance_to(&p2), 5.0);
    }

    #[test]
    fn test_modifiers() {
        let mods = Modifiers::none();
        assert!(mods.is_empty());

        let mods = Modifiers {
            ctrl: true,
            ..Default::default()
        };
        assert!(!mods.is_empty());
    }

    #[test]
    fn test_mouse_mode() {
        let mode = MouseMode::Normal;
        assert!(mode.is_normal());
        assert!(!mode.is_application());

        let mode = MouseMode::Application;
        assert!(!mode.is_normal());
        assert!(mode.is_application());
    }
}
