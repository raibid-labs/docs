//! Gruvbox theme (Light & Dark)
//!
//! Gruvbox is a retro groove color scheme.
//! Source: https://github.com/morhetz/gruvbox

use crate::theme::{Theme, ThemeColors, ThemeMetadata, ThemePalette, ThemeVariant};

pub fn dark() -> Theme {
    Theme {
        metadata: ThemeMetadata {
            id: "gruvbox-dark".to_string(),
            name: "Gruvbox Dark".to_string(),
            author: "Pavel Pertsev".to_string(),
            description: "Retro groove color scheme - dark variant".to_string(),
            variant: ThemeVariant::Dark,
            tags: vec!["dark".to_string(), "retro".to_string(), "warm".to_string()],
            url: Some("https://github.com/morhetz/gruvbox".to_string()),
        },
        colors: ThemeColors {
            foreground: "#ebdbb2".to_string(),
            background: "#282828".to_string(),
            cursor: "#ebdbb2".to_string(),
            cursor_text: Some("#282828".to_string()),
            selection_background: "#504945".to_string(),
            selection_foreground: None,
            palette: ThemePalette {
                black: "#282828".to_string(),
                red: "#cc241d".to_string(),
                green: "#98971a".to_string(),
                yellow: "#d79921".to_string(),
                blue: "#458588".to_string(),
                magenta: "#b16286".to_string(),
                cyan: "#689d6a".to_string(),
                white: "#a89984".to_string(),
                bright_black: "#928374".to_string(),
                bright_red: "#fb4934".to_string(),
                bright_green: "#b8bb26".to_string(),
                bright_yellow: "#fabd2f".to_string(),
                bright_blue: "#83a598".to_string(),
                bright_magenta: "#d3869b".to_string(),
                bright_cyan: "#8ec07c".to_string(),
                bright_white: "#ebdbb2".to_string(),
            },
            ui: None,
        },
    }
}

pub fn light() -> Theme {
    Theme {
        metadata: ThemeMetadata {
            id: "gruvbox-light".to_string(),
            name: "Gruvbox Light".to_string(),
            author: "Pavel Pertsev".to_string(),
            description: "Retro groove color scheme - light variant".to_string(),
            variant: ThemeVariant::Light,
            tags: vec!["light".to_string(), "retro".to_string(), "warm".to_string()],
            url: Some("https://github.com/morhetz/gruvbox".to_string()),
        },
        colors: ThemeColors {
            foreground: "#3c3836".to_string(),
            background: "#fbf1c7".to_string(),
            cursor: "#3c3836".to_string(),
            cursor_text: Some("#fbf1c7".to_string()),
            selection_background: "#ebdbb2".to_string(),
            selection_foreground: None,
            palette: ThemePalette {
                black: "#fbf1c7".to_string(),
                red: "#cc241d".to_string(),
                green: "#98971a".to_string(),
                yellow: "#d79921".to_string(),
                blue: "#458588".to_string(),
                magenta: "#b16286".to_string(),
                cyan: "#689d6a".to_string(),
                white: "#7c6f64".to_string(),
                bright_black: "#928374".to_string(),
                bright_red: "#9d0006".to_string(),
                bright_green: "#79740e".to_string(),
                bright_yellow: "#b57614".to_string(),
                bright_blue: "#076678".to_string(),
                bright_magenta: "#8f3f71".to_string(),
                bright_cyan: "#427b58".to_string(),
                bright_white: "#3c3836".to_string(),
            },
            ui: None,
        },
    }
}
