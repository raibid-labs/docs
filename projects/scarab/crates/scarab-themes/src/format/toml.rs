//! TOML format handler

use crate::{
    error::ThemeResult,
    format::FormatHandler,
    theme::Theme,
};

pub struct TomlFormat;

impl FormatHandler for TomlFormat {
    fn parse(content: &str) -> ThemeResult<Theme> {
        let theme: Theme = toml::from_str(content)?;
        Ok(theme)
    }

    fn serialize(theme: &Theme) -> ThemeResult<String> {
        let toml = toml::to_string_pretty(theme)?;
        Ok(toml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::themes;

    #[test]
    fn test_toml_roundtrip() {
        let original = themes::get_theme("dracula").unwrap();
        let serialized = TomlFormat::serialize(&original).unwrap();
        let parsed = TomlFormat::parse(&serialized).unwrap();
        assert_eq!(original.id(), parsed.id());
        assert_eq!(original.colors, parsed.colors);
    }
}
