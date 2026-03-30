//! Monokai theme
//!
//! The famous Monokai color scheme.
//! Inspired by Sublime Text's default theme.

use crate::theme::{Theme, ThemeColors, ThemeMetadata, ThemePalette, ThemeVariant};

pub fn theme() -> Theme {
    Theme {
        metadata: ThemeMetadata {
            id: "monokai".to_string(),
            name: "Monokai".to_string(),
            author: "Wimer Hazenberg".to_string(),
            description: "A rich, warm dark theme".to_string(),
            variant: ThemeVariant::Dark,
            tags: vec!["dark".to_string(), "warm".to_string(), "classic".to_string()],
            url: Some("https://monokai.pro".to_string()),
        },
        colors: ThemeColors {
            foreground: "#f8f8f2".to_string(),
            background: "#272822".to_string(),
            cursor: "#f8f8f0".to_string(),
            cursor_text: Some("#272822".to_string()),
            selection_background: "#49483e".to_string(),
            selection_foreground: None,
            palette: ThemePalette {
                black: "#272822".to_string(),
                red: "#f92672".to_string(),
                green: "#a6e22e".to_string(),
                yellow: "#f4bf75".to_string(),
                blue: "#66d9ef".to_string(),
                magenta: "#ae81ff".to_string(),
                cyan: "#a1efe4".to_string(),
                white: "#f8f8f2".to_string(),
                bright_black: "#75715e".to_string(),
                bright_red: "#f92672".to_string(),
                bright_green: "#a6e22e".to_string(),
                bright_yellow: "#e6db74".to_string(),
                bright_blue: "#66d9ef".to_string(),
                bright_magenta: "#ae81ff".to_string(),
                bright_cyan: "#a1efe4".to_string(),
                bright_white: "#f9f8f5".to_string(),
            },
            ui: None,
        },
    }
}
