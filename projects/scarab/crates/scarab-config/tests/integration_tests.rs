//! Integration tests for scarab-config

use scarab_config::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_default_config_is_valid() {
    let config = ScarabConfig::default();
    assert!(ConfigValidator::validate(&config).is_ok());
}

#[test]
fn test_load_nonexistent_config_returns_default() {
    let loader = ConfigLoader::new();
    let config = loader.load().unwrap();

    // Should not fail, should return defaults
    assert_eq!(config.font.size, 14.0);
    assert_eq!(config.terminal.scrollback_lines, 10_000);
}

#[test]
fn test_save_and_load_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let loader = ConfigLoader::with_path(config_path.clone());

    let mut config = ScarabConfig::default();
    config.font.size = 16.0;
    config.terminal.scrollback_lines = 20_000;
    config.colors.opacity = 0.9;

    // Save
    loader.save_global(&config).unwrap();
    assert!(config_path.exists());

    // Load
    let loaded = ConfigLoader::from_file(&config_path).unwrap();
    assert_eq!(loaded.font.size, 16.0);
    assert_eq!(loaded.terminal.scrollback_lines, 20_000);
    assert_eq!(loaded.colors.opacity, 0.9);
}

#[test]
fn test_config_merge() {
    let mut base = ScarabConfig::default();
    base.font.size = 14.0;
    base.terminal.scrollback_lines = 10_000;

    let mut override_config = ScarabConfig::default();
    override_config.font.size = 18.0;
    // Don't change scrollback

    base.merge(override_config);

    assert_eq!(base.font.size, 18.0);
    assert_eq!(base.terminal.scrollback_lines, 10_000);
}

