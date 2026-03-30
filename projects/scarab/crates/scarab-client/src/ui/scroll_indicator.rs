// Scroll position indicator UI
// Shows when user has scrolled away from live view and how far

use bevy::prelude::*;
use crate::terminal::scrollback::{ScrollbackBuffer, ScrollbackState};

/// Marker component for scroll indicator
#[derive(Component)]
pub struct ScrollIndicator;

/// Scroll indicator configuration
#[derive(Resource, Clone)]
pub struct ScrollIndicatorConfig {
    /// Position from top-right corner
    pub offset_x: f32,
    pub offset_y: f32,
    /// Background color
    pub bg_color: Color,
    /// Text color
    pub text_color: Color,
    /// Font size
    pub font_size: f32,
    /// Padding
    pub padding: f32,
    /// Border radius
    pub border_radius: f32,
}

impl Default for ScrollIndicatorConfig {
    fn default() -> Self {
        Self {
            offset_x: 20.0,
            offset_y: 20.0,
            bg_color: Color::srgba(0.2, 0.3, 0.4, 0.9),
            text_color: Color::srgb(1.0, 1.0, 1.0),
            font_size: 14.0,
            padding: 8.0,
            border_radius: 4.0,
        }
    }
}

/// System to spawn scroll indicator when scrolled
fn spawn_scroll_indicator(
    mut commands: Commands,
    state: Res<ScrollbackState>,
    config: Res<ScrollIndicatorConfig>,
    indicator_query: Query<Entity, With<ScrollIndicator>>,
) {
    if state.is_scrolled && indicator_query.is_empty() {
        commands.spawn((
            ScrollIndicator,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(config.offset_y),
                right: Val::Px(config.offset_x),
                padding: UiRect::all(Val::Px(config.padding)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(config.bg_color),
            BorderColor(Color::srgba(0.4, 0.5, 0.6, 0.8)),
            BorderRadius::all(Val::Px(config.border_radius)),
            Text::new(""),
            TextFont {
                font_size: config.font_size,
                ..default()
            },
            TextColor(config.text_color),
        ));

        debug!("Scroll indicator spawned");
    }
}

/// System to despawn scroll indicator when at bottom
fn despawn_scroll_indicator(
    mut commands: Commands,
    state: Res<ScrollbackState>,
    indicator_query: Query<Entity, With<ScrollIndicator>>,
) {
    if !state.is_scrolled && !indicator_query.is_empty() {
        for entity in indicator_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        debug!("Scroll indicator despawned");
    }
}

/// System to update scroll indicator content
fn update_scroll_indicator(
    scrollback: Res<ScrollbackBuffer>,
    mut indicator_query: Query<&mut Text, With<ScrollIndicator>>,
) {
    for mut text in indicator_query.iter_mut() {
        let offset = scrollback.scroll_offset();
        let total = scrollback.line_count();

        // Format indicator text
        **text = if offset > 0 {
            format!("↑ {} lines (of {})", offset, total)
        } else {
            "Live".to_string()
        };
    }
}

/// Plugin for scroll position indicator
pub struct ScrollIndicatorPlugin;

impl Plugin for ScrollIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScrollIndicatorConfig::default())
            .add_systems(
                Update,
                (
                    spawn_scroll_indicator,
                    despawn_scroll_indicator,
                    update_scroll_indicator,
                )
                    .chain(),
            );

        info!("Scroll indicator plugin initialized");
    }
}
