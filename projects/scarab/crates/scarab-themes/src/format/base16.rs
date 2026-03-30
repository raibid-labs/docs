//! Base16 format handler
//!
//! Supports importing themes in Base16 YAML format.
//! See: https://github.com/chriskempson/base16

use crate::{
    error::{ThemeError, ThemeResult},
    format::FormatHandler,
    theme::{Theme, ThemeColors, ThemeMetadata, ThemePalette, ThemeVariant},
};
use serde::{Deserialize, Serialize};

pub struct Base16Format;

/// Base16 theme structure
#[derive(Debug, Deserialize, Serialize)]
struct Base16Theme {
    scheme: String,
    author: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    variant: Option<String>,

    base00: String, // background
    base01: String, // lighter background
    base02: String, // selection background
    base03: String, // comments, invisibles
    base04: String, // dark foreground
    base05: String, // default foreground
    base06: String, // light foreground
    base07: String, // light background

    base08: String, // red
    base09: String, // orange
    base0A: String, // yellow
    base0B: String, // green
    base0C: String, // cyan
    base0D: String, // blue
    base0E: String, // magenta
    base0F: String, // brown
}

impl FormatHandler for Base16Format {
    fn parse(content: &str) -> ThemeResult<Theme> {
        let base16: Base16Theme = serde_yaml::from_str(content)
            .map_err(|e| ThemeError::InvalidFormat(format!("YAML parse error: {}", e)))?;

        // Convert Base16 to Theme
        let id = base16
            .scheme
            .to_lowercase()
            .replace(' ', "-")
            .replace('_', "-");

        let variant = match base16.variant.as_deref() {
            Some("light") => ThemeVariant::Light,
            _ => ThemeVariant::Dark,
        };

        let theme = Theme {
            metadata: ThemeMetadata {
                id,
                name: base16.scheme,
                author: base16.author,
                description: base16
                    .description
                    .unwrap_or_else(|| "Base16 theme".to_string()),
                variant,
                tags: vec!["base16".to_string()],
                url: None,
            },
            colors: ThemeColors {
                foreground: format!("#{}", base16.base05),
                background: format!("#{}", base16.base00),
                cursor: format!("#{}", base16.base05),
                cursor_text: Some(format!("#{}", base16.base00)),
                selection_background: format!("#{}", base16.base02),
                selection_foreground: None,
                palette: ThemePalette {
                    black: format!("#{}", base16.base00),
                    red: format!("#{}", base16.base08),
                    green: format!("#{}", base16.base0B),
                    yellow: format!("#{}", base16.base0A),
                    blue: format!("#{}", base16.base0D),
                    magenta: format!("#{}", base16.base0E),
                    cyan: format!("#{}", base16.base0C),
                    white: format!("#{}", base16.base05),
                    bright_black: format!("#{}", base16.base03),
                    bright_red: format!("#{}", base16.base08),
                    bright_green: format!("#{}", base16.base0B),
                    bright_yellow: format!("#{}", base16.base0A),
                    bright_blue: format!("#{}", base16.base0D),
                    bright_magenta: format!("#{}", base16.base0E),
                    bright_cyan: format!("#{}", base16.base0C),
                    bright_white: format!("#{}", base16.base07),
                },
                ui: None,
            },
        };

        Ok(theme)
    }

    fn serialize(theme: &Theme) -> ThemeResult<String> {
        // Convert Theme to Base16 (lossy conversion)
        let base16 = Base16Theme {
            scheme: theme.name().to_string(),
            author: theme.metadata.author.clone(),
            description: Some(theme.metadata.description.clone()),
            variant: Some(match theme.metadata.variant {
                ThemeVariant::Light => "light".to_string(),
                ThemeVariant::Dark => "dark".to_string(),
            }),
            base00: strip_hash(&theme.colors.background),
            base01: strip_hash(&theme.colors.palette.bright_black), // approximate
            base02: strip_hash(&theme.colors.selection_background),
            base03: strip_hash(&theme.colors.palette.bright_black),
            base04: strip_hash(&theme.colors.palette.white),
            base05: strip_hash(&theme.colors.foreground),
            base06: strip_hash(&theme.colors.palette.bright_white),
            base07: strip_hash(&theme.colors.palette.bright_white),
            base08: strip_hash(&theme.colors.palette.red),
            base09: strip_hash(&theme.colors.palette.yellow), // orange approximation
            base0A: strip_hash(&theme.colors.palette.yellow),
            base0B: strip_hash(&theme.colors.palette.green),
            base0C: strip_hash(&theme.colors.palette.cyan),
            base0D: strip_hash(&theme.colors.palette.blue),
            base0E: strip_hash(&theme.colors.palette.magenta),
            base0F: strip_hash(&theme.colors.palette.yellow), // brown approximation
        };

        let yaml = serde_yaml::to_string(&base16)
            .map_err(|e| ThemeError::InvalidFormat(format!("YAML serialize error: {}", e)))?;

        Ok(yaml)
    }
}

/// Strip leading '#' from hex color
fn strip_hash(color: &str) -> String {
    color.trim_start_matches('#').to_string()
}

// Add serde_yaml dependency to Cargo.toml
