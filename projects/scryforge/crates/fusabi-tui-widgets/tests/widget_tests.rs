use fusabi_streams_core::{Item, ItemContent, ItemId, Stream, StreamId, StreamType};
use fusabi_tui_widgets::prelude::*;
use ratatui::{backend::TestBackend, layout::Rect, Terminal};
use std::collections::HashMap;

/// Helper to create a test terminal with a specific size
fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

/// Helper to create a dummy stream for testing
fn create_test_stream(name: &str, unread_count: Option<u32>) -> Stream {
    Stream {
        id: StreamId(format!("test-stream-{}", name)),
        name: name.to_string(),
        provider_id: "test-provider".to_string(),
        stream_type: StreamType::Feed,
        icon: None,
        unread_count,
        total_count: None,
        last_updated: None,
        metadata: HashMap::new(),
    }
}

/// Helper to create a dummy item for testing
fn create_test_item(title: &str, is_read: bool, author_name: Option<&str>) -> Item {
    Item {
        id: ItemId(format!("test-item-{}", title)),
        stream_id: StreamId("test-stream".to_string()),
        title: title.to_string(),
        content: ItemContent::Text("Test content".to_string()),
        author: author_name.map(|name| fusabi_streams_core::Author {
            name: name.to_string(),
            email: None,
            url: None,
            avatar_url: None,
        }),
        published: None,
        updated: None,
        url: None,
        thumbnail_url: None,
        is_read,
        is_saved: false,
        tags: vec![],
        metadata: HashMap::new(),
    }
}

#[test]
fn test_stream_list_widget_renders() {
    let mut terminal = create_test_terminal(40, 10);
    let theme = Theme::default();

    let streams = vec![
        create_test_stream("Inbox", Some(5)),
        create_test_stream("RSS Feed", Some(0)),
        create_test_stream("Spotify", None),
    ];

    terminal
        .draw(|frame| {
            let widget = StreamListWidget::new(&streams, Some(0), &theme).focused(true);
            widget.render(frame, frame.area());
        })
        .unwrap();

    // Verify the widget rendered without panicking
    // We can't easily assert on buffer contents in a meaningful way without
    // making the test brittle, but we can verify the render completes
}

#[test]
fn test_stream_list_widget_empty() {
    let mut terminal = create_test_terminal(40, 10);
    let theme = Theme::default();
    let streams: Vec<Stream> = vec![];

    terminal
        .draw(|frame| {
            let widget = StreamListWidget::new(&streams, None, &theme).focused(false);
            widget.render(frame, frame.area());
        })
        .unwrap();

    // Should render without panicking even with no streams
}

#[test]
fn test_stream_list_widget_focus_state() {
    let mut terminal = create_test_terminal(40, 10);
    let theme = Theme::default();
    let streams = vec![create_test_stream("Test", None)];

    // Test unfocused state
    terminal
        .draw(|frame| {
            let widget = StreamListWidget::new(&streams, Some(0), &theme).focused(false);
            widget.render(frame, frame.area());
        })
        .unwrap();

    // Test focused state
    terminal
        .draw(|frame| {
            let widget = StreamListWidget::new(&streams, Some(0), &theme).focused(true);
            widget.render(frame, frame.area());
        })
        .unwrap();
}

#[test]
fn test_item_list_widget_renders() {
    let mut terminal = create_test_terminal(80, 20);
    let theme = Theme::default();

    let items = vec![
        create_test_item("First Item", false, Some("Author One")),
        create_test_item("Second Item", true, Some("Author Two")),
        create_test_item("Third Item", false, None),
    ];

    terminal
        .draw(|frame| {
            let widget = ItemListWidget::new(&items, Some(0), &theme).focused(true);
            widget.render(frame, frame.area());
        })
        .unwrap();
}

#[test]
fn test_item_list_widget_empty() {
    let mut terminal = create_test_terminal(80, 20);
    let theme = Theme::default();
    let items: Vec<Item> = vec![];

    terminal
        .draw(|frame| {
            let widget = ItemListWidget::new(&items, None, &theme).focused(false);
            widget.render(frame, frame.area());
        })
        .unwrap();
}

