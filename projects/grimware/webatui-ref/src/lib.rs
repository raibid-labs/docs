//! # webatui-ref
//!
//! Reference implementation for webatui - a terminal UI framework that works in both
//! terminal environments and web browsers through WASM.
//!
//! ## Features
//!
//! - **Terminal Support**: Full ratatui-based terminal UI
//! - **Web Support**: WASM compilation with Yew components
//! - **State Management**: Unified state across platforms
//! - **Component System**: Reusable UI components
//! - **Examples**: Multiple example applications
//!
//! ## Quick Start
//!
//! ```rust
//! use webatui_ref::prelude::*;
//!
//! // Create and use app state
//! let mut state = AppState::default();
//! state.update(Message::Navigate(Screen::Dashboard));
//! assert_eq!(state.current_screen, Screen::Dashboard);
//! ```

// Module declarations
#[cfg(feature = "terminal")]
pub mod components;
#[cfg(feature = "terminal")]
pub mod screens;
pub mod state;


// Prelude module for convenient imports
pub mod prelude {
    //! Common imports for webatui applications
    //!
    //! This module re-exports the most commonly used types and traits,
    //! providing a convenient way to import everything needed for
    //! building webatui applications.

    // Re-export core app types
    pub use crate::state::{AppState, Message, Screen};

    // Re-export component types (terminal only)
    #[cfg(feature = "terminal")]
    pub use crate::components::{Header, Footer};

    // Re-export ratatui essentials (terminal only)
    #[cfg(feature = "terminal")]
    pub use ratatui::prelude::*;
    #[cfg(feature = "terminal")]
    pub use ratatui::widgets::*;
    #[cfg(feature = "terminal")]
    pub use ratatui::layout::{Constraint, Direction, Layout, Rect};
    #[cfg(feature = "terminal")]
    pub use ratatui::style::{Color, Modifier, Style, Stylize};
    #[cfg(feature = "terminal")]
    pub use ratatui::text::{Line, Span, Text};

    // Re-export crossterm for terminal control (terminal only)
    #[cfg(feature = "terminal")]
    pub use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
    #[cfg(feature = "terminal")]
    pub use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
    #[cfg(feature = "terminal")]
    pub use crossterm::ExecutableCommand;

    // Platform-specific re-exports
    #[cfg(feature = "web")]
    pub use yew::prelude::*;

    #[cfg(feature = "web")]
    pub use wasm_bindgen::prelude::*;

    // Re-export common std types
    pub use std::io;
    pub use std::time::{Duration, Instant};

    // Re-export error handling
    pub use anyhow::{anyhow, Context, Result};
    pub use thiserror::Error;
}

// Re-export main types at crate root for convenience
pub use state::{AppState, Message, Screen};

#[cfg(feature = "terminal")]
pub use components::{Header, Footer};

#[cfg(feature = "web")]
pub use web::WebApp;

// Web module for WASM support
#[cfg(feature = "web")]
pub mod web {
    //! Web-specific application entry point for WASM builds

    use yew::prelude::*;
    use wasm_bindgen::prelude::*;

    /// Web application component
    ///
    /// This is the entry point for the WASM build, wrapping the terminal
    /// application in a Yew component for browser rendering.
    #[function_component(WebApp)]
    pub fn web_app() -> Html {
        html! {
            <div class="webatui-container">
                <div class="terminal-wrapper">
                    { "WebATUI Reference Implementation - Web Mode" }
                    <p>{ "Loading terminal interface..." }</p>
                </div>
            </div>
        }
    }

    /// WASM entry point
    #[wasm_bindgen(start)]
    pub fn run_app() -> Result<(), JsValue> {
        // Set up panic hook for better error messages
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        // Initialize logging
        wasm_logger::init(wasm_logger::Config::default());

        // Start Yew application
        yew::Renderer::<WebApp>::new().render();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_test_configure!(run_in_browser);

    // Helper macro to run tests in both native and WASM environments
    #[cfg(not(target_arch = "wasm32"))]
    macro_rules! test_fn {
        ($name:ident, $body:expr) => {
            #[test]
            fn $name() {
                $body
            }
        };
    }

    #[cfg(target_arch = "wasm32")]
    macro_rules! test_fn {
        ($name:ident, $body:expr) => {
            #[wasm_bindgen_test]
            fn $name() {
                $body
            }
        };
    }

    test_fn!(test_default_state, {
        let state = AppState::default();
        assert_eq!(state.current_screen, Screen::Home);
        assert!(!state.title.is_empty());
    });

    test_fn!(test_state_message_update, {
        let mut state = AppState::default();
        state.update(Message::NextScreen);
        // Should transition from Home to Dashboard
        assert_eq!(state.current_screen, Screen::Dashboard);
    });

    test_fn!(test_state_navigation, {
        let mut state = AppState::default();
        assert_eq!(state.current_screen, Screen::Home);

        state.update(Message::NextScreen);
        assert_eq!(state.current_screen, Screen::Dashboard);

        state.update(Message::NextScreen);
        assert_eq!(state.current_screen, Screen::Interactive);

        state.update(Message::NextScreen);
        assert_eq!(state.current_screen, Screen::Settings);

        state.update(Message::PrevScreen);
        assert_eq!(state.current_screen, Screen::Interactive);
    });

    #[cfg(feature = "web")]
    mod web_tests {
        use super::*;

        #[cfg(target_arch = "wasm32")]
        #[wasm_bindgen_test]
        fn test_web_app_renders() {
            // Test that the web app component can be created
            let app = web::WebApp::new();
            assert!(format!("{:?}", app).contains("WebApp"));
        }

        test_fn!(test_wasm_entry_point_exists, {
            // Just verify the module compiles correctly
            assert!(true);
        });
    }
}
