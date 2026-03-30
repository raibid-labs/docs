// Spacemacs-like leader key menu system
// Provides hierarchical command menus triggered by a leader key

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use std::time::{Duration, Instant};

/// Plugin for leader key functionality
pub struct LeaderKeyPlugin;

impl Plugin for LeaderKeyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LeaderKeyState>()
            .init_resource::<LeaderKeyMenus>()
            .add_event::<LeaderKeyActivatedEvent>()
            .add_systems(
                Update,
                (
                    handle_leader_key_system,
                    handle_menu_navigation_system,
                    render_menu_system,
                    timeout_check_system,
                )
                    .chain(),
            );
    }
}

/// State of leader key system
#[derive(Resource)]
pub struct LeaderKeyState {
    pub active: bool,
    pub last_press: Option<Instant>,
    pub timeout: Duration,
    pub current_menu: String,
    pub key_sequence: Vec<char>,
}

impl Default for LeaderKeyState {
    fn default() -> Self {
        Self {
            active: false,
            last_press: None,
            timeout: Duration::from_millis(1000),
            current_menu: "root".to_string(),
            key_sequence: Vec::new(),
        }
    }
}

/// Menu item in leader key system
#[derive(Clone)]
pub struct MenuItem {
    pub key: char,
    pub label: String,
    pub description: String,
    pub action: MenuAction,
}

/// Action that can be triggered by menu item
#[derive(Clone)]
pub enum MenuAction {
    Command(String),
    SubMenu(String),
}

/// Menu definition
#[derive(Clone, Default)]
pub struct Menu {
    pub title: String,
    pub items: Vec<MenuItem>,
}

/// Registry of all leader key menus
#[derive(Resource, Default)]
pub struct LeaderKeyMenus {
    menus: std::collections::HashMap<String, Menu>,
}

impl LeaderKeyMenus {
    pub fn register(&mut self, id: &str, menu: Menu) {
        self.menus.insert(id.to_string(), menu);
    }

    pub fn get(&self, id: &str) -> Option<&Menu> {
        self.menus.get(id)
    }
}

/// Event fired when leader key is activated
#[derive(Event)]
pub struct LeaderKeyActivatedEvent {
    pub command: String,
}

/// Component for menu UI
#[derive(Component)]
struct LeaderMenuUI;

/// Handle leader key activation
fn handle_leader_key_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<LeaderKeyState>,
) {
    // Activate with Space key (when not in insert mode)
    if keyboard.just_pressed(KeyCode::Space) {
        state.active = true;
        state.last_press = Some(Instant::now());
        state.current_menu = "root".to_string();
        state.key_sequence.clear();
        info!("Leader key activated");
    }

    // Deactivate with Escape
    if state.active && keyboard.just_pressed(KeyCode::Escape) {
        state.active = false;
        state.key_sequence.clear();
        info!("Leader key deactivated");
    }
}

/// Handle menu navigation and selection
fn handle_menu_navigation_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<LeaderKeyState>,
    menus: Res<LeaderKeyMenus>,
    mut event_writer: EventWriter<LeaderKeyActivatedEvent>,
) {
    if !state.active {
        return;
    }

    // Get current menu
    let current_menu = match menus.get(&state.current_menu) {
        Some(menu) => menu,
        None => return,
    };

    // Check for key presses
    for key_code in keyboard.get_just_pressed() {
        if let Some(char) = keycode_to_char(*key_code) {
            state.last_press = Some(Instant::now());

            // Find matching menu item
            if let Some(item) = current_menu.items.iter().find(|i| i.key == char) {
                state.key_sequence.push(char);

                match &item.action {
                    MenuAction::Command(cmd) => {
                        info!("Executing command: {}", cmd);
                        event_writer.send(LeaderKeyActivatedEvent {
                            command: cmd.clone(),
                        });
                        state.active = false;
                        state.key_sequence.clear();
                    }
                    MenuAction::SubMenu(menu_id) => {
                        info!("Entering submenu: {}", menu_id);
                        state.current_menu = menu_id.clone();
                    }
                }
            }
        }
    }
}

