// Integration tests for navigation UI
#![cfg(test)]

use scarab_nav_client::ratatui_helper::NavRecorder;
use scarab_nav_client::v1::ElementType;
use scarab_nav_client::NavClient;

/// Test that NavClient can be created without SCARAB_NAV_SOCKET
#[tokio::test]
async fn test_nav_client_creation_without_socket() {
    let client = NavClient::new();
    // Should not panic, operates in no-op mode
    drop(client);
}

/// Test NavRecorder basic registration
#[test]
fn test_nav_recorder_registration() {
    let mut recorder = NavRecorder::new();

    // Register a button
    recorder.register(10, 5, 20, 3, ElementType::Button, "Submit", None);

    // Register an input
    recorder.register(
        10,
        10,
        30,
        1,
        ElementType::Input,
        "Username",
        Some("[Tab]".to_string()),
    );

    let layout = recorder.finish();

    assert_eq!(layout.elements.len(), 2);
    assert_eq!(layout.elements[0].description, "Submit");
    assert_eq!(layout.elements[1].description, "Username");
    assert_eq!(layout.elements[1].key_hint, "[Tab]");
}

/// Test that element coordinates are preserved
#[test]
fn test_element_coordinates() {
    let mut recorder = NavRecorder::new();

    recorder.register(15, 20, 25, 5, ElementType::ListItem, "Item 1", None);

    let layout = recorder.finish();
    let element = &layout.elements[0];

    assert_eq!(element.x, 15);
    assert_eq!(element.y, 20);
    assert_eq!(element.width, 25);
    assert_eq!(element.height, 5);
}

/// Test different element types
#[test]
fn test_element_types() {
    let mut recorder = NavRecorder::new();

    recorder.register(0, 0, 10, 1, ElementType::Button, "Button", None);
    recorder.register(0, 1, 10, 1, ElementType::Input, "Input", None);
    recorder.register(0, 2, 10, 1, ElementType::Link, "Link", None);
    recorder.register(0, 3, 10, 1, ElementType::ListItem, "ListItem", None);
    recorder.register(0, 4, 10, 1, ElementType::Tab, "Tab", None);
    recorder.register(0, 5, 10, 1, ElementType::Text, "Text", None);

    let layout = recorder.finish();

    assert_eq!(layout.elements.len(), 6);
    assert_eq!(layout.elements[0].r#type, ElementType::Button as i32);
    assert_eq!(layout.elements[1].r#type, ElementType::Input as i32);
    assert_eq!(layout.elements[2].r#type, ElementType::Link as i32);
    assert_eq!(layout.elements[3].r#type, ElementType::ListItem as i32);
    assert_eq!(layout.elements[4].r#type, ElementType::Tab as i32);
    assert_eq!(layout.elements[5].r#type, ElementType::Text as i32);
}

/// Test window ID generation
#[test]
fn test_window_id_generation() {
    let recorder = NavRecorder::new();
    let layout = recorder.finish();

    // Window ID should be the process ID
    let expected_id = std::process::id().to_string();
    assert_eq!(layout.window_id, expected_id);
}

/// Snapshot test: Simple button layout
/// This test creates a visual snapshot that can be reviewed
#[test]
fn test_snapshot_simple_button_layout() {
    // Note: ratatui-testlib snapshot functionality
    // This is a placeholder for actual snapshot testing
    // The actual implementation would use ratatui-testlib's snapshot features

    let mut recorder = NavRecorder::new();

    recorder.register(
        5,
        5,
        20,
        3,
        ElementType::Button,
        "Click Me",
        Some("[Enter]".to_string()),
    );

    let layout = recorder.finish();

    // Verify the layout structure
    assert_eq!(layout.elements.len(), 1);
    assert_eq!(layout.elements[0].description, "Click Me");
}

/// Snapshot test: Navigation overlay with multiple elements
#[test]
fn test_snapshot_multiple_elements() {
    let mut recorder = NavRecorder::new();

    // Register multiple interactive elements
    recorder.register(
        2,
        2,
        15,
        1,
        ElementType::Tab,
        "Home",
        Some("[1]".to_string()),
    );
    recorder.register(
        18,
        2,
        15,
        1,
        ElementType::Tab,
        "Settings",
        Some("[2]".to_string()),
    );
    recorder.register(
        34,
        2,
        15,
        1,
        ElementType::Tab,
        "Help",
        Some("[3]".to_string()),
    );

    recorder.register(5, 10, 30, 1, ElementType::Input, "Search...", None);
    recorder.register(40, 10, 10, 3, ElementType::Button, "Search", None);

    let layout = recorder.finish();

    assert_eq!(layout.elements.len(), 5);

    // Verify tabs
    assert_eq!(layout.elements[0].r#type, ElementType::Tab as i32);
    assert_eq!(layout.elements[1].r#type, ElementType::Tab as i32);
    assert_eq!(layout.elements[2].r#type, ElementType::Tab as i32);

    // Verify search UI
    assert_eq!(layout.elements[3].r#type, ElementType::Input as i32);
    assert_eq!(layout.elements[4].r#type, ElementType::Button as i32);
}

/// Snapshot test: Link hints in a list
#[test]
fn test_snapshot_link_hints_list() {
    let mut recorder = NavRecorder::new();

    // Simulate a list of links
    for i in 0..5 {
        recorder.register(
            5,
            5 + i as u16 * 2,
            40,
            1,
            ElementType::Link,
            format!("https://example.com/item-{}", i),
            None,
        );
    }

    let layout = recorder.finish();

    assert_eq!(layout.elements.len(), 5);

    for (i, element) in layout.elements.iter().enumerate() {
        assert_eq!(element.r#type, ElementType::Link as i32);
        assert_eq!(
            element.description,
            format!("https://example.com/item-{}", i)
        );
        assert_eq!(element.y, 5 + i as u32 * 2);
    }
}

/// Test empty recorder
#[test]
fn test_empty_recorder() {
    let recorder = NavRecorder::new();
    let layout = recorder.finish();

    assert_eq!(layout.elements.len(), 0);
    assert!(!layout.window_id.is_empty());
}

/// Test recorder with large number of elements
#[test]
fn test_recorder_many_elements() {
    let mut recorder = NavRecorder::new();

    // Register 100 elements
    for i in 0..100 {
        recorder.register(
            (i % 80) as u16,
            (i / 80) as u16,
            1,
            1,
            ElementType::ListItem,
            format!("Item {}", i),
            None,
        );
    }

    let layout = recorder.finish();
    assert_eq!(layout.elements.len(), 100);
}

/// Test that NavClient can send updates without blocking
#[tokio::test]
async fn test_nav_client_non_blocking_update() {
    let client = NavClient::new();
    let mut recorder = NavRecorder::new();

    recorder.register(10, 10, 20, 3, ElementType::Button, "Test", None);

    let layout = recorder.finish();

    // This should not block even without a socket
    client.update(layout);

    // Give background task a moment
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
}

/// Test Clone implementation of NavClient
#[tokio::test]
async fn test_nav_client_clone() {
    let client1 = NavClient::new();
    let client2 = client1.clone();

    let mut recorder = NavRecorder::new();
    recorder.register(0, 0, 10, 1, ElementType::Button, "Test", None);
    let layout = recorder.finish();

    // Both clones should work
    client1.update(layout.clone());
    client2.update(layout);
}
