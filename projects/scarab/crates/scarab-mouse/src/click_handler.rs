//! Click detection and handling logic

use crate::types::{ClickType, MouseButton, MouseEvent, MouseEventKind, Position};
use std::time::{Duration, Instant};

/// Double-click threshold in milliseconds
const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(500);

/// Triple-click threshold in milliseconds
const TRIPLE_CLICK_THRESHOLD: Duration = Duration::from_millis(500);

/// Maximum pixel distance for multi-clicks
const MULTI_CLICK_DISTANCE: f32 = 3.0;

/// Click detector state machine
pub struct ClickDetector {
    last_click: Option<ClickState>,
}

#[derive(Debug, Clone)]
struct ClickState {
    position: Position,
    time: Instant,
    count: u8,
}

impl Default for ClickDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ClickDetector {
    pub fn new() -> Self {
        Self { last_click: None }
    }

    /// Process a mouse press event and detect click type
    pub fn handle_press(&mut self, event: &MouseEvent) -> ClickType {
        let now = Instant::now();

        if event.kind != MouseEventKind::Press {
            return ClickType::Single;
        }

        let click_type = if let Some(last) = &self.last_click {
            // Check if this is a multi-click
            let time_delta = now.duration_since(last.time);
            let distance = last.position.distance_to(&event.position);

            if distance <= MULTI_CLICK_DISTANCE {
                if time_delta <= DOUBLE_CLICK_THRESHOLD && last.count == 1 {
                    ClickType::Double
                } else if time_delta <= TRIPLE_CLICK_THRESHOLD && last.count == 2 {
                    ClickType::Triple
                } else {
                    ClickType::Single
                }
            } else {
                ClickType::Single
            }
        } else {
            ClickType::Single
        };

        // Update state
        let count = match click_type {
            ClickType::Single => 1,
            ClickType::Double => 2,
            ClickType::Triple => 3,
        };

        self.last_click = Some(ClickState {
            position: event.position,
            time: now,
            count,
        });

        click_type
    }

    /// Reset click state (e.g., after a drag or long delay)
    pub fn reset(&mut self) {
        self.last_click = None;
    }
}

/// Generate escape sequences for terminal cursor positioning
pub fn generate_cursor_position_sequence(pos: Position) -> Vec<u8> {
    // Move cursor to position using escape sequence
    // ESC [ row ; col H
    // Note: Terminal coordinates are 1-based
    format!("\x1b[{};{}H", pos.y + 1, pos.x + 1).into_bytes()
}

/// Generate SGR mouse event sequence for application mode
pub fn generate_mouse_sequence(event: &MouseEvent) -> Option<Vec<u8>> {
    // SGR format: CSI < button ; x ; y M/m
    // M = press, m = release
    let button_code = match event.button? {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
        MouseButton::ScrollUp => 64,
        MouseButton::ScrollDown => 65,
    };

    let mut code = button_code;

    // Add modifier flags
    if event.modifiers.shift {
        code += 4;
    }
    if event.modifiers.alt {
        code += 8;
    }
    if event.modifiers.ctrl {
        code += 16;
    }

    let action = match event.kind {
        MouseEventKind::Press => 'M',
        MouseEventKind::Release => 'm',
        MouseEventKind::Move => 'M', // Move events use same as press
        MouseEventKind::Scroll => 'M',
    };

    // Terminal coordinates are 1-based
    Some(format!(
        "\x1b[<{};{};{}{}",
        code,
        event.position.x + 1,
        event.position.y + 1,
        action
    ).into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Modifiers;

    #[test]
    fn test_single_click() {
        let mut detector = ClickDetector::new();
        let event = MouseEvent {
            kind: MouseEventKind::Press,
            position: Position::new(5, 5),
            button: Some(MouseButton::Left),
            modifiers: Modifiers::none(),
        };

        assert_eq!(detector.handle_press(&event), ClickType::Single);
    }

    #[test]
    fn test_double_click() {
        let mut detector = ClickDetector::new();
        let pos = Position::new(5, 5);
        let event = MouseEvent {
            kind: MouseEventKind::Press,
            position: pos,
            button: Some(MouseButton::Left),
            modifiers: Modifiers::none(),
        };

        // First click
        assert_eq!(detector.handle_press(&event), ClickType::Single);

        // Second click at same position
        assert_eq!(detector.handle_press(&event), ClickType::Double);
    }

    #[test]
    fn test_triple_click() {
        let mut detector = ClickDetector::new();
        let pos = Position::new(5, 5);
        let event = MouseEvent {
            kind: MouseEventKind::Press,
            position: pos,
            button: Some(MouseButton::Left),
            modifiers: Modifiers::none(),
        };

        // Three clicks in succession
        assert_eq!(detector.handle_press(&event), ClickType::Single);
        assert_eq!(detector.handle_press(&event), ClickType::Double);
        assert_eq!(detector.handle_press(&event), ClickType::Triple);
    }

    #[test]
    fn test_cursor_position_sequence() {
        let seq = generate_cursor_position_sequence(Position::new(10, 20));
        assert_eq!(seq, b"\x1b[21;11H"); // +1 for 1-based indexing
    }

    #[test]
    fn test_mouse_sequence_left_click() {
        let event = MouseEvent {
            kind: MouseEventKind::Press,
            position: Position::new(5, 10),
            button: Some(MouseButton::Left),
            modifiers: Modifiers::none(),
        };

        let seq = generate_mouse_sequence(&event).unwrap();
        assert_eq!(seq, b"\x1b[<0;6;11M");
    }

    #[test]
    fn test_mouse_sequence_with_modifiers() {
        let event = MouseEvent {
            kind: MouseEventKind::Press,
            position: Position::new(5, 10),
            button: Some(MouseButton::Left),
            modifiers: Modifiers {
                ctrl: true,
                ..Default::default()
            },
        };

        let seq = generate_mouse_sequence(&event).unwrap();
        // Ctrl adds 16 to button code
        assert_eq!(seq, b"\x1b[<16;6;11M");
    }
}
