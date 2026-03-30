use crate::ipc::RemoteMessageEvent;
use bevy::prelude::*;
use scarab_protocol::{DaemonMessage, LogLevel, NotifyLevel, OverlayStyle};

/// Component to tag entities as remote overlays
#[derive(Component)]
pub struct RemoteOverlay {
    pub id: u64,
}

/// Component for notification UI elements
#[derive(Component)]
struct NotificationUI {
    /// Time when notification was created
    created_at: f64,
    /// Duration before auto-dismiss (seconds)
    lifetime: f32,
}

/// Component for plugin log display
#[derive(Component)]
struct PluginLogDisplay;

/// Event to hide modal (sent when ESC is pressed or modal is dismissed)
#[derive(Event)]
pub struct HideModalEvent;

pub struct RemoteUiPlugin;

impl Plugin for RemoteUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HideModalEvent>()
            .add_systems(Update, (
                handle_remote_messages,
                update_notifications,
                handle_hide_modal,
            ));
    }
}

fn handle_remote_messages(
    mut commands: Commands,
    mut events: EventReader<RemoteMessageEvent>,
    mut show_modal_events: EventWriter<crate::ui::command_palette::ShowRemoteModalEvent>,
    mut hide_modal_events: EventWriter<HideModalEvent>,
    overlay_query: Query<(Entity, &RemoteOverlay)>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    for event in events.read() {
        match &event.0 {
            DaemonMessage::DrawOverlay {
                id,
                x,
                y,
                text,
                style,
            } => {
                // Calculate pixel position (Assuming 8x16 font for now - should come from config/resources)
                let char_width = 8.0;
                let char_height = 16.0;

                // Hack: Just spawning text for now. Position will be wrong until we get window size.
                let pixel_x = *x as f32 * char_width;
                let pixel_y = *y as f32 * -char_height; // Y is down in terminal, Up in Bevy

                commands.spawn((
                    Text2d::default(),
                    Text::new(text.as_str()),
                    TextFont {
                        font: asset_server.load("fonts/FiraCode-Regular.ttf"),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgba_u8(
                        (style.fg >> 24) as u8,
                        (style.fg >> 16) as u8,
                        (style.fg >> 8) as u8,
                        (style.fg) as u8,
                    )),
                    Transform::from_xyz(pixel_x, pixel_y, style.z_index),
                    RemoteOverlay { id: *id },
                ));
            }
            DaemonMessage::ClearOverlays { id } => {
                for (entity, overlay) in overlay_query.iter() {
                    if let Some(target_id) = id {
                        if overlay.id == *target_id {
                            commands.entity(entity).despawn_recursive();
                        }
                    } else {
                        // Clear all
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
            DaemonMessage::ShowModal { title, items } => {
                show_modal_events.send(crate::ui::command_palette::ShowRemoteModalEvent {
                    title: title.clone(),
                    items: items.clone(),
                });
            }
            DaemonMessage::HideModal => {
                hide_modal_events.send(HideModalEvent);
            }
            DaemonMessage::PluginLog {
                plugin_name,
                level,
                message,
            } => {
                // Log to console
                match level {
                    LogLevel::Error => error!("[{}] {}", plugin_name, message),
                    LogLevel::Warn => warn!("[{}] {}", plugin_name, message),
                    LogLevel::Info => info!("[{}] {}", plugin_name, message),
                    LogLevel::Debug => debug!("[{}] {}", plugin_name, message),
                }

                // TODO: Could also display in an on-screen log panel
            }
            DaemonMessage::PluginNotification { title, body, level } => {
                spawn_notification(&mut commands, &asset_server, title, body, *level, time.elapsed_secs_f64());
            }
            _ => {} // Ignore others
        }
    }
}

/// Spawn a notification UI element
fn spawn_notification(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    title: &str,
    body: &str,
    level: NotifyLevel,
    current_time: f64,
) {
    // Determine colors based on notification level
    let (bg_color, icon) = match level {
        NotifyLevel::Error => (Color::srgba(0.8, 0.2, 0.2, 0.95), "ERROR"),
        NotifyLevel::Warning => (Color::srgba(0.9, 0.6, 0.2, 0.95), "WARN"),
        NotifyLevel::Info => (Color::srgba(0.2, 0.5, 0.8, 0.95), "INFO"),
        NotifyLevel::Success => (Color::srgba(0.2, 0.7, 0.3, 0.95), "OK"),
    };

    commands
        .spawn((
            Node {
                width: Val::Px(350.0),
                height: Val::Auto,
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                top: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(15.0)),
                row_gap: Val::Px(5.0),
                ..default()
            },
            BackgroundColor(bg_color),
            NotificationUI {
                created_at: current_time,
                lifetime: 5.0, // Auto-dismiss after 5 seconds
            },
        ))
        .with_children(|parent| {
            // Notification header (icon + title)
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|header| {
                    // Icon
                    header.spawn((
                        Text::new(icon),
                        TextFont {
                            font: asset_server.load("fonts/FiraCode-Regular.ttf"),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Title
                    header.spawn((
                        Text::new(title),
                        TextFont {
                            font: asset_server.load("fonts/FiraCode-Regular.ttf"),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            flex_grow: 1.0,
                            ..default()
                        },
                    ));
                });

            // Notification body
            parent.spawn((
                Text::new(body),
                TextFont {
                    font: asset_server.load("fonts/FiraCode-Regular.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
                Node {
                    max_width: Val::Px(320.0),
                    ..default()
                },
            ));
        });
}

/// Update notifications - handle auto-dismiss and stacking
fn update_notifications(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &NotificationUI, &mut Node), With<NotificationUI>>,
) {
    let current_time = time.elapsed_secs_f64();

    // Collect notification data to avoid borrow checker issues
    let mut notifications_data: Vec<(Entity, f64, f32)> = query
        .iter()
        .map(|(e, n, _)| (e, n.created_at, n.lifetime))
        .collect();

    // Sort by creation time (newest first)
    notifications_data.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Process notifications
    for (index, (entity, created_at, lifetime)) in notifications_data.iter().enumerate() {
        // Check if notification should be dismissed
        let age = current_time - created_at;
        if age >= *lifetime as f64 {
            commands.entity(*entity).despawn_recursive();
            continue;
        }

        // Update position for stacking
        if let Ok((_, _, mut node)) = query.get_mut(*entity) {
            node.top = Val::Px(20.0 + (index as f32 * 90.0)); // Stack with 90px spacing
        }
    }
}

/// Handle modal hiding events
fn handle_hide_modal(
    mut events: EventReader<HideModalEvent>,
    mut palette_state: ResMut<crate::ui::command_palette::CommandPaletteState>,
) {
    for _event in events.read() {
        palette_state.active = false;
    }
}