/// Render leader key menu
fn render_menu_system(
    mut commands: Commands,
    state: Res<LeaderKeyState>,
    menus: Res<LeaderKeyMenus>,
    existing_ui: Query<Entity, With<LeaderMenuUI>>,
) {
    // Remove existing UI
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }

    if !state.active {
        return;
    }

    // Get current menu
    let current_menu = match menus.get(&state.current_menu) {
        Some(menu) => menu,
        None => return,
    };

    // Create menu container
    commands
        .spawn((
            LeaderMenuUI,
            Node {
                width: Val::Px(400.0),
                position_type: PositionType::Absolute,
                left: Val::Px(50.0),
                top: Val::Px(50.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(15.0)),
                row_gap: Val::Px(5.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
        ))
        .with_children(|parent| {
            // Menu title
            parent.spawn((
                Text::new(&current_menu.title),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Key sequence display
            if !state.key_sequence.is_empty() {
                let sequence: String = state.key_sequence.iter().collect();
                parent.spawn((
                    Text::new(format!("Keys: {}", sequence)),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.8, 0.5)),
                    Node {
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                ));
            }

            // Menu items
            for item in &current_menu.items {
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    })
                    .with_children(|item_parent| {
                        // Key
                        item_parent.spawn((
                            Text::new(format!("[{}]", item.key)),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.8, 0.0)),
                            Node {
                                margin: UiRect::right(Val::Px(10.0)),
                                ..default()
                            },
                        ));

                        // Label
                        item_parent.spawn((
                            Text::new(&item.label),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Node {
                                margin: UiRect::right(Val::Px(10.0)),
                                ..default()
                            },
                        ));

                        // Description
                        item_parent.spawn((
                            Text::new(&item.description),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                        ));
                    });
            }

            // Timeout indicator
            if let Some(last_press) = state.last_press {
                let elapsed = last_press.elapsed();
                let remaining = state.timeout.saturating_sub(elapsed);
                let progress = (remaining.as_millis() as f32) / (state.timeout.as_millis() as f32);

                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(4.0),
                            margin: UiRect::top(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.5)),
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            Node {
                                width: Val::Percent(progress * 100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.0, 0.8, 0.0)),
                        ));
                    });
            }
        });
}

/// Check for timeout and deactivate
fn timeout_check_system(mut state: ResMut<LeaderKeyState>) {
    if !state.active {
        return;
    }

    if let Some(last_press) = state.last_press {
        if last_press.elapsed() >= state.timeout {
            info!("Leader key timeout");
            state.active = false;
            state.key_sequence.clear();
        }
    }
}

/// Convert KeyCode to character
fn keycode_to_char(keycode: KeyCode) -> Option<char> {
    match keycode {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        _ => None,
    }
}

/// Initialize default leader key menus
pub fn register_default_menus(menus: &mut LeaderKeyMenus) {
    // Root menu
    let mut root_menu = Menu {
        title: "Leader Menu".to_string(),
        items: Vec::new(),
    };

    root_menu.items.push(MenuItem {
        key: 'b',
        label: "Buffer".to_string(),
        description: "Buffer operations".to_string(),
        action: MenuAction::SubMenu("buffer".to_string()),
    });

    root_menu.items.push(MenuItem {
        key: 'w',
        label: "Window".to_string(),
        description: "Window operations".to_string(),
        action: MenuAction::SubMenu("window".to_string()),
    });

    root_menu.items.push(MenuItem {
        key: 'f',
        label: "File".to_string(),
        description: "File operations".to_string(),
        action: MenuAction::SubMenu("file".to_string()),
    });

    menus.register("root", root_menu);

    // Buffer submenu
    let mut buffer_menu = Menu {
        title: "Buffer Operations".to_string(),
        items: Vec::new(),
    };

    buffer_menu.items.push(MenuItem {
        key: 'c',
        label: "Clear".to_string(),
        description: "Clear buffer".to_string(),
        action: MenuAction::Command("buffer.clear".to_string()),
    });

    buffer_menu.items.push(MenuItem {
        key: 's',
        label: "Save".to_string(),
        description: "Save buffer to file".to_string(),
        action: MenuAction::Command("buffer.save".to_string()),
    });

    menus.register("buffer", buffer_menu);

    // Window submenu
    let mut window_menu = Menu {
        title: "Window Operations".to_string(),
        items: Vec::new(),
    };

    window_menu.items.push(MenuItem {
        key: 's',
        label: "Split".to_string(),
        description: "Split window".to_string(),
        action: MenuAction::Command("window.split".to_string()),
    });

    window_menu.items.push(MenuItem {
        key: 'v',
        label: "VSplit".to_string(),
        description: "Split window vertically".to_string(),
        action: MenuAction::Command("window.vsplit".to_string()),
    });

    window_menu.items.push(MenuItem {
        key: 'c',
        label: "Close".to_string(),
        description: "Close current window".to_string(),
        action: MenuAction::Command("window.close".to_string()),
    });

    menus.register("window", window_menu);
}
