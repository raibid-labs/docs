//! Custom BRP tools for entity spawning with meshes and materials
//!
//! Standard BRP cannot spawn entities with meshes/materials because asset handles
//! contain Arc<StrongHandle> which aren't serializable. These custom methods
//! create assets internally and return the spawned entity ID.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "brp")]
use bevy::remote::{http::RemoteHttpPlugin, BrpResult, RemotePlugin};

#[cfg(feature = "brp")]
use serde_json::{json, Value};

/// Plugin that registers custom BRP methods for entity spawning
pub struct CustomBrpPlugin;

impl Plugin for CustomBrpPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "brp")]
        {
            app.add_plugins(
                RemotePlugin::default()
                    .with_method("bevy/spawn_cube", Self::spawn_cube)
                    .with_method("bevy/spawn_sphere", Self::spawn_sphere),
            )
            .add_plugins(RemoteHttpPlugin::default());

            info!("🔧 Custom BRP methods registered:");
            info!("   - bevy/spawn_cube");
            info!("   - bevy/spawn_sphere");
        }
    }
}

/// Parameters for spawning a cube via BRP
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpawnCubeParams {
    /// Position of the cube [x, y, z]
    #[serde(default)]
    pub position: [f32; 3],

    /// Scale of the cube [x, y, z]
    #[serde(default = "default_scale")]
    pub scale: [f32; 3],

    /// Base color [r, g, b] in range 0.0-1.0
    #[serde(default = "default_color")]
    pub color: [f32; 3],

    /// Metallic value (0.0-1.0)
    #[serde(default = "default_metallic")]
    pub metallic: f32,

    /// Perceptual roughness (0.0-1.0)
    #[serde(default = "default_roughness")]
    pub roughness: f32,

    /// Name for the entity
    #[serde(default = "default_cube_name")]
    pub name: String,
}

/// Parameters for spawning a sphere via BRP
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpawnSphereParams {
    /// Position of the sphere [x, y, z]
    #[serde(default)]
    pub position: [f32; 3],

    /// Radius of the sphere
    #[serde(default = "default_sphere_radius")]
    pub radius: f32,

    /// Base color [r, g, b] in range 0.0-1.0
    #[serde(default = "default_color")]
    pub color: [f32; 3],

    /// Metallic value (0.0-1.0)
    #[serde(default = "default_metallic")]
    pub metallic: f32,

    /// Perceptual roughness (0.0-1.0)
    #[serde(default = "default_roughness")]
    pub roughness: f32,

    /// Name for the entity
    #[serde(default = "default_sphere_name")]
    pub name: String,
}

/// Response from entity spawning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnResponse {
    /// The spawned entity ID
    pub entity: u32,

    /// The name assigned to the entity
    pub name: String,
}

// Default value functions for serde
fn default_scale() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

fn default_color() -> [f32; 3] {
    [0.8, 0.7, 0.6]
}

fn default_metallic() -> f32 {
    0.5
}

fn default_roughness() -> f32 {
    0.5
}

fn default_cube_name() -> String {
    "AI Spawned Cube".to_string()
}

fn default_sphere_name() -> String {
    "AI Spawned Sphere".to_string()
}

fn default_sphere_radius() -> f32 {
    0.5
}

#[cfg(feature = "brp")]
impl CustomBrpPlugin {
    /// Custom BRP method: spawn_cube
    ///
    /// Spawns a cube entity with mesh and material.
    ///
    /// Parameters:
    /// - position: [x, y, z] (default: [0, 0, 0])
    /// - scale: [x, y, z] (default: [1, 1, 1])
    /// - color: [r, g, b] (default: [0.8, 0.7, 0.6])
    /// - metallic: 0.0-1.0 (default: 0.5)
    /// - roughness: 0.0-1.0 (default: 0.5)
    /// - name: string (default: "AI Spawned Cube")
    ///
    /// Returns:
    /// - entity: u32 (entity ID)
    /// - name: string (assigned name)
    fn spawn_cube(
        In(params): In<Option<Value>>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) -> BrpResult {
        let params: SpawnCubeParams = params
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        let entity = commands
            .spawn((
                Mesh3d(meshes.add(Cuboid::default())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(params.color[0], params.color[1], params.color[2]),
                    metallic: params.metallic,
                    perceptual_roughness: params.roughness,
                    ..default()
                })),
                Transform {
                    translation: Vec3::from(params.position),
                    scale: Vec3::from(params.scale),
                    ..default()
                },
                Name::new(params.name.clone()),
            ))
            .id();

        info!("✅ Spawned cube '{}' at entity {:?}", params.name, entity);

        Ok(json!({
            "entity": entity.index(),
            "name": params.name,
        }))
    }

    /// Custom BRP method: spawn_sphere
    ///
    /// Spawns a sphere entity with mesh and material.
    ///
    /// Parameters:
    /// - position: [x, y, z] (default: [0, 0, 0])
    /// - radius: float (default: 0.5)
    /// - color: [r, g, b] (default: [0.8, 0.7, 0.6])
    /// - metallic: 0.0-1.0 (default: 0.5)
    /// - roughness: 0.0-1.0 (default: 0.5)
    /// - name: string (default: "AI Spawned Sphere")
    ///
    /// Returns:
    /// - entity: u32 (entity ID)
    /// - name: string (assigned name)
    fn spawn_sphere(
        In(params): In<Option<Value>>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) -> BrpResult {
        let params: SpawnSphereParams = params
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        let entity = commands
            .spawn((
                Mesh3d(meshes.add(Sphere::new(params.radius))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(params.color[0], params.color[1], params.color[2]),
                    metallic: params.metallic,
                    perceptual_roughness: params.roughness,
                    ..default()
                })),
                Transform::from_translation(Vec3::from(params.position)),
                Name::new(params.name.clone()),
            ))
            .id();

        info!("✅ Spawned sphere '{}' at entity {:?}", params.name, entity);

        Ok(json!({
            "entity": entity.index(),
            "name": params.name,
        }))
    }
}
