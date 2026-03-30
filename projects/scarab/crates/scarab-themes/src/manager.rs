//! Theme manager for loading, applying, and managing themes

use crate::{
    error::{ThemeError, ThemeResult},
    format::{self, ThemeFormat},
    theme::Theme,
    themes,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Theme manager handles theme operations
pub struct ThemeManager {
    /// Built-in themes
    builtin_themes: HashMap<String, Theme>,

    /// User-installed themes
    user_themes: HashMap<String, Theme>,

    /// Currently active theme ID
    active_theme_id: Option<String>,

    /// Preview theme (temporary, not applied)
    preview_theme_id: Option<String>,

    /// User themes directory
    themes_dir: PathBuf,
}

impl ThemeManager {
    /// Create a new theme manager
    pub fn new() -> Self {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let themes_dir = PathBuf::from(home_dir).join(".config/scarab/themes");

        let mut builtin_themes = HashMap::new();
        for theme in themes::all_themes() {
            builtin_themes.insert(theme.id().to_string(), theme);
        }

        Self {
            builtin_themes,
            user_themes: HashMap::new(),
            active_theme_id: None,
            preview_theme_id: None,
            themes_dir,
        }
    }

    /// Initialize theme manager (create directories, load user themes)
    pub fn initialize(&mut self) -> ThemeResult<()> {
        // Create themes directory if it doesn't exist
        if !self.themes_dir.exists() {
            std::fs::create_dir_all(&self.themes_dir)?;
            log::info!("Created themes directory: {}", self.themes_dir.display());
        }

        // Load user themes
        self.load_user_themes()?;

        Ok(())
    }

    /// Load all user themes from the themes directory
    fn load_user_themes(&mut self) -> ThemeResult<()> {
        if !self.themes_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.themes_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    if ext == "toml" || ext == "json" {
                        match self.load_theme_from_file(&path) {
                            Ok(theme) => {
                                log::info!("Loaded user theme: {}", theme.id());
                                self.user_themes.insert(theme.id().to_string(), theme);
                            }
                            Err(e) => {
                                log::warn!("Failed to load theme from {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get all available themes (built-in + user)
    pub fn all_themes(&self) -> Vec<&Theme> {
        self.builtin_themes
            .values()
            .chain(self.user_themes.values())
            .collect()
    }

    /// Get theme by ID
    pub fn get_theme(&self, id: &str) -> Option<&Theme> {
        self.builtin_themes
            .get(id)
            .or_else(|| self.user_themes.get(id))
    }

    /// Get all theme IDs
    pub fn theme_ids(&self) -> Vec<String> {
        let mut ids: Vec<_> = self
            .builtin_themes
            .keys()
            .chain(self.user_themes.keys())
            .cloned()
            .collect();
        ids.sort();
        ids
    }

    /// Get active theme
    pub fn active_theme(&self) -> Option<&Theme> {
        self.active_theme_id
            .as_ref()
            .and_then(|id| self.get_theme(id))
    }

    /// Set active theme by ID
    pub fn set_active_theme(&mut self, id: &str) -> ThemeResult<()> {
        if self.get_theme(id).is_none() {
            return Err(ThemeError::NotFound(id.to_string()));
        }

        self.active_theme_id = Some(id.to_string());
        self.preview_theme_id = None; // Clear preview when applying
        log::info!("Applied theme: {}", id);
        Ok(())
    }

    /// Get preview theme (if any)
    pub fn preview_theme(&self) -> Option<&Theme> {
        self.preview_theme_id
            .as_ref()
            .and_then(|id| self.get_theme(id))
    }

    /// Set preview theme (doesn't change active theme)
    pub fn set_preview_theme(&mut self, id: &str) -> ThemeResult<()> {
        if self.get_theme(id).is_none() {
            return Err(ThemeError::NotFound(id.to_string()));
        }

        self.preview_theme_id = Some(id.to_string());
        log::info!("Previewing theme: {}", id);
        Ok(())
    }

    /// Clear preview (return to active theme)
    pub fn clear_preview(&mut self) {
        self.preview_theme_id = None;
        log::info!("Cleared theme preview");
    }

    /// Get currently displayed theme (preview or active)
    pub fn current_theme(&self) -> Option<&Theme> {
        self.preview_theme().or_else(|| self.active_theme())
    }

    /// Import theme from file
    pub fn import_theme<P: AsRef<Path>>(&mut self, path: P) -> ThemeResult<Theme> {
        let theme = self.load_theme_from_file(path.as_ref())?;

        // Add to user themes
        self.user_themes.insert(theme.id().to_string(), theme.clone());

        log::info!("Imported theme: {}", theme.id());
        Ok(theme)
    }

    /// Load theme from file (auto-detect format)
    fn load_theme_from_file(&self, path: &Path) -> ThemeResult<Theme> {
        let contents = std::fs::read_to_string(path)?;

        let format = match path.extension().and_then(|s| s.to_str()) {
            Some("toml") => ThemeFormat::Toml,
            Some("json") => ThemeFormat::Json,
            _ => {
                return Err(ThemeError::InvalidFormat(
                    "Unknown file extension".to_string(),
                ))
            }
        };

        format::parse_theme(&contents, format)
    }

    /// Export theme to file
    pub fn export_theme<P: AsRef<Path>>(
        &self,
        theme_id: &str,
        path: P,
        format: ThemeFormat,
    ) -> ThemeResult<()> {
        let theme = self
            .get_theme(theme_id)
            .ok_or_else(|| ThemeError::NotFound(theme_id.to_string()))?;

        let contents = format::serialize_theme(theme, format)?;
        std::fs::write(path.as_ref(), contents)?;

        log::info!("Exported theme {} to {:?}", theme_id, path.as_ref());
        Ok(())
    }

    /// Create custom theme from current colors
    pub fn create_custom_theme(
        &mut self,
        id: String,
        name: String,
        colors: scarab_config::ColorConfig,
    ) -> ThemeResult<Theme> {
        use crate::theme::{ThemeColors, ThemeMetadata, ThemePalette, ThemeVariant};

        let theme = Theme {
            metadata: ThemeMetadata {
                id: id.clone(),
                name,
                author: "User".to_string(),
                description: "Custom user theme".to_string(),
                variant: ThemeVariant::Dark, // Default to dark
                tags: vec!["custom".to_string()],
                url: None,
            },
            colors: ThemeColors {
                foreground: colors.foreground.unwrap_or_else(|| "#ffffff".to_string()),
                background: colors.background.unwrap_or_else(|| "#000000".to_string()),
                cursor: colors.cursor.unwrap_or_else(|| "#ffffff".to_string()),
                cursor_text: None,
                selection_background: colors
                    .selection_background
                    .unwrap_or_else(|| "#444444".to_string()),
                selection_foreground: colors.selection_foreground,
                palette: ThemePalette {
                    black: colors.palette.black,
                    red: colors.palette.red,
                    green: colors.palette.green,
                    yellow: colors.palette.yellow,
                    blue: colors.palette.blue,
                    magenta: colors.palette.magenta,
                    cyan: colors.palette.cyan,
                    white: colors.palette.white,
                    bright_black: colors.palette.bright_black,
                    bright_red: colors.palette.bright_red,
                    bright_green: colors.palette.bright_green,
                    bright_yellow: colors.palette.bright_yellow,
                    bright_blue: colors.palette.bright_blue,
                    bright_magenta: colors.palette.bright_magenta,
                    bright_cyan: colors.palette.bright_cyan,
                    bright_white: colors.palette.bright_white,
                },
                ui: None,
            },
        };

        // Save to user themes
        self.user_themes.insert(id.clone(), theme.clone());

        log::info!("Created custom theme: {}", id);
        Ok(theme)
    }

    /// Search themes by tag
    pub fn search_by_tag(&self, tag: &str) -> Vec<&Theme> {
        self.all_themes()
            .into_iter()
            .filter(|t| t.metadata.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Get dark themes
    pub fn dark_themes(&self) -> Vec<&Theme> {
        self.all_themes()
            .into_iter()
            .filter(|t| t.is_dark())
            .collect()
    }

    /// Get light themes
    pub fn light_themes(&self) -> Vec<&Theme> {
        self.all_themes()
            .into_iter()
            .filter(|t| t.is_light())
            .collect()
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager_initialization() {
        let manager = ThemeManager::new();
        assert!(!manager.builtin_themes.is_empty());
        assert!(manager.get_theme("dracula").is_some());
        assert!(manager.get_theme("nord").is_some());
    }

    #[test]
    fn test_set_active_theme() {
        let mut manager = ThemeManager::new();
        manager.set_active_theme("dracula").unwrap();
        assert_eq!(manager.active_theme().unwrap().id(), "dracula");
    }

    #[test]
    fn test_preview_theme() {
        let mut manager = ThemeManager::new();
        manager.set_active_theme("dracula").unwrap();
        manager.set_preview_theme("nord").unwrap();

        assert_eq!(manager.active_theme().unwrap().id(), "dracula");
        assert_eq!(manager.preview_theme().unwrap().id(), "nord");
        assert_eq!(manager.current_theme().unwrap().id(), "nord");

        manager.clear_preview();
        assert_eq!(manager.current_theme().unwrap().id(), "dracula");
    }

    #[test]
    fn test_search_themes() {
        let manager = ThemeManager::new();
        let dark_themes = manager.dark_themes();
        assert!(!dark_themes.is_empty());

        let light_themes = manager.light_themes();
        assert!(!light_themes.is_empty());
    }
}
