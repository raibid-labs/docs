//! Demo scene systems
//!
//! Systems for running demo animations and interactions.

use bevy::prelude::*;

/// Marker component for rotating entities
#[derive(Component)]
pub struct Rotating {
    pub speed: f32,
}

impl Default for Rotating {
    fn default() -> Self {
        Self { speed: 1.0 }
    }
}

/// System that rotates entities with the Rotating component
pub fn rotate_entities(time: Res<Time>, mut query: Query<(&mut Transform, &Rotating)>) {
    for (mut transform, rotating) in &mut query {
        transform.rotate_y(time.delta_secs() * rotating.speed);
    }
}

/// Plugin for demo systems
pub struct DemoSystemsPlugin;

impl Plugin for DemoSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, rotate_entities);
    }
}
