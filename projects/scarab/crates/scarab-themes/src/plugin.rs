//! Theme plugin implementation

use async_trait::async_trait;
use scarab_plugin_api::{
    types::{Action, ModalItem, RemoteCommand},
    Plugin, PluginContext, PluginMetadata, Result,
};
use std::sync::Mutex;

use crate::{manager::ThemeManager, format::ThemeFormat};

/// Theme plugin state
struct PluginState {
    manager: ThemeManager,
}

/// Theme system plugin
pub struct ThemePlugin {
    metadata: PluginMetadata,
    state: Mutex<PluginState>,
}

impl ThemePlugin {
    /// Create a new theme plugin
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "scarab-themes",
                env!("CARGO_PKG_VERSION"),
                "Theme system with 10+ built-in themes and import/export",
                "Scarab Team",
            )
            .with_homepage("https://github.com/raibid-labs/scarab")
            .with_catchphrase("Paint your terminal in style")
            .with_color("#bd93f9"), // Dracula purple
            state: Mutex::new(PluginState {
                manager: ThemeManager::new(),
            }),
        }
    }

    /// Get command palette items for theme operations
    fn get_theme_commands(&self) -> Vec<ModalItem> {
        let state = self.state.lock().unwrap();

        let mut commands = vec![
            ModalItem {
                id: "theme:select".to_string(),
                label: "Theme: Select Theme".to_string(),
                description: Some("Choose from available themes".to_string()),
            },
            ModalItem {
                id: "theme:preview".to_string(),
                label: "Theme: Preview Theme".to_string(),
                description: Some("Live preview without applying".to_string()),
            },
            ModalItem {
                id: "theme:clear-preview".to_string(),
                label: "Theme: Clear Preview".to_string(),
                description: Some("Return to active theme".to_string()),
            },
            ModalItem {
                id: "theme:import".to_string(),
                label: "Theme: Import from File".to_string(),
                description: Some("Import TOML, JSON, or Base16 theme".to_string()),
            },
            ModalItem {
                id: "theme:export".to_string(),
                label: "Theme: Export Current Theme".to_string(),
                description: Some("Export theme to file".to_string()),
            },
            ModalItem {
                id: "theme:create-custom".to_string(),
                label: "Theme: Create Custom".to_string(),
                description: Some("Create theme from current colors".to_string()),
            },
            ModalItem {
                id: "theme:list-dark".to_string(),
                label: "Theme: Show Dark Themes".to_string(),
                description: Some("List all dark themes".to_string()),
            },
            ModalItem {
                id: "theme:list-light".to_string(),
                label: "Theme: Show Light Themes".to_string(),
                description: Some("List all light themes".to_string()),
            },
        ];

        // Add quick-select commands for each theme
        for theme in state.manager.all_themes() {
            commands.push(ModalItem {
                id: format!("theme:apply:{}", theme.id()),
                label: format!("Theme: {}", theme.name()),
                description: Some(theme.metadata.description.clone()),
            });
        }

        commands
    }

    /// Handle theme selection
    fn handle_theme_command(&self, command_id: &str, ctx: &PluginContext) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        match command_id {
            "theme:select" => {
                // Show theme selection modal
                let themes: Vec<ModalItem> = state
                    .manager
                    .all_themes()
                    .iter()
                    .map(|t| {
                        let variant = if t.is_dark() { "dark" } else { "light" };
                        ModalItem {
                            id: format!("theme:apply:{}", t.id()),
                            label: t.name().to_string(),
                            description: Some(format!("{} ({})", t.metadata.description, variant)),
                        }
                    })
                    .collect();

                ctx.queue_command(RemoteCommand::ShowModal {
                    title: "Select Theme".to_string(),
                    items: themes,
                });
            }

            "theme:preview" => {
                // Show theme preview modal
                let themes: Vec<ModalItem> = state
                    .manager
                    .all_themes()
                    .iter()
                    .map(|t| ModalItem {
                        id: format!("theme:preview:{}", t.id()),
                        label: t.name().to_string(),
                        description: Some("Preview without applying".to_string()),
                    })
                    .collect();

                ctx.queue_command(RemoteCommand::ShowModal {
                    title: "Preview Theme".to_string(),
                    items: themes,
                });
            }

            "theme:clear-preview" => {
                state.manager.clear_preview();
                ctx.queue_command(RemoteCommand::PluginNotify {
                    title: "Theme Preview Cleared".to_string(),
                    body: "Returned to active theme".to_string(),
                    level: scarab_plugin_api::context::NotifyLevel::Info,
                });
            }

            "theme:list-dark" => {
                let themes: Vec<ModalItem> = state
                    .manager
                    .dark_themes()
                    .iter()
                    .map(|t| ModalItem {
                        id: format!("theme:apply:{}", t.id()),
                        label: t.name().to_string(),
                        description: Some(t.metadata.description.clone()),
                    })
                    .collect();

                ctx.queue_command(RemoteCommand::ShowModal {
                    title: "Dark Themes".to_string(),
                    items: themes,
                });
            }

            "theme:list-light" => {
                let themes: Vec<ModalItem> = state
                    .manager
                    .light_themes()
                    .iter()
                    .map(|t| ModalItem {
                        id: format!("theme:apply:{}", t.id()),
                        label: t.name().to_string(),
                        description: Some(t.metadata.description.clone()),
                    })
                    .collect();

                ctx.queue_command(RemoteCommand::ShowModal {
                    title: "Light Themes".to_string(),
                    items: themes,
                });
            }

            id if id.starts_with("theme:apply:") => {
                let theme_id = id.strip_prefix("theme:apply:").unwrap();
                if let Err(e) = state.manager.set_active_theme(theme_id) {
                    log::error!("Failed to apply theme {}: {}", theme_id, e);
                    ctx.queue_command(RemoteCommand::PluginNotify {
                        title: "Theme Error".to_string(),
                        body: format!("Failed to apply theme: {}", e),
                        level: scarab_plugin_api::context::NotifyLevel::Error,
                    });
                } else {
                    log::info!("Applied theme: {}", theme_id);
                    ctx.queue_command(RemoteCommand::PluginNotify {
                        title: "Theme Applied".to_string(),
                        body: format!("Switched to {}", theme_id),
                        level: scarab_plugin_api::context::NotifyLevel::Success,
                    });

                    // TODO: Send theme update to client via IPC
                    // This would require extending the RemoteCommand enum
                }
            }

            id if id.starts_with("theme:preview:") => {
                let theme_id = id.strip_prefix("theme:preview:").unwrap();
                if let Err(e) = state.manager.set_preview_theme(theme_id) {
                    log::error!("Failed to preview theme {}: {}", theme_id, e);
                } else {
                    log::info!("Previewing theme: {}", theme_id);
                    ctx.queue_command(RemoteCommand::PluginNotify {
                        title: "Theme Preview".to_string(),
                        body: format!("Previewing {}", theme_id),
                        level: scarab_plugin_api::context::NotifyLevel::Info,
                    });
                }
            }

            _ => {
                log::warn!("Unknown theme command: {}", command_id);
            }
        }

        Ok(())
    }
}

impl Default for ThemePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for ThemePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn get_commands(&self) -> Vec<ModalItem> {
        self.get_theme_commands()
    }

    async fn on_load(&mut self, _ctx: &mut PluginContext) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        // Initialize theme manager
        if let Err(e) = state.manager.initialize() {
            log::error!("Failed to initialize theme manager: {}", e);
        }

        log::info!(
            "Theme plugin loaded with {} themes",
            state.manager.all_themes().len()
        );

        Ok(())
    }

    async fn on_remote_command(&mut self, id: &str, ctx: &PluginContext) -> Result<()> {
        if id.starts_with("theme:") {
            self.handle_theme_command(id, ctx)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata() {
        let plugin = ThemePlugin::new();
        assert_eq!(plugin.metadata().name, "scarab-themes");
    }

    #[test]
    fn test_get_commands() {
        let plugin = ThemePlugin::new();
        let commands = plugin.get_commands();
        assert!(!commands.is_empty());
        assert!(commands.iter().any(|c| c.id == "theme:select"));
    }
}
