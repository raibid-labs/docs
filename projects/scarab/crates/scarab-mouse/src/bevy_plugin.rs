//! Bevy plugin integration for mouse handling
//!
//! This provides the client-side Bevy systems that handle mouse input
//! and render selection/context menus.

use crate::{
    click_handler::{generate_cursor_position_sequence, generate_mouse_sequence, ClickDetector},
    context_menu::ContextMenu,
    selection::{find_word_at, Selection},
    types::{ClickType, MouseButton, MouseEvent, MouseEventKind, MouseMode, Modifiers, Position},
    ClickableItem, ClickableKind, MouseState,
};
use bevy::prelude::*;
use parking_lot::Mutex;
use std::sync::Arc;

/// Bevy plugin for mouse support
pub struct MousePlugin {
    state: Arc<Mutex<MouseState>>,
}

impl MousePlugin {
    pub fn new(state: Arc<Mutex<MouseState>>) -> Self {
        Self { state }
    }
}

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MousePluginState {
            shared_state: Arc::clone(&self.state),
            click_detector: ClickDetector::new(),
            drag_start: None,
            is_dragging: false,
        })
        .add_systems(
            Update,
            (
                handle_mouse_input,
                handle_scroll,
                update_selection_rendering,
                handle_context_menu_input,
            )
                .chain(),
        );
    }
}

/// Bevy resource for mouse plugin state
#[derive(Resource)]
struct MousePluginState {
    shared_state: Arc<Mutex<MouseState>>,
    click_detector: ClickDetector,
    drag_start: Option<Position>,
    is_dragging: bool,
}

/// Component for rendered selection overlay
#[derive(Component)]
struct SelectionOverlay;

/// Component for context menu UI
#[derive(Component)]
struct ContextMenuComponent;

/// System to handle mouse button input
fn handle_mouse_input(
    mut plugin_state: ResMut<MousePluginState>,
    mouse_button: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
    windows: Query<&Window>,
    mut commands: Commands,
) {
    let window = windows.single();
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Convert window coordinates to terminal grid coordinates
    // TODO: This needs actual font metrics from the terminal renderer
    let grid_pos = screen_to_grid(cursor_pos, window.width(), window.height());

    // Handle left mouse button
    if mouse_button.just_pressed(bevy::input::mouse::MouseButton::Left) {
        handle_left_click(&mut plugin_state, grid_pos);
    }

    if mouse_button.pressed(bevy::input::mouse::MouseButton::Left) {
        handle_left_drag(&mut plugin_state, grid_pos);
    }

    if mouse_button.just_released(bevy::input::mouse::MouseButton::Left) {
        handle_left_release(&mut plugin_state, grid_pos);
    }

    // Handle right mouse button (context menu)
    if mouse_button.just_pressed(bevy::input::mouse::MouseButton::Right) {
        handle_right_click(&mut plugin_state, grid_pos, &mut commands);
    }

    // Handle middle mouse button (paste)
    if mouse_button.just_pressed(bevy::input::mouse::MouseButton::Middle) {
        handle_middle_click(&mut plugin_state, grid_pos);
    }
}

/// Handle left mouse button click
fn handle_left_click(plugin_state: &mut MousePluginState, pos: Position) {
    let mut state = plugin_state.shared_state.lock();

    // Close context menu if open
    if state.context_menu_visible {
        state.context_menu_visible = false;
        return;
    }

    // Get keyboard modifiers
    // TODO: Get actual modifiers from Bevy input system
    let modifiers = Modifiers::none();

    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: pos,
        button: Some(MouseButton::Left),
        modifiers,
    };

    // Detect click type
    let click_type = plugin_state.click_detector.handle_press(&event);

    match state.mode {
        MouseMode::Application => {
            // Forward to application
            if let Some(seq) = generate_mouse_sequence(&event) {
                log::debug!("Sending mouse event to application: {:?}", seq);
                // TODO: Send to daemon via IPC
            }
        }
        MouseMode::Normal => {
            // Handle in Scarab
            match click_type {
                ClickType::Single => {
                    if modifiers.ctrl {
                        // Ctrl+Click - try to open URL/file
                        handle_ctrl_click(&state, pos);
                    } else if modifiers.shift {
                        // Shift+Click - extend selection
                        extend_selection(&mut state, pos);
                    } else {
                        // Normal click - clear selection and position cursor
                        state.selection = None;
                        plugin_state.drag_start = Some(pos);

                        // Send cursor position to terminal
                        let seq = generate_cursor_position_sequence(pos);
                        log::debug!("Positioning cursor at {:?}: {:?}", pos, seq);
                        // TODO: Send to daemon via IPC
                    }
                }
                ClickType::Double => {
                    // Select word
                    select_word_at(&mut state, pos);
                }
                ClickType::Triple => {
                    // Select line
                    select_line_at(&mut state, pos);
                }
            }
        }
    }
}

