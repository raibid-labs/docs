use crate::core::overlay::OverlayState;
use crate::core::targets::TargetRegistry;
use crate::core::plugin::LocustPlugin;
use crate::core::input::{LocustEventOutcome, PluginEventResult};
use crossterm::event::Event;
use ratatui::backend::Backend;
use ratatui::prelude::Frame;

/// Global configuration for Locust.
#[derive(Debug, Clone)]
pub struct LocustConfig {
    /// Whether the built-in navigation plugin should be registered by default.
    pub enable_nav_plugin: bool,
}

impl Default for LocustConfig {
    fn default() -> Self {
        Self {
            enable_nav_plugin: true,
        }
    }
}

/// Shared state passed to plugins: targets, overlay bookkeeping, etc.
#[derive(Debug, Default)]
pub struct LocustContext {
    pub targets: TargetRegistry,
    pub overlay: OverlayState,
}

/// Central entry point for embedding Locust into a ratatui app.
pub struct Locust<B>
where
    B: Backend,
{
    pub config: LocustConfig,
    pub ctx: LocustContext,
    plugins: Vec<Box<dyn LocustPlugin<B>>>,
}

impl<B> Locust<B>
where
    B: Backend,
{
    /// Create a new Locust instance with optional configuration.
    ///
    /// The caller is responsible for registering plugins. A convenience
    /// constructor that installs the built-in navigation plugin lives in
    /// the `plugins` module.
    pub fn new(config: LocustConfig) -> Self {
        Self {
            config,
            ctx: LocustContext::default(),
            plugins: Vec::new(),
        }
    }

    /// Register a plugin. Its `init` hook will be called immediately.
    pub fn register_plugin<P>(&mut self, mut plugin: P)
    where
        P: LocustPlugin<B> + 'static,
    {
        plugin.init(&mut self.ctx);
        self.plugins.push(Box::new(plugin));
    }

    /// Offer an input event to all plugins in registration order.
    ///
    /// Returns whether the event was consumed and whether the caller
    /// should trigger a redraw.
    pub fn on_event(&mut self, event: &Event) -> LocustEventOutcome {
        let mut results = Vec::with_capacity(self.plugins.len());
        for plugin in self.plugins.iter_mut() {
            let res = plugin.on_event(event, &mut self.ctx);
            results.push(res);
            if res.is_consumed() {
                // Stop on first consumer; plugins are ordered.
                break;
            }
        }

        use crate::core::input::LocustEventOutcome as Outcome;
        if results.is_empty() {
            Outcome::NOT_HANDLED
        } else {
            let consumed = results.iter().any(|r| r.is_consumed());
            let request_redraw = results.iter().any(|r| r.requests_redraw());
            Outcome { consumed, request_redraw }
        }
    }

    /// Clear any per-frame state before drawing.
    pub fn begin_frame(&mut self) {
        self.ctx.targets.clear();
        self.ctx.overlay.begin_frame();
    }

    /// Ask all plugins to render their overlays on top of the frame.
    ///
    /// This should be called *after* the application has rendered its
    /// widgets for the current frame.
    pub fn render_overlay(&self, frame: &mut Frame<'_, B>) {
        for plugin in self.plugins.iter() {
            plugin.render_overlay(frame, &self.ctx);
        }
    }
}
