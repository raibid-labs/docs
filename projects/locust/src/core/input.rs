use crossterm::event::Event;

/// Result of giving an event to a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginEventResult {
    /// Plugin did not handle the event; it should be offered to others / the app.
    NotHandled,
    /// Plugin handled the event but no redraw is strictly required.
    Consumed,
    /// Plugin handled the event and the UI should be redrawn.
    ConsumedRequestRedraw,
}

impl PluginEventResult {
    pub fn is_consumed(self) -> bool {
        matches!(self, PluginEventResult::Consumed | PluginEventResult::ConsumedRequestRedraw)
    }

    pub fn requests_redraw(self) -> bool {
        matches!(self, PluginEventResult::ConsumedRequestRedraw)
    }
}

/// Indicates whether Locust consumed an event and whether the caller should redraw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocustEventOutcome {
    pub consumed: bool,
    pub request_redraw: bool,
}

impl LocustEventOutcome {
    pub const NOT_HANDLED: Self = Self { consumed: false, request_redraw: false };
    pub const CONSUMED: Self = Self { consumed: true, request_redraw: false };
    pub const CONSUMED_REDRAW: Self = Self { consumed: true, request_redraw: true };
}

// Tiny compile-time guard that Event is what we expect.
#[allow(dead_code)]
fn _assert_event_is_crossterm_event(_: &Event) {}