/// Handle left mouse button drag
fn handle_left_drag(plugin_state: &mut MousePluginState, pos: Position) {
    let mut state = plugin_state.shared_state.lock();

    if state.mode == MouseMode::Normal {
        if let Some(start) = plugin_state.drag_start {
            // Start dragging if moved enough
            if !plugin_state.is_dragging && start.distance_to(&pos) > 2.0 {
                plugin_state.is_dragging = true;
            }

            if plugin_state.is_dragging {
                // Update selection
                state.selection = Some(Selection::character(start, pos));
            }
        }
    }
}

/// Handle left mouse button release
fn handle_left_release(plugin_state: &mut MousePluginState, pos: Position) {
    plugin_state.drag_start = None;
    plugin_state.is_dragging = false;

    let state = plugin_state.shared_state.lock();
    if state.mode == MouseMode::Application {
        let event = MouseEvent {
            kind: MouseEventKind::Release,
            position: pos,
            button: Some(MouseButton::Left),
            modifiers: Modifiers::none(),
        };

        if let Some(seq) = generate_mouse_sequence(&event) {
            log::debug!("Sending mouse release to application: {:?}", seq);
            // TODO: Send to daemon via IPC
        }
    }
}

/// Handle right mouse button click (context menu)
fn handle_right_click(
    plugin_state: &mut MousePluginState,
    pos: Position,
    _commands: &mut Commands,
) {
    let mut state = plugin_state.shared_state.lock();

    if state.mode == MouseMode::Normal {
        // Show context menu
        let has_selection = state.selection.is_some();

        // Check if clicking on a URL or file path
        let _menu = if let Some(item) = find_clickable_at(&state, pos) {
            match item.kind {
                ClickableKind::Url => ContextMenu::url_menu(pos, item.text.clone()),
                ClickableKind::FilePath => ContextMenu::file_menu(pos, item.text.clone()),
            }
        } else {
            ContextMenu::standard(pos, has_selection)
        };

        state.context_menu_visible = true;

        log::debug!("Showing context menu at {:?}", pos);
        // TODO: Spawn context menu UI entity
    } else {
        // Forward to application
        let event = MouseEvent {
            kind: MouseEventKind::Press,
            position: pos,
            button: Some(MouseButton::Right),
            modifiers: Modifiers::none(),
        };

        if let Some(seq) = generate_mouse_sequence(&event) {
            log::debug!("Sending right click to application: {:?}", seq);
            // TODO: Send to daemon via IPC
        }
    }
}

/// Handle middle mouse button click (paste)
fn handle_middle_click(plugin_state: &mut MousePluginState, pos: Position) {
    let state = plugin_state.shared_state.lock();

    if state.mode == MouseMode::Normal {
        log::info!("Middle click paste at {:?}", pos);
        // TODO: Integrate with clipboard plugin for X11 primary selection
    }
}

/// Handle Ctrl+Click to open URLs and files
fn handle_ctrl_click(state: &MouseState, pos: Position) {
    if let Some(item) = find_clickable_at(state, pos) {
        match item.kind {
            ClickableKind::Url => {
                log::info!("Opening URL: {}", item.text);
                open_url(&item.text);
            }
            ClickableKind::FilePath => {
                log::info!("Opening file: {}", item.text);
                open_file(&item.text);
            }
        }
    }
}

