//! Screen modules
//!
//! Each screen represents a full-page view in the application.

pub mod home;
pub mod dashboard;
pub mod interactive;
pub mod settings;

// Re-export screens
pub use home::HomeScreen;
pub use dashboard::DashboardScreen;
pub use interactive::InteractiveScreen;
pub use settings::SettingsScreen;
