// Visual selection mode for terminal text
// Allows users to select text using keyboard

use crate::integration::SharedMemoryReader;
use crate::rendering::text::TextRenderer;
use crate::ui::grid_utils::grid_to_pixel;
use arboard::Clipboard;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use scarab_protocol::{SharedState, GRID_HEIGHT, GRID_WIDTH};

/// Plugin for visual selection
pub struct VisualSelectionPlugin;

impl Plugin for VisualSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionState>()
            .add_event::<SelectionChangedEvent>()
            .add_event::<SelectionCopiedEvent>()
            .add_systems(
                Update,
                (
                    handle_selection_input_system,
                    render_selection_system,
                    copy_selection_system,
                ),
            );
    }
}

/// Selection mode type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SelectionMode {
    Character,
    Line,
    Block,
}

/// Region of selected text
#[derive(Clone, Debug, Default)]
pub struct SelectionRegion {
    pub start_x: u16,
    pub start_y: u16,
    pub end_x: u16,
    pub end_y: u16,
}

impl SelectionRegion {
    pub fn new(start_x: u16, start_y: u16, end_x: u16, end_y: u16) -> Self {
        Self {
            start_x,
            start_y,
            end_x,
            end_y,
        }
    }

    pub fn contains(&self, x: u16, y: u16) -> bool {
        let (min_x, max_x) = if self.start_x <= self.end_x {
            (self.start_x, self.end_x)
        } else {
            (self.end_x, self.start_x)
        };

        let (min_y, max_y) = if self.start_y <= self.end_y {
            (self.start_y, self.end_y)
        } else {
            (self.end_y, self.start_y)
        };

        x >= min_x && x <= max_x && y >= min_y && y <= max_y
    }

    pub fn is_empty(&self) -> bool {
        self.start_x == self.end_x && self.start_y == self.end_y
    }

    pub fn normalize(&mut self) {
        if self.start_y > self.end_y || (self.start_y == self.end_y && self.start_x > self.end_x) {
            std::mem::swap(&mut self.start_x, &mut self.end_x);
            std::mem::swap(&mut self.start_y, &mut self.end_y);
        }
    }
}

/// State of visual selection
#[derive(Resource)]
pub struct SelectionState {
    pub active: bool,
    pub mode: SelectionMode,
    pub region: SelectionRegion,
    pub cursor_x: u16,
    pub cursor_y: u16,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            active: false,
            mode: SelectionMode::Character,
            region: SelectionRegion::default(),
            cursor_x: 0,
            cursor_y: 0,
        }
    }
}

impl SelectionState {
    pub fn start_selection(&mut self, x: u16, y: u16, mode: SelectionMode) {
        self.active = true;
        self.mode = mode;
        self.region = SelectionRegion::new(x, y, x, y);
        self.cursor_x = x;
        self.cursor_y = y;
    }

    pub fn update_selection(&mut self, x: u16, y: u16) {
        if !self.active {
            return;
        }

        self.region.end_x = x;
        self.region.end_y = y;
        self.cursor_x = x;
        self.cursor_y = y;
    }

    pub fn end_selection(&mut self) {
        self.active = false;
    }

    pub fn clear(&mut self) {
        self.active = false;
        self.region = SelectionRegion::default();
    }
}

/// Event fired when selection changes
#[derive(Event)]
pub struct SelectionChangedEvent {
    pub region: SelectionRegion,
}

/// Event fired when selection is copied
#[derive(Event)]
pub struct SelectionCopiedEvent {
    pub text: String,
}

/// Component for selection overlay
#[derive(Component)]
struct SelectionOverlay;

