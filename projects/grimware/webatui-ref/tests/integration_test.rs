//! Integration tests for webatui-ref
//!
//! These tests verify that components work correctly together

#![cfg(not(target_arch = "wasm32"))]

use webatui_ref::prelude::*;

#[test]
fn test_app_state_lifecycle() {
    let mut state = AppState::default();

    // Test initial state
    assert_eq!(state.current_screen, Screen::Home);
    assert!(!state.should_quit);
    assert!(state.input.is_empty());

    // Test navigation through all screens
    state.update(Message::NextScreen);
    assert_eq!(state.current_screen, Screen::Dashboard);

    state.update(Message::NextScreen);
    assert_eq!(state.current_screen, Screen::Interactive);

    state.update(Message::NextScreen);
    assert_eq!(state.current_screen, Screen::Settings);

    // Test quit functionality
    state.update(Message::Quit);
    assert!(state.should_quit);
}

#[cfg(feature = "terminal")]
#[test]
fn test_header_component() {
    let header = Header::new("Test App", Screen::Home);
    // Header can be created successfully
    assert!(true);
}

#[cfg(feature = "terminal")]
#[test]
fn test_footer_component() {
    let footer = Footer::new(Screen::Home);
    // Footer can be created successfully
    assert!(true);
}

#[test]
fn test_state_serialization() {
    let state = AppState::default();

    // Test JSON serialization
    let json = serde_json::to_string(&state).expect("Failed to serialize");
    assert!(!json.is_empty());

    // Test deserialization
    let deserialized: AppState = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.current_screen, state.current_screen);
    assert_eq!(deserialized.title, state.title);
}

#[test]
fn test_message_handling() {
    let mut state = AppState::default();

    // Test input message
    state.update(Message::Input("hello".to_string()));
    assert_eq!(state.input, "hello");

    // Test clear message
    state.update(Message::Input("".to_string()));
    assert!(state.input.is_empty());
}

#[tokio::test]
async fn test_async_state_operations() {
    let state = AppState::default();

    // Test that state can be used in async context
    let result = tokio::spawn(async move {
        state.current_screen
    }).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Screen::Home);
}

#[test]
fn test_screen_ordering() {
    use std::mem::discriminant;

    let screens = vec![Screen::Home, Screen::Dashboard, Screen::Settings];

    // Verify each screen is unique
    for (i, screen1) in screens.iter().enumerate() {
        for (j, screen2) in screens.iter().enumerate() {
            if i == j {
                assert_eq!(discriminant(screen1), discriminant(screen2));
            } else {
                assert_ne!(discriminant(screen1), discriminant(screen2));
            }
        }
    }
}
