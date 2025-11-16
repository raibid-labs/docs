//! Main binary entry point for Bevy MCP Ratatui reference implementation
//!
//! Default behavior: Dual-mode rendering (window + terminal output)
//! The window provides 3D rendering context, terminal displays ASCII/Unicode conversion.

use bevy::prelude::*;

#[cfg(feature = "tui")]
use bevy_mcp_ratatui_ref::tui::BevyMcpTuiPlugin;

#[cfg(feature = "brp")]
use bevy_brp_extras::BrpExtrasPlugin;

fn main() {
    let mut app = App::new();

    // Use DefaultPlugins for full 3D rendering infrastructure (dual mode: window + terminal)
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Bevy MCP Ratatui - AI-Controlled Terminal Rendering".to_string(),
            resolution: (1024., 768.).into(),
            ..default()
        }),
        ..default()
    }));

    #[cfg(feature = "tui")]
    info!("🖥️  Running in dual mode (window + terminal rendering)");

    #[cfg(not(feature = "tui"))]
    info!("🪟 Running in windowed mode (use --features tui for terminal rendering)");

    // Add BRP (Bevy Remote Protocol) for MCP integration
    #[cfg(feature = "brp")]
    {
        app.add_plugins(BrpExtrasPlugin);
        info!("🎮 BRP enabled on port 15702");
        info!("🤖 MCP tools can now interact with this game");
    }

    // Add TUI rendering plugin
    #[cfg(feature = "tui")]
    {
        app.add_plugins(BevyMcpTuiPlugin::default());
        info!("🖥️  TUI rendering enabled");
        info!("📺 Terminal will show 3D scene rendering");
    }

    // Add demo scene setup
    app.add_systems(Startup, setup_demo_scene);

    info!("🚀 Starting Bevy MCP Ratatui application");
    app.run();
}

/// Setup a basic demo scene
fn setup_demo_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Name::new("Demo Cube"),
    ));

    // Spawn a ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 10.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("Ground Plane"),
    ));

    // Spawn a light
    commands.spawn((
        PointLight {
            intensity: 2000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
        Name::new("Main Light"),
    ));

    // Spawn a camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Main Camera"),
    ));

    info!("✅ Demo scene setup complete");
}
