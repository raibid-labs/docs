use bevy::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Bevy MCP Reference - AI-Assisted Game Development".to_string(),
            resolution: (800., 600.).into(),
            ..default()
        }),
        ..default()
    }));

    // Add Bevy Remote Protocol for MCP integration
    #[cfg(feature = "brp")]
    {
        use bevy_brp_extras::BrpExtrasPlugin;
        // BrpExtrasPlugin includes RemotePlugin and RemoteHttpPlugin internally
        app.add_plugins(BrpExtrasPlugin);
        info!("Bevy Remote Protocol (BRP) enabled on port 15702");
        info!("BRP Extras enabled - full mutation support!");
        info!("MCP tools can now interact with this game instance");
    }

    app.add_systems(Startup, setup)
        .add_systems(Update, rotate_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Name::new("MCP Demo Cube"),
    ));

    // Spawn a light
    commands.spawn((
        PointLight {
            intensity: 1500.0,
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
}

fn rotate_camera(time: Res<Time>, mut query: Query<&mut Transform, With<Camera3d>>) {
    for mut transform in &mut query {
        transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(time.delta_secs() * 0.2));
    }
}
