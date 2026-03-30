//! Integration tests for theme system

use scarab_themes::{
    format::{ThemeFormat, FormatHandler, TomlFormat, JsonFormat},
    manager::ThemeManager,
    themes,
};

#[test]
fn test_all_builtin_themes_valid() {
    let themes = themes::all_themes();
    assert!(themes.len() >= 13, "Expected at least 13 built-in themes");

    for theme in themes {
        // Validate theme has required fields
        assert!(!theme.id().is_empty());
        assert!(!theme.name().is_empty());
        assert!(!theme.metadata.author.is_empty());
        assert!(!theme.colors.foreground.is_empty());
        assert!(!theme.colors.background.is_empty());

        // Validate colors start with #
        assert!(theme.colors.foreground.starts_with('#'));
        assert!(theme.colors.background.starts_with('#'));
        assert!(theme.colors.cursor.starts_with('#'));
    }
}

#[test]
fn test_theme_manager_initialization() {
    let mut manager = ThemeManager::new();
    assert!(!manager.all_themes().is_empty());

    // Test theme lookup
    assert!(manager.get_theme("dracula").is_some());
    assert!(manager.get_theme("nord").is_some());
    assert!(manager.get_theme("tokyo-night").is_some());
    assert!(manager.get_theme("nonexistent").is_none());
}

#[test]
fn test_theme_application() {
    let mut manager = ThemeManager::new();

    // Apply theme
    manager.set_active_theme("dracula").unwrap();
    assert_eq!(manager.active_theme().unwrap().id(), "dracula");

    // Change theme
    manager.set_active_theme("nord").unwrap();
    assert_eq!(manager.active_theme().unwrap().id(), "nord");

    // Invalid theme
    assert!(manager.set_active_theme("invalid").is_err());
}

#[test]
fn test_theme_preview() {
    let mut manager = ThemeManager::new();

    // Set active theme
    manager.set_active_theme("dracula").unwrap();

    // Preview different theme
    manager.set_preview_theme("nord").unwrap();

    // Active theme unchanged
    assert_eq!(manager.active_theme().unwrap().id(), "dracula");
    // Preview theme set
    assert_eq!(manager.preview_theme().unwrap().id(), "nord");
    // Current returns preview
    assert_eq!(manager.current_theme().unwrap().id(), "nord");

    // Clear preview
    manager.clear_preview();
    assert!(manager.preview_theme().is_none());
    assert_eq!(manager.current_theme().unwrap().id(), "dracula");
}

#[test]
fn test_dark_light_filtering() {
    let manager = ThemeManager::new();

    let dark_themes = manager.dark_themes();
    let light_themes = manager.light_themes();

    assert!(!dark_themes.is_empty());
    assert!(!light_themes.is_empty());

    // Verify all dark themes are actually dark
    for theme in dark_themes {
        assert!(theme.is_dark());
        assert!(!theme.is_light());
    }

    // Verify all light themes are actually light
    for theme in light_themes {
        assert!(theme.is_light());
        assert!(!theme.is_dark());
    }
}

#[test]
fn test_toml_serialization() {
    let theme = themes::get_theme("dracula").unwrap();

    // Serialize to TOML
    let toml = TomlFormat::serialize(&theme).unwrap();
    assert!(!toml.is_empty());

    // Deserialize back
    let parsed = TomlFormat::parse(&toml).unwrap();

    // Verify round-trip
    assert_eq!(theme.id(), parsed.id());
    assert_eq!(theme.name(), parsed.name());
    assert_eq!(theme.colors, parsed.colors);
}

#[test]
fn test_json_serialization() {
    let theme = themes::get_theme("nord").unwrap();

    // Serialize to JSON
    let json = JsonFormat::serialize(&theme).unwrap();
    assert!(!json.is_empty());

    // Deserialize back
    let parsed = JsonFormat::parse(&json).unwrap();

    // Verify round-trip
    assert_eq!(theme.id(), parsed.id());
    assert_eq!(theme.name(), parsed.name());
    assert_eq!(theme.colors, parsed.colors);
}

#[test]
fn test_theme_to_color_config() {
    let theme = themes::get_theme("dracula").unwrap();
    let config = theme.to_color_config();

    assert_eq!(config.theme.as_deref(), Some("dracula"));
    assert_eq!(config.foreground.as_deref(), Some("#f8f8f2"));
    assert_eq!(config.background.as_deref(), Some("#282a36"));
    assert_eq!(config.palette.red, "#ff5555");
}

#[test]
fn test_theme_ids_unique() {
    let manager = ThemeManager::new();
    let ids = manager.theme_ids();

    let mut sorted_ids = ids.clone();
    sorted_ids.sort();
    sorted_ids.dedup();

    assert_eq!(ids.len(), sorted_ids.len(), "Duplicate theme IDs detected");
}

#[test]
fn test_popular_themes_exist() {
    let required_themes = vec![
        "dracula",
        "nord",
        "monokai",
        "solarized-dark",
        "solarized-light",
        "one-dark",
        "one-light",
        "gruvbox-dark",
        "gruvbox-light",
        "tokyo-night",
        "catppuccin-mocha",
        "catppuccin-latte",
        "synthwave",
    ];

    let manager = ThemeManager::new();
    for theme_id in required_themes {
        assert!(
            manager.get_theme(theme_id).is_some(),
            "Required theme '{}' not found",
            theme_id
        );
    }
}

#[test]
fn test_theme_format_detection() {
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