/// Extend selection to position (Shift+Click)
fn extend_selection(state: &mut MouseState, pos: Position) {
    if let Some(selection) = &mut state.selection {
        selection.update_end(pos);
    }
}

/// Select word at position
fn select_word_at(state: &mut MouseState, pos: Position) {
    // TODO: Get actual character at position from terminal grid
    let get_char = |_p: Position| Some('x'); // Placeholder

    if let Some((start, end)) = find_word_at(pos, get_char) {
        state.selection = Some(Selection::word(start, end));
        log::debug!("Selected word from {:?} to {:?}", start, end);
    }
}

/// Select line at position
fn select_line_at(state: &mut MouseState, pos: Position) {
    // TODO: Get actual terminal dimensions
    let cols = 80; // Placeholder

    state.selection = Some(Selection::line(pos.y, pos.y, cols));
    log::debug!("Selected line {}", pos.y);
}

/// Find clickable item at position
fn find_clickable_at(state: &MouseState, pos: Position) -> Option<&ClickableItem> {
    state.clickable_items.iter().find(|item| {
        pos.x >= item.start.x
            && pos.x <= item.end.x
            && pos.y >= item.start.y
            && pos.y <= item.end.y
    })
}

/// Open URL in default browser
fn open_url(url: &str) {
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();

    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd")
        .args(&["/C", "start", "", url])
        .spawn();
}

/// Open file in default application
fn open_file(path: &str) {
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(path).spawn();

    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(path).spawn();

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("explorer").arg(path).spawn();
}

/// System to handle scroll wheel input
fn handle_scroll(
    plugin_state: Res<MousePluginState>,
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let grid_pos = screen_to_grid(cursor_pos, window.width(), window.height());

    for event in scroll_events.read() {
        let state = plugin_state.shared_state.lock();

        let button = if event.y > 0.0 {
            MouseButton::ScrollUp
        } else {
            MouseButton::ScrollDown
        };

        let mouse_event = MouseEvent {
            kind: MouseEventKind::Scroll,
            position: grid_pos,
            button: Some(button),
            modifiers: Modifiers::none(),
        };

        match state.mode {
            MouseMode::Application => {
                if let Some(seq) = generate_mouse_sequence(&mouse_event) {
                    log::debug!("Sending scroll to application: {:?}", seq);
                    // TODO: Send to daemon via IPC
                }
            }
            MouseMode::Normal => {
                // Scroll scrollback buffer
                let lines = (event.y.abs() * 3.0) as i32;
                log::debug!("Scrolling {} lines", lines);
                // TODO: Send scroll command to daemon
            }
        }
    }
}

/// System to render selection overlay
fn update_selection_rendering(
    plugin_state: Res<MousePluginState>,
    mut commands: Commands,
    selection_query: Query<Entity, With<SelectionOverlay>>,
) {
    let state = plugin_state.shared_state.lock();

    // Clear existing selection overlays
    for entity in &selection_query {
        commands.entity(entity).despawn();
    }

    // Render new selection if exists
    if let Some(selection) = &state.selection {
        log::trace!("Rendering selection: {:?}", selection);
        // TODO: Create visual overlay entities for selection
        // This would spawn colored quads/sprites at the selected cell positions
    }
}

/// System to handle context menu keyboard input
fn handle_context_menu_input(
    plugin_state: Res<MousePluginState>,
    _keyboard: Res<ButtonInput<KeyCode>>,
) {
    let state = plugin_state.shared_state.lock();

    if !state.context_menu_visible {
        return;
    }

    // Handle arrow keys, Enter, Escape for context menu navigation
    // TODO: Implement context menu interaction
}

/// Convert screen coordinates to terminal grid coordinates
fn screen_to_grid(cursor_pos: Vec2, window_width: f32, window_height: f32) -> Position {
    // TODO: Use actual font metrics and terminal dimensions
    // This is a placeholder calculation
    let cols = 80;
    let rows = 24;

    let x = ((cursor_pos.x / window_width) * cols as f32) as u16;
    let y = ((cursor_pos.y / window_height) * rows as f32) as u16;

    Position::new(x.min(cols - 1), y.min(rows - 1))
}
