use async_trait::async_trait;
use scarab_plugin_api::{
    types::RemoteCommand, Action, Plugin, PluginContext, PluginMetadata, Result,
};
use std::sync::Mutex;

pub struct PalettePlugin {
    metadata: PluginMetadata,
    state: Mutex<PluginState>,
}

#[derive(Default)]
struct PluginState {
    active: bool,
}

impl PalettePlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "scarab-palette",
                "0.1.0",
                "Command Palette integration",
                "Scarab Team",
            ),
            state: Mutex::new(PluginState::default()),
        }
    }
}

#[async_trait]
impl Plugin for PalettePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_input(&mut self, input: &[u8], ctx: &PluginContext) -> Result<Action> {
        // Trigger: Ctrl+P (0x10)
        if input == [0x10] {
            // Ctrl+P
            log::info!("Opening Command Palette");

            // Get aggregated commands from shared state
            let items = ctx.state.lock().commands.clone();

            // Send ShowModal
            ctx.queue_command(RemoteCommand::ShowModal {
                title: "Command Palette".to_string(),
                items,
            });

            return Ok(Action::Modify(Vec::new())); // Consume key
        }

        Ok(Action::Continue)
    }
}
