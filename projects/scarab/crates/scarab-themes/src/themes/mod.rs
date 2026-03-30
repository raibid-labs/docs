//! Built-in theme collection
//!
//! This module contains 10+ professionally designed terminal themes,
//! including popular community favorites and custom designs.

mod dracula;
mod solarized;
mod nord;
mod monokai;
mod one_dark;
mod one_light;
mod gruvbox;
mod tokyo_night;
mod catppuccin;
mod synthwave;

use crate::theme::Theme;

/// Get all built-in themes
pub fn all_themes() -> Vec<Theme> {
    vec![
        dracula::theme(),
        solarized::light(),
        solarized::dark(),
        nord::theme(),
        monokai::theme(),
        one_dark::theme(),
        one_light::theme(),
        gruvbox::light(),
        gruvbox::dark(),
        tokyo_night::theme(),
        catppuccin::mocha(),
        catppuccin::latte(),
        synthwave::theme(),
    ]
}

/// Get built-in theme by ID
pub fn get_theme(id: &str) -> Option<Theme> {
    all_themes().into_iter().find(|t| t.id() == id)
}

/// Get all theme IDs
pub fn theme_ids() -> Vec<String> {
    all_themes().iter().map(|t| t.id().to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_themes_unique_ids() {
        let themes = all_themes();
        let mut ids: Vec<_> = themes.iter().map(|t| t.id()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), themes.len(), "Duplicate theme IDs found");
    }

    #[test]
    fn test_get_theme() {
        assert!(get_theme("dracula").is_some());
        assert!(get_theme("nord").is_some());
        assert!(get_theme("nonexistent").is_none());
    }

    #[test]
    fn test_theme_ids() {
        let ids = theme_ids();
        assert!(ids.contains(&"dracula".to_string()));
        assert!(ids.contains(&"nord".to_string()));
    }
}
