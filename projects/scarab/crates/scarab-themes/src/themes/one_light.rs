//! One Light theme
//!
//! Based on Atom's One Light theme.
//! Source: https://github.com/atom/atom/tree/master/packages/one-light-ui

use crate::theme::{Theme, ThemeColors, ThemeMetadata, ThemePalette, ThemeVariant};

pub fn theme() -> Theme {
    Theme {
        metadata: ThemeMetadata {
            id: "one-light".to_string(),
            name: "One Light".to_string(),
            author: "Atom".to_string(),
            description: "A light UI theme from Atom".to_string(),
            variant: ThemeVariant::Light,
            tags: vec!["light".to_string(), "popular".to_string()],
            url: Some("https://github.com/atom/atom".to_string()),
        },
        colors: ThemeColors {
            foreground: "#383a42".to_string(),
            background: "#fafafa".to_string(),
            cursor: "#526fff".to_string(),
            cursor_text: Some("#fafafa".to_string()),
            selection_background: "#e5e5e6".to_string(),
            selection_foreground: None,
            palette: ThemePalette {
                black: "#383a42".to_string(),
                red: "#e45649".to_string(),
                green: "#50a14f".to_string(),
                yellow: "#c18401".to_string(),
                blue: "#0184bc".to_string(),
                magenta: "#a626a4".to_string(),
                cyan: "#0997b3".to_string(),
                white: "#fafafa".to_string(),
                bright_black: "#4f525e".to_string(),
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
