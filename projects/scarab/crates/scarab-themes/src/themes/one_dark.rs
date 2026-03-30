//! One Dark theme
//!
//! Based on Atom's One Dark theme.
//! Source: https://github.com/atom/atom/tree/master/packages/one-dark-ui

use crate::theme::{Theme, ThemeColors, ThemeMetadata, ThemePalette, ThemeVariant};

pub fn theme() -> Theme {
    Theme {
        metadata: ThemeMetadata {
            id: "one-dark".to_string(),
            name: "One Dark".to_string(),
            author: "Atom".to_string(),
            description: "A dark UI theme from Atom".to_string(),
            variant: ThemeVariant::Dark,
            tags: vec!["dark".to_string(), "popular".to_string()],
            url: Some("https://github.com/atom/atom".to_string()),
        },
        colors: ThemeColors {
            foreground: "#abb2bf".to_string(),
            background: "#282c34".to_string(),
            cursor: "#528bff".to_string(),
            cursor_text: Some("#282c34".to_string()),
            selection_background: "#3e4451".to_string(),
            selection_foreground: None,
            palette: ThemePalette {
                black: "#282c34".to_string(),
                red: "#e06c75".to_string(),
                green: "#98c379".to_string(),
                yellow: "#e5c07b".to_string(),
                blue: "#61afef".to_string(),
                magenta: "#c678dd".to_string(),
                cyan: "#56b6c2".to_string(),
                white: "#abb2bf".to_string(),
                bright_black: "#5c6370".to_string(),
                bright_red: "#e06c75".to_string(),
                bright_green: "#98c379".to_string(),
                bright_yellow: "#e5c07b".to_string(),
                bright_blue: "#61afef".to_string(),
                bright_magenta: "#c678dd".to_string(),
                bright_cyan: "#56b6c2".to_string(),
                bright_white: "#ffffff".to_string(),
            },
            ui: None,
        },
    }
}