/// Extract text from selection region using SharedMemoryReader
fn extract_selection_text(
    state_reader: &SharedMemoryReader,
    region: &SelectionRegion,
    mode: SelectionMode,
) -> String {
    let shared_ptr = state_reader.shmem.0.as_ptr() as *const SharedState;

    unsafe {
        let state = &*shared_ptr;
        let mut region = region.clone();
        region.normalize();

        let mut text = String::new();

        match mode {
            SelectionMode::Character => {
                // Extract character-wise selection
                for y in region.start_y..=region.end_y {
                    let start_x = if y == region.start_y {
                        region.start_x
                    } else {
                        0
                    };

                    let end_x = if y == region.end_y {
                        region.end_x
                    } else {
                        (GRID_WIDTH - 1) as u16
                    };

                    for x in start_x..=end_x {
                        if let Some(cell) =
                            crate::integration::get_cell_at(state, x as usize, y as usize)
                        {
                            if cell.char_codepoint == 0 || cell.char_codepoint == 32 {
                                text.push(' ');
                            } else if let Some(ch) = char::from_u32(cell.char_codepoint) {
                                text.push(ch);
                            }
                        }
                    }

                    // Add newline except for last line
                    if y < region.end_y {
                        text.push('\n');
                    }
                }
            }

            SelectionMode::Line => {
                // Extract full lines
                for y in region.start_y..=region.end_y {
                    for x in 0..GRID_WIDTH {
                        if let Some(cell) = crate::integration::get_cell_at(state, x, y as usize) {
                            if cell.char_codepoint == 0 || cell.char_codepoint == 32 {
                                text.push(' ');
                            } else if let Some(ch) = char::from_u32(cell.char_codepoint) {
                                text.push(ch);
                            }
                        }
                    }

                    // Add newline except for last line
                    if y < region.end_y {
                        text.push('\n');
                    }
                }
            }

            SelectionMode::Block => {
                // Extract rectangular block
                for y in region.start_y..=region.end_y {
                    for x in region.start_x..=region.end_x {
                        if let Some(cell) =
                            crate::integration::get_cell_at(state, x as usize, y as usize)
                        {
                            if cell.char_codepoint == 0 || cell.char_codepoint == 32 {
                                text.push(' ');
                            } else if let Some(ch) = char::from_u32(cell.char_codepoint) {
                                text.push(ch);
                            }
                        }
                    }

                    // Add newline except for last line
                    if y < region.end_y {
                        text.push('\n');
                    }
                }
            }
        }

        // Trim trailing whitespace from each line
        text.lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Handle keyboard input for visual selection
fn handle_selection_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<SelectionState>,
    mut event_writer: EventWriter<SelectionChangedEvent>,
    state_reader: Res<SharedMemoryReader>,
) {
    // Get current cursor position from terminal state
    let shared_ptr = state_reader.shmem.0.as_ptr() as *const SharedState;
    let (cursor_x, cursor_y) = unsafe {
        let s = &*shared_ptr;
        (s.cursor_x, s.cursor_y)
    };

    // Enter visual mode with 'v'
    if keyboard.just_pressed(KeyCode::KeyV) && !state.active {
        state.start_selection(cursor_x, cursor_y, SelectionMode::Character);
        info!("Visual selection mode activated");
    }

    // Enter visual line mode with 'V'
    if keyboard.just_pressed(KeyCode::KeyV) && keyboard.pressed(KeyCode::ShiftLeft) && !state.active
    {
        state.start_selection(cursor_x, cursor_y, SelectionMode::Line);
        info!("Visual line selection mode activated");
    }

    // Enter visual block mode with Ctrl+V
    if keyboard.just_pressed(KeyCode::KeyV)
        && keyboard.pressed(KeyCode::ControlLeft)
        && !state.active
    {
        state.start_selection(cursor_x, cursor_y, SelectionMode::Block);
        info!("Visual block selection mode activated");
    }

    if !state.active {
        return;
    }

    // Exit visual mode with Escape
    if keyboard.just_pressed(KeyCode::Escape) {
        state.end_selection();
        info!("Visual selection mode deactivated");
        return;
    }

    // Move cursor with arrow keys
    let mut cursor_moved = false;
    let mut new_x = state.cursor_x;
    let mut new_y = state.cursor_y;

    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        new_x = new_x.saturating_sub(1);
        cursor_moved = true;
    }

    if keyboard.just_pressed(KeyCode::ArrowRight) {
        new_x = (new_x + 1).min((GRID_WIDTH - 1) as u16);
        cursor_moved = true;
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        new_y = new_y.saturating_sub(1);
        cursor_moved = true;
    }

    if keyboard.just_pressed(KeyCode::ArrowDown) {
        new_y = (new_y + 1).min((GRID_HEIGHT - 1) as u16);
        cursor_moved = true;
    }

    if cursor_moved {
        state.update_selection(new_x, new_y);
        event_writer.send(SelectionChangedEvent {
            region: state.region.clone(),
        });
    }
}

