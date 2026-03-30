// Search overlay UI for scrollback buffer
// Provides Ctrl+F search interface with match highlighting and navigation

use bevy::prelude::*;
use crate::terminal::scrollback::{ScrollbackBuffer, ScrollbackState};

/// Marker component for search overlay UI
#[derive(Component)]
pub struct SearchOverlay;

/// Marker component for search input box
#[derive(Component)]
pub struct SearchInputBox;

/// Marker component for search results text
#[derive(Component)]
pub struct SearchResultsText;

/// Search overlay configuration
#[derive(Resource, Clone)]
pub struct SearchOverlayConfig {
    /// Height of search overlay in pixels
    pub height: f32,
    /// Background color
    pub bg_color: Color,
    /// Text color
    pub text_color: Color,
    /// Border color
    pub border_color: Color,
    /// Font size
    pub font_size: f32,
}

impl Default for SearchOverlayConfig {
    fn default() -> Self {
        Self {
            height: 60.0,
            bg_color: Color::srgba(0.1, 0.1, 0.1, 0.95),
            text_color: Color::srgb(0.9, 0.9, 0.9),
            border_color: Color::srgb(0.3, 0.5, 0.8),
            font_size: 16.0,
        }
    }
}

/// System to spawn search overlay when activated
fn spawn_search_overlay(
    mut commands: Commands,
    state: Res<ScrollbackState>,
    config: Res<SearchOverlayConfig>,
    overlay_query: Query<Entity, With<SearchOverlay>>,
    window_query: Query<&Window>,
) {
    // Get window dimensions
    let window = window_query.single();
    let window_height = window.height();

    if state.search_visible && overlay_query.is_empty() {
        // Spawn search overlay container
        commands
            .spawn((
                SearchOverlay,
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(config.height),
                    border: UiRect::top(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(config.bg_color),
                BorderColor(config.border_color),
            ))
            .with_children(|parent| {
                // Search prompt label
                parent.spawn((
                    Text::new("Find: "),
                    TextFont {
                        font_size: config.font_size,
                        ..default()
                    },
                    TextColor(config.text_color),
                    Node {
                        margin: UiRect::right(Val::Px(10.0)),
                        ..default()
                    },
                ));

                // Search input box (visual representation)
                parent.spawn((
                    SearchInputBox,
                    Text::new(""),
                    TextFont {
                        font_size: config.font_size,
                        ..default()
                    },
                    TextColor(config.text_color),
                    Node {
                        flex_grow: 1.0,
                        margin: UiRect::right(Val::Px(20.0)),
                        ..default()
                    },
                ));

                // Results counter
                parent.spawn((
                    SearchResultsText,
                    Text::new(""),
                    TextFont {
                        font_size: config.font_size,
                        ..default()
                    },
                    TextColor(config.text_color),
                ));
            });

        info!("Search overlay spawned");
    }
}

/// System to despawn search overlay when deactivated
fn despawn_search_overlay(
    mut commands: Commands,
    state: Res<ScrollbackState>,
    overlay_query: Query<Entity, With<SearchOverlay>>,
) {
    if !state.search_visible && !overlay_query.is_empty() {
        for entity in overlay_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        info!("Search overlay despawned");
    }
}

/// System to update search overlay content
fn update_search_overlay(
    state: Res<ScrollbackState>,
    scrollback: Res<ScrollbackBuffer>,
    mut input_query: Query<&mut Text, (With<SearchInputBox>, Without<SearchResultsText>)>,
    mut results_query: Query<&mut Text, (With<SearchResultsText>, Without<SearchInputBox>)>,
) {
    if !state.search_visible {
        return;
    }

    // Update input text
    for mut text in input_query.iter_mut() {
        **text = format!("{}_", state.search_input); // Add cursor
    }

    // Update results text
    if let Some(search_state) = scrollback.search_state() {
        for mut text in results_query.iter_mut() {
            if search_state.total_results > 0 {
                **text = format!(
                    "{} of {} matches{}{}",
                    search_state.current_index + 1,
                    search_state.total_results,
                    if search_state.case_sensitive { " [Aa]" } else { "" },
                    if search_state.use_regex { " [.*]" } else { "" }
                );
            } else {
                **text = format!(
                    "No matches{}{}",
                    if search_state.case_sensitive { " [Aa]" } else { "" },
                    if search_state.use_regex { " [.*]" } else { "" }
                );
            }
        }
    } else {
        for mut text in results_query.iter_mut() {
            **text = "Type to search".to_string();
        }
    }
}

/// System to handle character input in search box
fn handle_search_input(
    mut char_events: EventReader<bevy::input::keyboard::KeyboardInput>,
    mut state: ResMut<ScrollbackState>,
    mut scrollback: ResMut<ScrollbackBuffer>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if !state.search_visible {
        return;
    }

    for event in char_events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        match &event.logical_key {
            bevy::input::keyboard::Key::Character(ref s) => {
                // Don't add control characters
                if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
                    continue;
                }

                // Add character to search input
                state.search_input.push_str(s);

                // Trigger search
                if !state.search_input.is_empty() {
                    scrollback.search(
                        state.search_input.clone(),
                        false, // case_sensitive (TODO: make configurable)
                        false, // use_regex (TODO: make configurable)
                    );
                }
            }
            bevy::input::keyboard::Key::Backspace => {
                // Remove last character
                state.search_input.pop();

                // Re-trigger search or clear if empty
                if !state.search_input.is_empty() {
                    scrollback.search(state.search_input.clone(), false, false);
                } else {
                    scrollback.clear_search();
                }
            }
            _ => {}
        }
    }
}

/// Plugin for search overlay functionality
pub struct SearchOverlayPlugin;

impl Plugin for SearchOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SearchOverlayConfig::default())
            .add_systems(
                Update,
                (
                    spawn_search_overlay,
                    despawn_search_overlay,
                    update_search_overlay,
                    handle_search_input,
                )
                    .chain(),
            );

        info!("Search overlay plugin initialized");
    }
}
