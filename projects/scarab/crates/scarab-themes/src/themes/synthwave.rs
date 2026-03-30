//! Synthwave '84 theme
//!
//! Retro synthwave cyberpunk theme with neon glow.
//! Inspired by the synthwave aesthetic of the 1980s.

use crate::theme::{Theme, ThemeColors, ThemeMetadata, ThemePalette, ThemeVariant};

pub fn theme() -> Theme {
    Theme {
        metadata: ThemeMetadata {
            id: "synthwave".to_string(),
            name: "Synthwave '84".to_string(),
            author: "Robb Owen".to_string(),
            description: "Retro synthwave cyberpunk with neon glow".to_string(),
            variant: ThemeVariant::Dark,
            tags: vec![
                "dark".to_string(),
                "retro".to_string(),
                "cyberpunk".to_string(),
                "neon".to_string(),
            ],
            url: Some("https://github.com/robb0wen/synthwave-vscode".to_string()),
        },
        colors: ThemeColors {
            foreground: "#f7f7f7".to_string(),
            background: "#262335".to_string(),
            cursor: "#ff7edb".to_string(),
            cursor_text: Some("#262335".to_string()),
            selection_background: "#495495".to_string(),
            selection_foreground: None,
            palette: ThemePalette {
                black: "#241b2f".to_string(),
                red: "#fe4450".to_string(),
                green: "#72f1b8".to_string(),
                yellow: "#fede5d".to_string(),
                blue: "#03edf9".to_string(),
                magenta: "#ff7edb".to_string(),
                cyan: "#b6f4f7".to_string(),
                white: "#f7f7f7".to_string(),
                bright_black: "#495495".to_string(),
                bright_red: "#fe4450".to_string(),
                bright_green: "#72f1b8".to_string(),
                bright_yellow: "#fede5d".to_string(),
                bright_blue: "#03edf9".to_string(),
                bright_magenta: "#ff7edb".to_string(),
                bright_cyan: "#b6f4f7".to_string(),
                bright_white: "#ffffff".to_string(),
            },
            ui: None,
        },
    }
}