#[test]
fn test_item_list_widget_unread_indicator() {
    let mut terminal = create_test_terminal(80, 20);
    let theme = Theme::default();

    let items = vec![
        create_test_item("Unread Item", false, None),
        create_test_item("Read Item", true, None),
    ];

    terminal
        .draw(|frame| {
            let widget = ItemListWidget::new(&items, Some(0), &theme);
            widget.render(frame, frame.area());
        })
        .unwrap();
}

#[test]
fn test_preview_widget_renders_with_item() {
    let mut terminal = create_test_terminal(80, 20);
    let theme = Theme::default();

    let item = create_test_item("Test Article", false, Some("Test Author"));

    terminal
        .draw(|frame| {
            let widget = PreviewWidget::new(Some(&item), &theme).focused(true);
            widget.render(frame, frame.area());
        })
        .unwrap();
}

#[test]
fn test_preview_widget_renders_without_item() {
    let mut terminal = create_test_terminal(80, 20);
    let theme = Theme::default();

    terminal
        .draw(|frame| {
            let widget = PreviewWidget::new(None, &theme).focused(false);
            widget.render(frame, frame.area());
        })
        .unwrap();
}

#[test]
fn test_preview_widget_different_content_types() {
    let mut terminal = create_test_terminal(80, 20);
    let theme = Theme::default();

    // Test different content types
    let content_types = vec![
        ItemContent::Text("Plain text content".to_string()),
        ItemContent::Markdown("# Markdown Header\n\nContent".to_string()),
        ItemContent::Html("<p>HTML content</p>".to_string()),
        ItemContent::Article {
            summary: Some("Summary".to_string()),
            full_content: Some("Full content".to_string()),
        },
        ItemContent::Video {
            description: "Video description".to_string(),
            duration_seconds: Some(120),
            view_count: None,
        },
    ];

    for content in content_types {
        let mut item = create_test_item("Test", false, None);
        item.content = content;

        terminal
            .draw(|frame| {
                let widget = PreviewWidget::new(Some(&item), &theme);
                widget.render(frame, frame.area());
            })
            .unwrap();
    }
}

#[test]
fn test_status_bar_widget_renders() {
    let mut terminal = create_test_terminal(80, 1);
    let theme = Theme::default();

    terminal
        .draw(|frame| {
            let widget = StatusBarWidget::new("Ready", "Connected", &theme);
            widget.render(frame, frame.area());
        })
        .unwrap();
}

#[test]
fn test_omnibar_widget_renders() {
    let mut terminal = create_test_terminal(80, 3);
    let theme = Theme::default();

    // Test with empty input
    terminal
        .draw(|frame| {
            let widget = OmnibarWidget::new("", &theme).active(true);
            widget.render(frame, frame.area());
        })
        .unwrap();

    // Test with input
    terminal
        .draw(|frame| {
            let widget = OmnibarWidget::new("search query", &theme).active(false);
            widget.render(frame, frame.area());
        })
        .unwrap();
}

#[test]
fn test_omnibar_widget_placeholder() {
    let mut terminal = create_test_terminal(80, 3);
    let theme = Theme::default();

    terminal
        .draw(|frame| {
            let widget = OmnibarWidget::new("", &theme)
                .placeholder("Custom placeholder")
                .active(true);
            widget.render(frame, frame.area());
        })
        .unwrap();
}

#[test]
fn test_theme_default() {
    let theme = Theme::default();
    // Just verify theme creation works
    assert_eq!(theme.background, ratatui::style::Color::Reset);
}

#[test]
fn test_widgets_with_small_area() {
    let mut terminal = create_test_terminal(10, 5);
    let theme = Theme::default();

    let streams = vec![create_test_stream("Test", None)];
    let items = vec![create_test_item("Test", false, None)];

    // Verify widgets handle small rendering areas gracefully
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 5, 3);
            let widget = StreamListWidget::new(&streams, None, &theme);
            widget.render(frame, area);
        })
        .unwrap();

    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 5, 3);
            let widget = ItemListWidget::new(&items, None, &theme);
            widget.render(frame, area);
        })
        .unwrap();
}
