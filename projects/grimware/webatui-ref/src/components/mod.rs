//! UI Components module
//!
//! This module contains reusable UI components for the webatui application.
//! Each component implements the ratatui Widget trait for rendering.

pub mod header;
pub mod footer;
pub mod list;
pub mod counter;
pub mod gauge;
pub mod chart;
pub mod metrics_state;

// Re-export component types for convenience
pub use header::Header;
pub use footer::Footer;
pub use list::List;
pub use counter::Counter;
pub use gauge::GaugeWidget;
pub use chart::{ChartWidget, NetworkChart};
pub use metrics_state::{MetricsState, ProcessInfo, ProcessStatus};
