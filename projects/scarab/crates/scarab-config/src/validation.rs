//! Configuration validation

use crate::{ColorPalette, ConfigError, Result, ScarabConfig};
use tracing::warn;

/// Configuration validator
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate entire configuration
    pub fn validate(config: &ScarabConfig) -> Result<()> {
        Self::validate_font(&config.font)?;
        Self::validate_terminal(&config.terminal)?;
        Self::validate_colors(&config.colors)?;
        Self::validate_ui(&config.ui)?;
        Ok(())
    }

    /// Validate font configuration
    fn validate_font(font: &crate::FontConfig) -> Result<()> {
        if font.size < 6.0 || font.size > 72.0 {
            return Err(ConfigError::InvalidFontSize(font.size.to_string()));
        }

        if font.line_height < 0.5 || font.line_height > 3.0 {
            return Err(ConfigError::InvalidLineHeight(font.line_height.to_string()));
        }

        if font.family.is_empty() {
            return Err(ConfigError::Validation(
                "Font family cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate terminal configuration
    fn validate_terminal(terminal: &crate::TerminalConfig) -> Result<()> {
        if terminal.scrollback_lines < 100 || terminal.scrollback_lines > 100_000 {
            return Err(ConfigError::InvalidScrollback(terminal.scrollback_lines.to_string()));
        }

        if terminal.default_shell.is_empty() {
            return Err(ConfigError::InvalidShell(
                "Shell command cannot be empty".to_string(),
            ));
        }

        if terminal.scroll_multiplier < 0.1 || terminal.scroll_multiplier > 10.0 {
            warn!(
                "Scroll multiplier {} is unusual (recommended: 1.0-5.0)",
                terminal.scroll_multiplier
            );
        }

        Ok(())
    }

    /// Validate color configuration
    fn validate_colors(colors: &crate::ColorConfig) -> Result<()> {
        // Validate custom colors if set
        if let Some(ref fg) = colors.foreground {
            Self::validate_color(fg)?;
        }
        if let Some(ref bg) = colors.background {
            Self::validate_color(bg)?;
        }
        if let Some(ref cursor) = colors.cursor {
            Self::validate_color(cursor)?;
        }
        if let Some(ref sel_bg) = colors.selection_background {
            Self::validate_color(sel_bg)?;
        }
        if let Some(ref sel_fg) = colors.selection_foreground {
            Self::validate_color(sel_fg)?;
        }

        // Validate palette
        Self::validate_palette(&colors.palette)?;

        // Validate opacity
        if colors.opacity < 0.0 || colors.opacity > 1.0 {
            return Err(ConfigError::Validation(format!(
                "Opacity {} must be between 0.0 and 1.0",
                colors.opacity
            )));
        }

        if colors.dim_opacity < 0.0 || colors.dim_opacity > 1.0 {
            return Err(ConfigError::Validation(format!(
                "Dim opacity {} must be between 0.0 and 1.0",
                colors.dim_opacity
            )));
        }

        Ok(())
    }

    /// Validate UI configuration
    fn validate_ui(ui: &crate::UiConfig) -> Result<()> {
        if ui.cursor_blink_interval < 100 || ui.cursor_blink_interval > 5000 {
            warn!(
                "Cursor blink interval {}ms is unusual (recommended: 500-1000ms)",
                ui.cursor_blink_interval
            );
        }

        Ok(())
    }

    /// Validate a single color (hex format)
    fn validate_color(color: &str) -> Result<()> {
        if !color.starts_with('#') {
            return Err(ConfigError::InvalidColor(format!(
                "{} (missing # prefix)",
                color
            )));
        }

        if color.len() != 7 && color.len() != 9 {
            return Err(ConfigError::InvalidColor(format!(
                "{} (must be #RRGGBB or #RRGGBBAA)",
                color
            )));
        }

        // Validate hex digits
        for ch in color[1..].chars() {
            if !ch.is_ascii_hexdigit() {
                return Err(ConfigError::InvalidColor(format!(
                    "{} (invalid hex digit: {})",
                    color, ch
                )));
            }
        }

        Ok(())
    }

    /// Validate color palette
    fn validate_palette(palette: &ColorPalette) -> Result<()> {
        let colors = [
            ("black", &palette.black),
            ("red", &palette.red),
            ("green", &palette.green),
            ("yellow", &palette.yellow),
            ("blue", &palette.blue),
            ("magenta", &palette.magenta),
            ("cyan", &palette.cyan),
            ("white", &palette.white),
            ("bright_black", &palette.bright_black),
            ("bright_red", &palette.bright_red),
            ("bright_green", &palette.bright_green),
            ("bright_yellow", &palette.bright_yellow),
            ("bright_blue", &palette.bright_blue),
            ("bright_magenta", &palette.bright_magenta),
            ("bright_cyan", &palette.bright_cyan),
            ("bright_white", &palette.bright_white),
        ];

        for (name, color) in colors {
            Self::validate_color(color)
                .map_err(|e| ConfigError::InvalidColor(format!("palette.{}: {}", name, e)))?;
        }

        Ok(())
    }

    /// Auto-fix common issues (returns fixed config)
    pub fn auto_fix(mut config: ScarabConfig) -> ScarabConfig {
        // Clamp font size
        if config.font.size < 6.0 {
            warn!("Font size too small, setting to 6.0");
            config.font.size = 6.0;
        } else if config.font.size > 72.0 {
            warn!("Font size too large, setting to 72.0");
            config.font.size = 72.0;
        }

        // Clamp line height
        if config.font.line_height < 0.5 {
            warn!("Line height too small, setting to 0.5");
            config.font.line_height = 0.5;
        } else if config.font.line_height > 3.0 {
            warn!("Line height too large, setting to 3.0");
            config.font.line_height = 3.0;
        }

        // Clamp scrollback
        if config.terminal.scrollback_lines < 100 {
            warn!("Scrollback too small, setting to 100");
            config.terminal.scrollback_lines = 100;
        } else if config.terminal.scrollback_lines > 100_000 {
            warn!("Scrollback too large, setting to 100,000");
            config.terminal.scrollback_lines = 100_000;
        }

        // Clamp opacity
        config.colors.opacity = config.colors.opacity.clamp(0.0, 1.0);
        config.colors.dim_opacity = config.colors.dim_opacity.clamp(0.0, 1.0);

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_default_config() {
        let config = ScarabConfig::default();
        assert!(ConfigValidator::validate(&config).is_ok());
    }

    #[test]
    fn test_invalid_font_size() {
        let mut config = ScarabConfig::default();
        config.font.size = 100.0;
        assert!(ConfigValidator::validate(&config).is_err());
    }

    #[test]
    fn test_invalid_color() {
        let result = ConfigValidator::validate_color("FF5555");
        assert!(result.is_err());

        let result = ConfigValidator::validate_color("#GG5555");
        assert!(result.is_err());

        let result = ConfigValidator::validate_color("#FF5555");
        assert!(result.is_ok());
    }

    #[test]
    fn test_auto_fix() {
        let mut config = ScarabConfig::default();
        config.font.size = 100.0;
        config.terminal.scrollback_lines = 200_000;

        let fixed = ConfigValidator::auto_fix(config);
        assert_eq!(fixed.font.size, 72.0);
        assert_eq!(fixed.terminal.scrollback_lines, 100_000);
    }

    #[test]
    fn test_validate_opacity() {
        let mut config = ScarabConfig::default();
        config.colors.opacity = 1.5;
        assert!(ConfigValidator::validate(&config).is_err());

        config.colors.opacity = 0.8;
        assert!(ConfigValidator::validate(&config).is_ok());
    }
}
