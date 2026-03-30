//! Tokyo Night theme
//!
//! A clean, dark theme that celebrates the lights of downtown Tokyo at night.
//! Source: https://github.com/enkia/tokyo-night-vscode-theme

use crate::theme::{Theme, ThemeColors, ThemeMetadata, ThemePalette, ThemeVariant};

pub fn theme() -> Theme {
    Theme {
        metadata: ThemeMetadata {
            id: "tokyo-night".to_string(),
            name: "Tokyo Night".to_string(),
            author: "Enkia".to_string(),
            description: "A clean, dark theme inspired by Tokyo at night".to_string(),
            variant: ThemeVariant::Dark,
            tags: vec!["dark".to_string(), "modern".to_string(), "blue".to_string()],
            url: Some("https://github.com/enkia/tokyo-night-vscode-theme".to_string()),
        },
        colors: ThemeColors {
            foreground: "#a9b1d6".to_string(),
            background: "#1a1b26".to_string(),
            cursor: "#c0caf5".to_string(),
            cursor_text: Some("#1a1b26".to_string()),
            selection_background: "#33467c".to_string(),
            selection_foreground: None,
            palette: ThemePalette {
                black: "#15161e".to_string(),
                red: "#f7768e".to_string(),
                green: "#9ece6a".to_string(),
                yellow: "#e0af68".to_string(),
                blue: "#7aa2f7".to_string(),
                magenta: "#bb9af7".to_string(),
                cyan: "#7dcfff".to_string(),
                white: "#a9b1d6".to_string(),
                bright_black: "#414868".to_string(),
                bright_red: "#f7768e".to_string(),
                bright_green: "#9ece6a".to_string(),
                bright_yellow: "#e0af68".to_string(),
                bright_blue: "#7aa2f7".to_string(),
                bright_magenta: "#bb9af7".to_string(),
                bright_cyan: "#7dcfff".to_string(),
                bright_white: "#c0caf5".to_string(),
            },
            ui: None,
        },
    }
}
