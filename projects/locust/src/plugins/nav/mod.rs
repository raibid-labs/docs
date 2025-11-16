//! Locust navigation plugin.
//!
//! This plugin is responsible for:
//! - Managing navigation modes (normal, hint, etc.).
//! - Consuming key events that control navigation.
//! - Rendering hint overlays on top of the UI.

use crate::core::context::LocustContext;
use crate::core::input::PluginEventResult;
use crate::core::overlay::OverlayState;
use crate::core::plugin::LocustPlugin;
use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::backend::Backend;
use ratatui::prelude::Frame;

/// Current navigation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavMode {
    Normal,
    Hint,
    // Future: Find, Command, Visual, etc.
}

/// Simple navigation plugin implementing a Vimium-style "press `f` to show hints" flow.
pub struct NavPlugin {
    pub mode: NavMode,
}

impl NavPlugin {
    pub fn new() -> Self {
        Self { mode: NavMode::Normal }
    }
}

impl<B> LocustPlugin<B> for NavPlugin
where
    B: Backend,
{
    fn id(&self) -> &'static str {
        "locust.nav"
    }

    fn init(&mut self, _ctx: &mut LocustContext) {
        // Nothing to do yet; hook exists for future use.
    }

    fn on_event(
        &mut self,
        event: &Event,
        ctx: &mut LocustContext,
    ) -> PluginEventResult {
        if let Event::Key(KeyEvent { code, .. }) = event {
            match (self.mode, code) {
                (NavMode::Normal, KeyCode::Char('f')) => {
                    // Enter hint mode
                    self.mode = NavMode::Hint;
                    ctx.overlay.mark_has_overlay();
                    return PluginEventResult::ConsumedRequestRedraw;
                }
                (NavMode::Hint, KeyCode::Esc) => {
                    // Exit hint mode
                    self.mode = NavMode::Normal;
                    return PluginEventResult::ConsumedRequestRedraw;
                }
                _ => {}
            }
        }

        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame<'_, B>, ctx: &LocustContext) {
        if self.mode != NavMode::Hint {
            return;
        }

        // Placeholder: draw a simple banner at the top when in hint mode.
        // Later, this will render per-target hint labels.
        use ratatui::widgets::{Block, Borders, Paragraph};
        use ratatui::layout::Rect;
        use ratatui::text::{Line, Span};

        let area = {
            let size = frame.size();
            Rect {
                x: size.x,
                y: size.y,
                width: size.width,
                height: 1,
            }
        };

        let text = Line::from(vec![
            Span::raw(" Locust: Hint mode active (press Esc to exit) "),
        ]);
        let para = Paragraph::new(text).block(Block::default().borders(Borders::BOTTOM));
        frame.render_widget(para, area);
    }
}
