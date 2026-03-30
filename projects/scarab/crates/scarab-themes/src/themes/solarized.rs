//! Solarized theme (Light & Dark)
//!
//! The famous Solarized color scheme by Ethan Schoonover.
//! Source: https://ethanschoonover.com/solarized/

use crate::theme::{Theme, ThemeColors, ThemeMetadata, ThemePalette, ThemeVariant};

pub fn light() -> Theme {
    Theme {
        metadata: ThemeMetadata {
            id: "solarized-light".to_string(),
            name: "Solarized Light".to_string(),
            author: "Ethan Schoonover".to_string(),
            description: "Precision colors for machines and people".to_string(),
            variant: ThemeVariant::Light,
            tags: vec!["light".to_string(), "classic".to_string()],
            url: Some("https://ethanschoonover.com/solarized/".to_string()),
        },
        colors: ThemeColors {
            foreground: "#657b83".to_string(),
            background: "#fdf6e3".to_string(),
            cursor: "#657b83".to_string(),
            cursor_text: Some("#fdf6e3".to_string()),
            selection_background: "#eee8d5".to_string(),
            selection_foreground: None,
            palette: ThemePalette {
                black: "#073642".to_string(),
                red: "#dc322f".to_string(),
                green: "#859900".to_string(),
                yellow: "#b58900".to_string(),
                blue: "#268bd2".to_string(),
                magenta: "#d33682".to_string(),
                cyan: "#2aa198".to_string(),
                white: "#eee8d5".to_string(),
                bright_black: "#002b36".to_string(),
                bright_red: "#cb4b16".to_string(),
                bright_green: "#586e75".to_string(),
                bright_yellow: "#657b83".to_string(),
                bright_blue: "#839496".to_string(),
                bright_magenta: "#6c71c4".to_string(),
                bright_cyan: "#93a1a1".to_string(),
                bright_white: "#fdf6e3".to_string(),
            },
            ui: None,
        },
    }
}

pub fn dark() -> Theme {
    Theme {
        metadata: ThemeMetadata {
            id: "solarized-dark".to_string(),
            name: "Solarized Dark".to_string(),
            author: "Ethan Schoonover".to_string(),
            description: "Precision colors for machines and people".to_string(),
            variant: ThemeVariant::Dark,
            tags: vec!["dark".to_string(), "classic".to_string()],
            url: Some("https://ethanschoonover.com/solarized/".to_string()),
        },
        colors: ThemeColors {
            foreground: "#839496".to_string(),
            background: "#002b36".to_string(),
            cursor: "#839496".to_string(),
            cursor_text: Some("#002b36".to_string()),
            selection_background: "#073642".to_string(),
            selection_foreground: None,
            palette: ThemePalette {
                black: "#073642".to_string(),
                red: "#dc322f".to_string(),
                green: "#859900".to_string(),
                yellow: "#b58900".to_string(),
                blue: "#268bd2".to_string(),
                magenta: "#d33682".to_string(),
                cyan: "#2aa198".to_string(),
                white: "#eee8d5".to_string(),
                bright_black: "#002b36".to_string(),
                bright_red: "#cb4b16".to_string(),
                bright_green: "#586e75".to_string(),
                bright_yellow: "#657b83".to_string(),
                bright_blue: "#839496".to_string(),
                bright_magenta: "#6c71c4".to_string(),
                bright_cyan: "#93a1a1".to_string(),
                bright_white: "#fdf6e3".to_string(),
            },
            ui: None,
        },
    }
}
