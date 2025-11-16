//! WASM-specific integration tests
//!
//! These tests are designed to run in a browser environment using wasm-bindgen-test

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;
use webatui_ref::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_wasm_state_creation() {
    let state = AppState::default();
    assert_eq!(state.current_screen, Screen::Home);
}

#[wasm_bindgen_test]
fn test_wasm_state_transitions() {
    let mut state = AppState::default();

    // Test forward navigation
    state.update(Message::NextScreen);
    assert_eq!(state.current_screen, Screen::Dashboard);

    state.update(Message::NextScreen);
    assert_eq!(state.current_screen, Screen::Settings);

    // Test backward navigation
    state.update(Message::PrevScreen);
    assert_eq!(state.current_screen, Screen::Dashboard);
}

#[wasm_bindgen_test]
fn test_wasm_message_handling() {
    let mut state = AppState::default();

    // Test quit message
    state.update(Message::Quit);
    assert!(state.should_quit);

    // Test input message
    let test_input = "test input".to_string();
    state.update(Message::Input(test_input.clone()));
    assert_eq!(state.input, test_input);
}

#[cfg(feature = "web")]
mod web_feature_tests {
    use super::*;
    use webatui_ref::web::*;

    #[wasm_bindgen_test]
    fn test_web_app_initialization() {
        // Test that WebApp can be created without panicking
        let _app = WebApp::new();
    }

    #[wasm_bindgen_test]
    fn test_wasm_bindgen_entry_point() {
        // Test that the WASM entry point can be called
        // Note: This won't actually render in test environment
        let result = run_app();
        assert!(result.is_ok());
    }
}

#[wasm_bindgen_test]
fn test_serialization() {
    let state = AppState::default();
    let serialized = serde_json::to_string(&state);
    assert!(serialized.is_ok());

    let deserialized: Result<AppState, _> = serde_json::from_str(&serialized.unwrap());
    assert!(deserialized.is_ok());
}
