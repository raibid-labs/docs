//! Catppuccin theme
//!
//! Soothing pastel theme for the high-spirited.
//! Source: https://github.com/catppuccin/catppuccin

use crate::theme::{Theme, ThemeColors, ThemeMetadata, ThemePalette, ThemeVariant};

pub fn mocha() -> Theme {
    Theme {
        metadata: ThemeMetadata {
            id: "catppuccin-mocha".to_string(),
            name: "Catppuccin Mocha".to_string(),
            author: "Catppuccin".to_string(),
            description: "Soothing pastel theme - dark variant".to_string(),
            variant: ThemeVariant::Dark,
            tags: vec!["dark".to_string(), "pastel".to_string(), "modern".to_string()],
            url: Some("https://github.com/catppuccin/catppuccin".to_string()),
        },
        colors: ThemeColors {
            foreground: "#cdd6f4".to_string(),
            background: "#1e1e2e".to_string(),
            cursor: "#f5e0dc".to_string(),
            cursor_text: Some("#1e1e2e".to_string()),
            selection_background: "#585b70".to_string(),
            selection_foreground: None,
            palette: ThemePalette {
                black: "#45475a".to_string(),
                red: "#f38ba8".to_string(),
                green: "#a6e3a1".to_string(),
                yellow: "#f9e2af".to_string(),
                blue: "#89b4fa".to_string(),
                magenta: "#f5c2e7".to_string(),
                cyan: "#94e2d5".to_string(),
                white: "#bac2de".to_string(),
                bright_black: "#585b70".to_string(),
                bright_red: "#f38ba8".to_string(),
                bright_green: "#a6e3a1".to_string(),
                bright_yellow: "#f9e2af".to_string(),
                bright_blue: "#89b4fa".to_string(),
                bright_magenta: "#f5c2e7".to_string(),
                bright_cyan: "#94e2d5".to_string(),
                bright_white: "#a6adc8".to_string(),
            },
            ui: None,
        },
    }
}

pub fn latte() -> Theme {
    Theme {
        metadata: ThemeMetadata {
            id: "catppuccin-latte".to_string(),
            name: "Catppuccin Latte".to_string(),
            author: "Catppuccin".to_string(),
            description: "Soothing pastel theme - light variant".to_string(),
            variant: ThemeVariant::Light,
            tags: vec!["light".to_string(), "pastel".to_string(), "modern".to_string()],
            url: Some("https://github.com/catppuccin/catppuccin".to_string()),
        },
        colors: ThemeColors {
            foreground: "#4c4f69".to_string(),
            background: "#eff1f5".to_string(),
            cursor: "#dc8a78".to_string(),
            cursor_text: Some("#eff1f5".to_string()),
            selection_background: "#acb0be".to_string(),
            selection_foreground: None,
            palette: ThemePalette {
                black: "#5c5f77".to_string(),
                red: "#d20f39".to_string(),
                green: "#40a02b".to_string(),
                yellow: "#df8e1d".to_string(),
                blue: "#1e66f5".to_string(),
                magenta: "#ea76cb".to_string(),
                cyan: "#179299".to_string(),
                white: "#acb0be".to_string(),
                bright_black: "#6c6f85".to_string(),
                bright_red: "#d20f39".to_string(),
                bright_green: "#40a02b".to_string(),
                bright_yellow: "#df8e1d".to_string(),
                bright_blue: "#1e66f5".to_string(),
                bright_magenta: "#ea76cb".to_string(),
                bright_cyan: "#179299".to_string(),
                bright_white: "#bcc0cc".to_string(),
            },
            ui: None,
        },
    }
}
