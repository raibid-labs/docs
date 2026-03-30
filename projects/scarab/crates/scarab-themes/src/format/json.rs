//! JSON format handler

use crate::{
    error::ThemeResult,
    format::FormatHandler,
    theme::Theme,
};

pub struct JsonFormat;

impl FormatHandler for JsonFormat {
    fn parse(content: &str) -> ThemeResult<Theme> {
        let theme: Theme = serde_json::from_str(content)?;
        Ok(theme)
    }

    fn serialize(theme: &Theme) -> ThemeResult<String> {
        let json = serde_json::to_string_pretty(theme)?;
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::themes;

    #[test]
    fn test_json_roundtrip() {
        let original = themes::get_theme("nord").unwrap();
        let serialized = JsonFormat::serialize(&original).unwrap();
        let parsed = JsonFormat::parse(&serialized).unwrap();
        assert_eq!(original.id(), parsed.id());
        assert_eq!(original.colors, parsed.colors);
    }
}