#[test]
fn test_validate_font_size() {
    let mut config = ScarabConfig::default();

    // Valid
    config.font.size = 12.0;
    assert!(ConfigValidator::validate(&config).is_ok());

    // Too small
    config.font.size = 4.0;
    assert!(ConfigValidator::validate(&config).is_err());

    // Too large
    config.font.size = 100.0;
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_scrollback() {
    let mut config = ScarabConfig::default();

    // Valid
    config.terminal.scrollback_lines = 10_000;
    assert!(ConfigValidator::validate(&config).is_ok());

    // Too small
    config.terminal.scrollback_lines = 50;
    assert!(ConfigValidator::validate(&config).is_err());

    // Too large
    config.terminal.scrollback_lines = 200_000;
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_colors() {
    let mut config = ScarabConfig::default();

    // Valid color
    config.colors.foreground = Some("#ff5555".to_string());
    assert!(ConfigValidator::validate(&config).is_ok());

    // Invalid color (no #)
    config.colors.foreground = Some("ff5555".to_string());
    assert!(ConfigValidator::validate(&config).is_err());

    // Invalid color (wrong length)
    config.colors.foreground = Some("#ff55".to_string());
    assert!(ConfigValidator::validate(&config).is_err());

    // Invalid color (non-hex)
    config.colors.foreground = Some("#gggggg".to_string());
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_auto_fix() {
    let mut config = ScarabConfig::default();
    config.font.size = 100.0;
    config.terminal.scrollback_lines = 200_000;
    config.colors.opacity = 1.5;

    let fixed = ConfigValidator::auto_fix(config);

    assert_eq!(fixed.font.size, 72.0);
    assert_eq!(fixed.terminal.scrollback_lines, 100_000);
    assert_eq!(fixed.colors.opacity, 1.0);
}

#[test]
fn test_toml_serialization() {
    let config = ScarabConfig::default();

    // Serialize to TOML
    let toml = toml::to_string_pretty(&config).unwrap();
    assert!(toml.contains("[terminal]"));
    assert!(toml.contains("[font]"));
    assert!(toml.contains("[colors]"));

    // Deserialize back
    let parsed: ScarabConfig = toml::from_str(&toml).unwrap();
    assert_eq!(parsed.font.size, config.font.size);
    assert_eq!(
        parsed.terminal.scrollback_lines,
        config.terminal.scrollback_lines
    );
}

#[test]
fn test_local_config_override() {
    let temp_dir = TempDir::new().unwrap();

    // Create global config
    let global_path = temp_dir.path().join("global.toml");
    let mut global_config = ScarabConfig::default();
    global_config.font.size = 14.0;

    let toml = toml::to_string_pretty(&global_config).unwrap();
    fs::write(&global_path, toml).unwrap();

    // Create local config with override
    let local_path = temp_dir.path().join("local.toml");
    let mut local_config = ScarabConfig::default();
    local_config.font.size = 18.0;

    let toml = toml::to_string_pretty(&local_config).unwrap();
    fs::write(&local_path, toml).unwrap();

    // Load and merge
    let global = ConfigLoader::from_file(&global_path).unwrap();
    let local = ConfigLoader::from_file(&local_path).unwrap();

    let mut merged = global;
    merged.merge(local);

    assert_eq!(merged.font.size, 18.0);
}

#[test]
fn test_color_palette_validation() {
    let config = ScarabConfig::default();
    assert!(ConfigValidator::validate(&config).is_ok());

    // All default palette colors should be valid
    let palette = &config.colors.palette;
    assert!(palette.black.starts_with('#'));
    assert!(palette.red.starts_with('#'));
    assert!(palette.green.starts_with('#'));
}

#[test]
fn test_keybindings_custom() {
    let mut config = ScarabConfig::default();
    config
        .keybindings
        .custom
        .insert("my_action".to_string(), "Ctrl+Alt+X".to_string());

    let toml = toml::to_string_pretty(&config).unwrap();
    let parsed: ScarabConfig = toml::from_str(&toml).unwrap();

    assert_eq!(
        parsed.keybindings.custom.get("my_action").unwrap(),
        "Ctrl+Alt+X"
    );
}

#[test]
fn test_plugin_config() {
    let mut config = ScarabConfig::default();
    config.plugins.enabled.push("auto-notify".to_string());

    let plugin_cfg = serde_json::json!({
        "keywords": ["ERROR", "FAIL"],
        "min_runtime": 30
    });

    config
        .plugins
        .config
        .insert("auto-notify".to_string(), plugin_cfg);

    let toml = toml::to_string_pretty(&config).unwrap();
    let parsed: ScarabConfig = toml::from_str(&toml).unwrap();

    assert!(parsed.plugins.enabled.contains(&"auto-notify".to_string()));
    assert!(parsed.plugins.config.contains_key("auto-notify"));
}

#[test]
fn test_config_watcher_creation() {
    let config = ScarabConfig::default();
    let watcher = ConfigWatcher::new(config);
    assert!(watcher.is_ok());
}

#[test]
fn test_config_watcher_get_config() {
    let mut config = ScarabConfig::default();
    config.font.size = 18.0;

    let watcher = ConfigWatcher::new(config).unwrap();
    let retrieved = watcher.get_config();

    assert_eq!(retrieved.font.size, 18.0);
}

#[test]
fn test_ensure_default_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("scarab/config.toml");

    let loader = ConfigLoader::with_path(config_path.clone());

    // Should create the file
    let path = loader.ensure_default_config().unwrap();
    assert!(path.exists());

    // Should be valid config
    let loaded = ConfigLoader::from_file(&path).unwrap();
    assert!(ConfigValidator::validate(&loaded).is_ok());
}

#[test]
fn test_error_messages() {
    let err = ConfigError::InvalidFontSize("100.0".to_string());
    let error_msg = format!("{}", err);
    assert!(error_msg.contains("Invalid font size"));

    let err = ConfigError::InvalidColor("GGGGGG".to_string());
    let error_msg = format!("{}", err);
    assert!(error_msg.contains("Invalid color"));
    assert!(error_msg.contains("GGGGGG"));
}

#[test]
fn test_opacity_validation() {
    let mut config = ScarabConfig::default();

    config.colors.opacity = 0.5;
    assert!(ConfigValidator::validate(&config).is_ok());

    config.colors.opacity = -0.1;
    assert!(ConfigValidator::validate(&config).is_err());

    config.colors.opacity = 1.5;
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_line_height_validation() {
    let mut config = ScarabConfig::default();

    config.font.line_height = 1.2;
    assert!(ConfigValidator::validate(&config).is_ok());

    config.font.line_height = 0.3;
    assert!(ConfigValidator::validate(&config).is_err());

    config.font.line_height = 5.0;
    assert!(ConfigValidator::validate(&config).is_err());
}
