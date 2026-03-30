//! Theme import/export format handlers
//!
//! Supports multiple theme formats:
//! - TOML: Simple, human-readable format
//! - JSON: Standard interchange format
//! - Base16: Compatible with Base16 theme system

mod base16;
mod json;
mod toml;

pub use self::json::JsonFormat;
pub use self::toml::TomlFormat;
pub use base16::Base16Format;

use crate::{
    error::{ThemeError, ThemeResult},
    theme::Theme,
};

/// Theme file format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeFormat {
    /// TOML format
    Toml,
    /// JSON format
    Json,
    /// Base16 YAML format
    Base16,
}

impl ThemeFormat {
    /// Get file extension for format
    pub fn extension(&self) -> &'static str {
        match self {
            ThemeFormat::Toml => "toml",
            ThemeFormat::Json => "json",
            ThemeFormat::Base16 => "yaml",
        }
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "toml" => Some(ThemeFormat::Toml),
            "json" => Some(ThemeFormat::Json),
            "yaml" | "yml" => Some(ThemeFormat::Base16),
            _ => None,
        }
    }
}

/// Parse theme from string with given format
pub fn parse_theme(content: &str, format: ThemeFormat) -> ThemeResult<Theme> {
    match format {
        ThemeFormat::Toml => TomlFormat::parse(content),
        ThemeFormat::Json => JsonFormat::parse(content),
        ThemeFormat::Base16 => Base16Format::parse(content),
    }
}

/// Serialize theme to string with given format
pub fn serialize_theme(theme: &Theme, format: ThemeFormat) -> ThemeResult<String> {
    match format {
        ThemeFormat::Toml => TomlFormat::serialize(theme),
        ThemeFormat::Json => JsonFormat::serialize(theme),
        ThemeFormat::Base16 => Base16Format::serialize(theme),
    }
}

/// Trait for theme format parsers/serializers
pub trait FormatHandler {
    /// Parse theme from string
    fn parse(content: &str) -> ThemeResult<Theme>;

    /// Serialize theme to string
    fn serialize(theme: &Theme) -> ThemeResult<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_extension() {
        assert_eq!(ThemeFormat::Toml.extension(), "toml");
        assert_eq!(ThemeFormat::Json.extension(), "json");
        assert_eq!(ThemeFormat::Base16.extension(), "yaml");
    }

    #[test]
    fn test_detect_format() {
        assert_eq!(
            ThemeFormat::from_extension("toml"),
            Some(ThemeFormat::Toml)
        );
        assert_eq!(
            ThemeFormat::from_extension("json"),
            Some(ThemeFormat::Json)
        );
        assert_eq!(
            ThemeFormat::from_extension("yaml"),
            Some(ThemeFormat::Base16)
        );
        assert_eq!(ThemeFormat::from_extension("txt"), None);
    }
}
