//! Dracula theme
//!
//! Official Dracula theme for terminal emulators.
//! Source: https://draculatheme.com/

use crate::theme::{Theme, ThemeColors, ThemeMetadata, ThemePalette, ThemeVariant};

pub fn theme() -> Theme {
    Theme {
        metadata: ThemeMetadata {
            id: "dracula".to_string(),
            name: "Dracula".to_string(),
            author: "Zeno Rocha".to_string(),
            description: "A dark theme with vibrant purple accents".to_string(),
            variant: ThemeVariant::Dark,
            tags: vec!["dark".to_string(), "purple".to_string(), "popular".to_string()],
            url: Some("https://draculatheme.com".to_string()),
        },
        colors: ThemeColors {
            foreground: "#f8f8f2".to_string(),
            background: "#282a36".to_string(),
            cursor: "#f8f8f2".to_string(),
            cursor_text: Some("#282a36".to_string()),
            selection_background: "#44475a".to_string(),
            selection_foreground: None,
            palette: ThemePalette {
                black: "#21222c".to_string(),
                red: "#ff5555".to_string(),
                green: "#50fa7b".to_string(),
                yellow: "#f1fa8c".to_string(),
                blue: "#bd93f9".to_string(),
                magenta: "#ff79c6".to_string(),
                cyan: "#8be9fd".to_string(),
                white: "#f8f8f2".to_string(),
                bright_black: "#6272a4".to_string(),
                bright_red: "#ff6e6e".to_string(),
                bright_green: "#69ff94".to_string(),
                bright_yellow: "#ffffa5".to_string(),
                bright_blue: "#d6acff".to_string(),
                bright_magenta: "#ff92df".to_string(),
                bright_cyan: "#a4ffff".to_string(),
                bright_white: "#ffffff".to_string(),
            },
            ui: None,
        },
    }
}
