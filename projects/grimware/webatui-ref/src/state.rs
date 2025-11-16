//! Application state management
//!
//! Defines the state structure and message types for state updates.

use serde::{Deserialize, Serialize};

#[cfg(feature = "terminal")]
use crate::components::metrics_state::MetricsState;

/// Main application state
///
/// This structure holds all the application's state that needs to be
/// synchronized across different screens and components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    /// Current screen being displayed
    pub current_screen: Screen,

    /// Application title
    pub title: String,

    /// Counter for example interactions
    pub counter: i32,

    /// List of items for demonstration
    pub items: Vec<String>,

    /// Selected item index
    pub selected_index: usize,

    /// System metrics (not serialized due to complexity, only available with terminal feature)
    #[cfg(feature = "terminal")]
    #[serde(skip)]
    pub metrics: Option<MetricsState>,

    /// Selected process index in dashboard
    pub selected_process: usize,

    /// Flag to indicate if the application should quit
    pub should_quit: bool,

    /// Input buffer for user input
    pub input: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_screen: Screen::Home,
            title: String::from("WebATUI Reference"),
            counter: 0,
            items: vec![
                "Item 1".to_string(),
                "Item 2".to_string(),
                "Item 3".to_string(),
            ],
            selected_index: 0,
            #[cfg(feature = "terminal")]
            metrics: Some(MetricsState::default()),
            selected_process: 0,
            should_quit: false,
            input: String::new(),
        }
    }
}

impl AppState {
    /// Update the application state based on a message
    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::Quit => {
                self.should_quit = true;
            }
            Message::Navigate(screen) => {
                self.current_screen = screen;
            }
            Message::NextScreen => {
                self.current_screen = match self.current_screen {
                    Screen::Home => Screen::Dashboard,
                    Screen::Dashboard => Screen::Interactive,
                    Screen::Interactive => Screen::Settings,
                    Screen::Settings => Screen::Home,
                };
            }
            Message::PrevScreen => {
                self.current_screen = match self.current_screen {
                    Screen::Home => Screen::Settings,
                    Screen::Dashboard => Screen::Home,
                    Screen::Interactive => Screen::Dashboard,
                    Screen::Settings => Screen::Interactive,
                };
            }
            Message::Increment => {
                self.counter += 1;
            }
            Message::Decrement => {
                self.counter -= 1;
            }
            Message::SelectNext => {
                if !self.items.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.items.len();
                }
            }
            Message::SelectPrevious => {
                if !self.items.is_empty() {
                    self.selected_index = if self.selected_index == 0 {
                        self.items.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                }
            }
            Message::AddItem(item) => {
                self.items.push(item);
            }
            Message::RemoveItem => {
                if !self.items.is_empty() && self.selected_index < self.items.len() {
                    self.items.remove(self.selected_index);
                    if self.selected_index >= self.items.len() && !self.items.is_empty() {
                        self.selected_index = self.items.len() - 1;
                    }
                }
            }
            Message::UpdateMetrics => {
                #[cfg(feature = "terminal")]
                {
                    // Metrics update logic would go here
                    // For now, metrics are updated on-demand via MetricsState::new()
                }
            }
            Message::SelectNextProcess => {
                #[cfg(feature = "terminal")]
                {
                    if let Some(ref metrics) = self.metrics {
                        let process_count = metrics.processes.len();
                        if process_count > 0 {
                            self.selected_process = (self.selected_process + 1) % process_count;
                        }
                    }
                }
            }
            Message::SelectPreviousProcess => {
                #[cfg(feature = "terminal")]
                {
                    if let Some(ref metrics) = self.metrics {
                        let process_count = metrics.processes.len();
                        if process_count > 0 {
                            self.selected_process = if self.selected_process == 0 {
                                process_count - 1
                            } else {
                                self.selected_process - 1
                            };
                        }
                    }
                }
            }
            Message::RefreshDashboard => {
                #[cfg(feature = "terminal")]
                {
                    // Dashboard refresh logic would go here
                    // Metrics are refreshed automatically when rendered
                }
            }
            Message::Input(text) => {
                self.input = text;
            }
        }
    }
}

/// Available screens in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Screen {
    /// Home/landing screen
    Home,

    /// Dashboard with multiple widgets
    Dashboard,

    /// Interactive example screen
    Interactive,

    /// Settings screen
    Settings,
}

impl Screen {
    /// Get the display name for the screen
    pub fn name(&self) -> &'static str {
        match self {
            Screen::Home => "Home",
            Screen::Dashboard => "Dashboard",
            Screen::Interactive => "Interactive",
            Screen::Settings => "Settings",
        }
    }
}

/// Messages for state updates
///
/// These messages represent all possible state changes in the application.
#[derive(Debug, Clone)]
pub enum Message {
    /// Quit the application
    Quit,

    /// Navigate to a specific screen
    Navigate(Screen),

    /// Increment counter
    Increment,

    /// Decrement counter
    Decrement,

    /// Select next item
    SelectNext,

    /// Select previous item
    SelectPrevious,

    /// Add a new item
    AddItem(String),

    /// Remove selected item
    RemoveItem,

    /// Update metrics (for dashboard)
    UpdateMetrics,

    /// Select next process in dashboard
    SelectNextProcess,

    /// Select previous process in dashboard
    SelectPreviousProcess,

    /// Refresh dashboard data
    RefreshDashboard,

    /// Navigate to next screen
    NextScreen,

    /// Navigate to previous screen
    PrevScreen,

    /// Text input
    Input(String),
}