/// Render selection overlay
fn render_selection_system(
    mut commands: Commands,
    state: Res<SelectionState>,
    existing_overlays: Query<Entity, With<SelectionOverlay>>,
    renderer: Res<TextRenderer>,
) {
    // Remove existing overlays
    for entity in existing_overlays.iter() {
        commands.entity(entity).despawn();
    }

    if !state.active || state.region.is_empty() {
        return;
    }

    // Get actual cell dimensions from TextRenderer
    let cell_width = renderer.cell_width;
    let cell_height = renderer.cell_height;

    let mut region = state.region.clone();
    region.normalize();

    match state.mode {
        SelectionMode::Character => {
            // Render character-wise selection
            for y in region.start_y..=region.end_y {
                let start_x = if y == region.start_y {
                    region.start_x
                } else {
                    0
                };

                let end_x = if y == region.end_y {
                    region.end_x
                } else {
                    (GRID_WIDTH - 1) as u16
                };

                // Calculate pixel position using grid_to_pixel
                let position = grid_to_pixel(start_x, y, cell_width, cell_height);
                let width = (end_x - start_x + 1) as f32 * cell_width;

                commands.spawn((
                    SelectionOverlay,
                    Sprite {
                        color: Color::srgba(0.3, 0.5, 1.0, 0.3),
                        custom_size: Some(Vec2::new(width, cell_height)),
                        ..default()
                    },
                    Transform::from_xyz(
                        position.x + width / 2.0, // Sprite anchor is center
                        position.y - cell_height / 2.0,
                        10.0,
                    ),
                ));
            }
        }

        SelectionMode::Line => {
            // Render line-wise selection (full lines)
            for y in region.start_y..=region.end_y {
                // Calculate pixel position for start of line
                let position = grid_to_pixel(0, y, cell_width, cell_height);
                let width = GRID_WIDTH as f32 * cell_width;

                commands.spawn((
                    SelectionOverlay,
                    Sprite {
                        color: Color::srgba(0.3, 0.5, 1.0, 0.3),
                        custom_size: Some(Vec2::new(width, cell_height)),
                        ..default()
                    },
                    Transform::from_xyz(
                        position.x + width / 2.0, // Sprite anchor is center
                        position.y - cell_height / 2.0,
                        10.0,
                    ),
                ));
            }
        }

        SelectionMode::Block => {
            // Render block-wise selection (rectangular)
            let position = grid_to_pixel(region.start_x, region.start_y, cell_width, cell_height);
            let width = (region.end_x - region.start_x + 1) as f32 * cell_width;
            let height = (region.end_y - region.start_y + 1) as f32 * cell_height;

            commands.spawn((
                SelectionOverlay,
                Sprite {
                    color: Color::srgba(0.3, 0.5, 1.0, 0.3),
                    custom_size: Some(Vec2::new(width, height)),
                    ..default()
                },
                Transform::from_xyz(
                    position.x + width / 2.0, // Sprite anchor is center
                    position.y - height / 2.0,
                    10.0,
                ),
            ));
        }
    }
}

/// Copy selected text to clipboard
fn copy_selection_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<SelectionState>,
    state_reader: Res<SharedMemoryReader>,
    mut event_writer: EventWriter<SelectionCopiedEvent>,
) {
    if !state.active {
        return;
    }

    // Copy with 'y' (yank in vim terminology)
    if keyboard.just_pressed(KeyCode::KeyY) {
        // Extract actual text from SharedState based on selection region
        let text = extract_selection_text(&state_reader, &state.region, state.mode);

        info!("Yanking {} characters to clipboard", text.len());

        // Copy to system clipboard
        match Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(&text) {
                    error!("Failed to copy to clipboard: {}", e);
                } else {
                    info!("Copied selection to clipboard");
                    event_writer.send(SelectionCopiedEvent { text });
                }
            }
            Err(e) => {
                error!("Failed to initialize clipboard: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_region_contains() {
        let region = SelectionRegion::new(5, 5, 10, 10);

        assert!(region.contains(5, 5));
        assert!(region.contains(10, 10));
        assert!(region.contains(7, 7));
        assert!(!region.contains(4, 5));
        assert!(!region.contains(11, 10));
    }

    #[test]
    fn test_selection_region_normalize() {
        let mut region = SelectionRegion::new(10, 10, 5, 5);
        region.normalize();

        assert_eq!(region.start_x, 5);
        assert_eq!(region.start_y, 5);
        assert_eq!(region.end_x, 10);
        assert_eq!(region.end_y, 10);
    }

    #[test]
    fn test_selection_state() {
        let mut state = SelectionState::default();

        assert!(!state.active);

        state.start_selection(5, 5, SelectionMode::Character);
        assert!(state.active);
        assert_eq!(state.region.start_x, 5);
        assert_eq!(state.region.start_y, 5);

        state.update_selection(10, 10);
        assert_eq!(state.region.end_x, 10);
        assert_eq!(state.region.end_y, 10);

        state.end_selection();
        assert!(!state.active);
    }
}
