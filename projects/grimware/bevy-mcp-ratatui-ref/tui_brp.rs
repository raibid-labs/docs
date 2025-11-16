//! TUI + BRP integration example
//!
//! Demonstrates 3D rendering to terminal with AI control via BRP.
//! Creates a window (for 3D rendering) AND displays ASCII in terminal.
//!
//! Run with: cargo run --example tui_brp --features full
//! BRP listens on: localhost:15702
//! Exit with: Ctrl+C or close window

use bevy::prelude::*;
use bevy_mcp_ratatui_ref::prelude::*;

#[cfg(feature = "brp")]
use bevy_brp_extras::BrpExtrasPlugin;

fn main() {
    let mut app = App::new();

    // Note: bevy_ratatui_camera requires full 3D rendering infrastructure
    // DefaultPlugins provides the rendering pipeline that gets converted to ASCII
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "TUI + BRP - AI-Controlled 3D Terminal Rendering".to_string(),
            resolution: (1024., 768.).into(),
            ..default()
        }),
        ..default()
    }));

    // Add BRP for AI control
    #[cfg(feature = "brp")]
    {
        app.add_plugins(CustomBrpPlugin);  // Custom BRP methods for entity spawning
        app.add_plugins(BrpExtrasPlugin);  // Extra features (screenshot, shutdown)
        info!("🤖 BRP enabled on port 15702");
        info!("💡 AI can now control this scene via MCP tools");
        info!("🔧 Custom spawn methods available: bevy/spawn_cube, bevy/spawn_sphere");
    }

    // Add TUI rendering
    app.add_plugins(BevyMcpTuiPlugin::default());

    // Add demo systems
    app.add_plugins(DemoSystemsPlugin);

    // Setup scene
    app.add_systems(Startup, setup);

    info!("🚀 Starting TUI + BRP example");
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Setting up AI-controllable TUI scene...");

    // Spawn multiple colored spheres
    let sphere_configs = [
        (Vec3::new(-2.0, 0.5, 0.0), Color::srgb(0.8, 0.2, 0.2), "Red Sphere"),
        (Vec3::new(0.0, 0.5, 0.0), Color::srgb(0.2, 0.8, 0.2), "Green Sphere"),
        (Vec3::new(2.0, 0.5, 0.0), Color::srgb(0.2, 0.2, 0.8), "Blue Sphere"),
    ];

    for (pos, color, name) in sphere_configs.iter() {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.add(*color)),
            Transform::from_translation(*pos),
            Rotating { speed: 0.5 },
            Name::new(*name),
        ));
        info!("Spawned: {}", name);
    }

    // Spawn ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(15.0, 15.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("Ground Plane"),
    ));

    // Spawn lights
    commands.spawn((
        PointLight {
            intensity: 3000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
        Name::new("Main Light"),
    ));

    // Spawn camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
        Name::new("Main Camera"),
    ));

    info!("✅ AI-controllable scene setup complete");
    info!("📺 Check your terminal for ASCII rendering!");
    info!("🤖 BRP ready for MCP commands on localhost:15702");
    info!("🪟 Window provides 3D rendering, terminal shows ASCII conversion");
    info!("");
    info!("💡 Try AI prompts like:");
    info!("   - 'Show me all entities'");
    info!("   - 'Add a cube at position [3, 1, 0]'");
    info!("   - 'Spawn a red sphere at [-3, 1, 0]'");
    info!("   - 'Move the red sphere up by 2 units'");
    info!("   - 'Change the green sphere color to yellow'");
}
