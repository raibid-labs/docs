//! Core theme data structures

use serde::{Deserialize, Serialize};

/// Complete theme definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    /// Theme metadata
    pub metadata: ThemeMetadata,

    /// Color scheme
    pub colors: ThemeColors,
}

/// Theme metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeMetadata {
    /// Unique theme identifier (lowercase, dash-separated)
    pub id: String,

    /// Display name
    pub name: String,

    /// Theme author
    pub author: String,

    /// Description
    pub description: String,

    /// Light or dark theme
    pub variant: ThemeVariant,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Homepage or source URL
    #[serde(default)]
    pub url: Option<String>,
}

/// Theme variant (light/dark)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeVariant {
    Light,
    Dark,
}

/// Complete color scheme for a theme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeColors {
    /// Foreground (default text color)
    pub foreground: String,

    /// Background (default background)
    pub background: String,

    /// Cursor color
    pub cursor: String,

    /// Cursor text color (optional, defaults to background)
    #[serde(default)]
    pub cursor_text: Option<String>,

    /// Selection background
    pub selection_background: String,

    /// Selection foreground (optional, defaults to foreground)
    #[serde(default)]
    pub selection_foreground: Option<String>,

    /// ANSI color palette (16 colors)
    pub palette: ThemePalette,

    /// Additional UI colors (optional)
    #[serde(default)]
    pub ui: Option<UiColors>,
}

/// 16-color ANSI palette
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemePalette {
    // Normal colors (0-7)
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,

    // Bright colors (8-15)
    pub bright_black: String,
    pub bright_red: String,
    pub bright_green: String,
    pub bright_yellow: String,
    pub bright_blue: String,
    pub bright_magenta: String,
    pub bright_cyan: String,
    pub bright_white: String,
}

/// Additional UI-specific colors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiColors {
    /// Tab bar background
    #[serde(default)]
    pub tab_bar_background: Option<String>,

    /// Active tab background
    #[serde(default)]
    pub active_tab_background: Option<String>,

    /// Inactive tab background
    #[serde(default)]
    pub inactive_tab_background: Option<String>,

    /// Border color
    #[serde(default)]
    pub border: Option<String>,

    /// Search match highlight
    #[serde(default)]
    pub search_match: Option<String>,
}

impl Theme {
    /// Create a new theme
    pub fn new(metadata: ThemeMetadata, colors: ThemeColors) -> Self {
        Self { metadata, colors }
    }

    /// Get theme ID
    pub fn id(&self) -> &str {
        &self.metadata.id
    }

    /// Get theme display name
    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    /// Check if theme is dark variant
    pub fn is_dark(&self) -> bool {
        self.metadata.variant == ThemeVariant::Dark
    }

    /// Check if theme is light variant
    pub fn is_light(&self) -> bool {
        self.metadata.variant == ThemeVariant::Light
    }

    /// Convert to ColorConfig for application
    pub fn to_color_config(&self) -> scarab_config::ColorConfig {
        scarab_config::ColorConfig {
            theme: Some(self.metadata.id.clone()),
            foreground: Some(self.colors.foreground.clone()),
            background: Some(self.colors.background.clone()),
            cursor: Some(self.colors.cursor.clone()),
            selection_background: Some(self.colors.selection_background.clone()),
            selection_foreground: self.colors.selection_foreground.clone(),
            palette: self.to_color_palette(),
            opacity: 1.0,
            dim_opacity: 0.7,
        }
    }

    /// Convert palette to scarab_config::ColorPalette
    fn to_color_palette(&self) -> scarab_config::ColorPalette {
        scarab_config::ColorPalette {
            black: self.colors.palette.black.clone(),
            red: self.colors.palette.red.clone(),
            green: self.colors.palette.green.clone(),
            yellow: self.colors.palette.yellow.clone(),
            blue: self.colors.palette.blue.clone(),
            magenta: self.colors.palette.magenta.clone(),
            cyan: self.colors.palette.cyan.clone(),
            white: self.colors.palette.white.clone(),
            bright_black: self.colors.palette.bright_black.clone(),
            bright_red: self.colors.palette.bright_red.clone(),
            bright_green: self.colors.palette.bright_green.clone(),
            bright_yellow: self.colors.palette.bright_yellow.clone(),
            bright_blue: self.colors.palette.bright_blue.clone(),
            bright_magenta: self.colors.palette.bright_magenta.clone(),
            bright_cyan: self.colors.palette.bright_cyan.clone(),
            bright_white: self.colors.palette.bright_white.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_theme() -> Theme {
        Theme {
            metadata: ThemeMetadata {
                id: "test-theme".to_string(),
                name: "Test Theme".to_string(),
                author: "Test Author".to_string(),
                description: "A test theme".to_string(),
                variant: ThemeVariant::Dark,
                tags: vec!["test".to_string()],
                url: None,
            },
            colors: ThemeColors {
                foreground: "#ffffff".to_string(),
                background: "#000000".to_string(),
                cursor: "#ffffff".to_string(),
                cursor_text: None,
                selection_background: "#444444".to_string(),
                selection_foreground: None,
                palette: ThemePalette {
                    black: "#000000".to_string(),
                    red: "#ff0000".to_string(),
                    green: "#00ff00".to_string(),
                    yellow: "#ffff00".to_string(),
                    blue: "#0000ff".to_string(),
                    magenta: "#ff00ff".to_string(),
                    cyan: "#00ffff".to_string(),
                    white: "#ffffff".to_string(),
                    bright_black: "#888888".to_string(),
                    bright_red: "#ff8888".to_string(),
                    bright_green: "#88ff88".to_string(),
                    bright_yellow: "#ffff88".to_string(),
                    bright_blue: "#8888ff".to_string(),
                    bright_magenta: "#ff88ff".to_string(),
                    bright_cyan: "#88ffff".to_string(),
                    bright_white: "#ffffff".to_string(),
                },
                ui: None,
            },
        }
    }

    #[test]
    fn test_theme_variant() {
        let theme = create_test_theme();
        assert!(theme.is_dark());
        assert!(!theme.is_light());
    }

    #[test]
    fn test_theme_accessors() {
        let theme = create_test_theme();
        assert_eq!(theme.id(), "test-theme");
        assert_eq!(theme.name(), "Test Theme");
    }

    #[test]
    fn test_serialize_deserialize() {
        let theme = create_test_theme();
        let json = serde_json::to_string(&theme).unwrap();
        let parsed: Theme = serde_json::from_str(&json).unwrap();
        assert_eq!(theme, parsed);
    }
}
