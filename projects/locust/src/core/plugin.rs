use crate::core::context::LocustContext;
use crate::core::input::PluginEventResult;
use ratatui::prelude::Frame;
use ratatui::backend::Backend;
use crossterm::event::Event;

/// Trait implemented by all Locust plugins.
///
/// Plugins can:
/// - Observe and optionally consume input events.
/// - Render overlays on top of the existing ratatui UI.
pub trait LocustPlugin<B>
where
    B: Backend,
{
    /// Stable, unique identifier for the plugin.
    fn id(&self) -> &'static str;

    /// Called once when the plugin is registered with Locust.
    fn init(&mut self, _ctx: &mut LocustContext) {}

    /// Allow the plugin to react to and optionally consume an input event.
    fn on_event(
        &mut self,
        event: &Event,
        ctx: &mut LocustContext,
    ) -> PluginEventResult;

    /// Render this plugin's overlay on top of the current frame.
    fn render_overlay(&self, frame: &mut Frame<'_, B>, ctx: &LocustContext);
}
