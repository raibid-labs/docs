//! Windowed + TUI dual rendering example (Debugging Mode)
//!
//! Demonstrates simultaneous window and terminal rendering.
//! Useful for debugging - compare the 3D window output with ASCII terminal rendering.
//!
//! Run with: cargo run --example windowed_tui --features full
//! Exit with: Close window or Ctrl+C

use bevy::prelude::*;
use bevy_mcp_ratatui_ref::prelude::*;

#[cfg(feature = "brp")]
use bevy_brp_extras::BrpExtrasPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Windowed + TUI - Dual Rendering".to_string(),
            resolution: (1280., 720.).into(),
            ..default()
        }),
        ..default()
    }));

    // Add BRP
    #[cfg(feature = "brp")]
    {
        app.add_plugins(CustomBrpPlugin);  // Custom BRP methods for entity spawning
        app.add_plugins(BrpExtrasPlugin);  // Extra features (screenshot, shutdown)
    }

    // Add TUI rendering
    app.add_plugins(BevyMcpTuiPlugin::default());

    // Add demo systems
    app.add_plugins(DemoSystemsPlugin);

    // Setup scene
    app.add_systems(Startup, setup);

    info!("🚀 Starting dual rendering mode");
    info!("🪟 Window: High-fidelity 3D rendering");
    info!("📺 Terminal: ASCII/Unicode rendering");
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Setting up dual rendering scene...");

    // Create a more complex scene for demonstration

    // Central rotating cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.4, 0.2),
            metallic: 0.5,
            perceptual_roughness: 0.3,
            ..default()
        })),
        Transform::from_xyz(0.0, 1.0, 0.0),
        Rotating { speed: 1.0 },
        Name::new("Central Cube"),
    ));

    // Orbiting spheres
    for i in 0..4 {
        let angle = (i as f32 / 4.0) * std::f32::consts::PI * 2.0;
        let radius = 3.0;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.3))),
            MeshMaterial3d(materials.add(Color::hsl(i as f32 * 90.0, 0.8, 0.5))),
            Transform::from_xyz(x, 0.5, z),
            Rotating { speed: 0.3 },
            Name::new(format!("Orbiting Sphere {}", i + 1)),
        ));
    }

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.4, 0.2))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("Ground Plane"),
    ));

    // Lighting
    commands.spawn((
        PointLight {
            intensity: 4000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0),
        Name::new("Main Light"),
    ));

    commands.spawn((
        PointLight {
            intensity: 2000.0,
            color: Color::srgb(0.8, 0.8, 1.0),
            ..default()
        },
        Transform::from_xyz(-5.0, 8.0, -5.0),
        Name::new("Fill Light"),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 8.0, 12.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        Name::new("Main Camera"),
    ));

    info!("✅ Dual rendering scene complete");
    info!("💡 Same scene renders to both window and terminal!");
    info!("🔍 Use this mode to verify TUI rendering accuracy");
    info!("🎮 Close window or Ctrl+C to exit");
    info!("");
    info!("💡 Try AI prompts like:");
    info!("   - 'Add a cube at position [3, 1, 0]'");
    info!("   - 'Spawn a purple sphere at [-3, 1, 0]'");
}
