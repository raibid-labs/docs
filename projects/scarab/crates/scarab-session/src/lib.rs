use async_trait::async_trait;
use scarab_plugin_api::{context::NotifyLevel, Plugin, PluginContext, PluginMetadata, Result};
use scarab_protocol::ModalItem;

pub struct SessionPlugin {
    metadata: PluginMetadata,
}

impl SessionPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "scarab-session",
                "0.1.0",
                "Session management commands",
                "Scarab Team",
            ),
        }
    }
}

#[async_trait]
impl Plugin for SessionPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn get_commands(&self) -> Vec<ModalItem> {
        vec![
            ModalItem {
                id: "session.new_tab".to_string(),
                label: "New Tab".to_string(),
                description: Some("Open a new tab in current window".to_string()),
            },
            ModalItem {
                id: "session.close_tab".to_string(),
                label: "Close Tab".to_string(),
                description: Some("Close current tab".to_string()),
            },
            ModalItem {
                id: "session.detach".to_string(),
                label: "Detach Session".to_string(),
                description: Some("Detach client from session".to_string()),
            },
        ]
    }

    async fn on_remote_command(&mut self, id: &str, ctx: &PluginContext) -> Result<()> {
        match id {
            "session.new_tab" => {
                log::info!("Creating new tab");
                ctx.notify_info("New Tab", "Tab created (mock implementation)");
                // TODO: Actual implementation
            }
            "session.close_tab" => {
                log::info!("Closing tab");
                ctx.notify_info("Close Tab", "Tab closed (mock implementation)");
            }
            "session.detach" => {
                log::info!("Detaching session");
                ctx.notify_info("Detach", "Detaching from session...");
            }
            _ => {}
        }
        Ok(())
    }
}
