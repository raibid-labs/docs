//! Nord theme
//!
//! An arctic, north-bluish color palette.
//! Source: https://www.nordtheme.com/

use crate::theme::{Theme, ThemeColors, ThemeMetadata, ThemePalette, ThemeVariant};

pub fn theme() -> Theme {
    Theme {
        metadata: ThemeMetadata {
            id: "nord".to_string(),
            name: "Nord".to_string(),
            author: "Arctic Ice Studio".to_string(),
            description: "An arctic, north-bluish color palette".to_string(),
            variant: ThemeVariant::Dark,
            tags: vec!["dark".to_string(), "blue".to_string(), "minimal".to_string()],
            url: Some("https://www.nordtheme.com".to_string()),
        },
        colors: ThemeColors {
            foreground: "#d8dee9".to_string(),
            background: "#2e3440".to_string(),
            cursor: "#d8dee9".to_string(),
            cursor_text: Some("#2e3440".to_string()),
            selection_background: "#4c566a".to_string(),
            selection_foreground: None,
            palette: ThemePalette {
                black: "#3b4252".to_string(),
                red: "#bf616a".to_string(),
                green: "#a3be8c".to_string(),
                yellow: "#ebcb8b".to_string(),
                blue: "#81a1c1".to_string(),
                magenta: "#b48ead".to_string(),
                cyan: "#88c0d0".to_string(),
                white: "#e5e9f0".to_string(),
                bright_black: "#4c566a".to_string(),
                bright_red: "#bf616a".to_string(),
                bright_green: "#a3be8c".to_string(),
                bright_yellow: "#ebcb8b".to_string(),
                bright_blue: "#81a1c1".to_string(),
                bright_magenta: "#b48ead".to_string(),
                bright_cyan: "#8fbcbb".to_string(),
                bright_white: "#eceff4".to_string(),
            },
            ui: None,
        },
    }
}
