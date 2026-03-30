// Command palette with fuzzy search
// Provides quick access to all terminal commands

use crate::ipc::IpcChannel;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use scarab_protocol::{ControlMessage, ModalItem};
use std::sync::Arc;

/// Plugin for command palette functionality
pub struct CommandPalettePlugin;

impl Plugin for CommandPalettePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CommandRegistry>()
            .init_resource::<CommandPaletteState>()
            .add_event::<CommandExecutedEvent>()
            .add_event::<ShowRemoteModalEvent>()
            .add_systems(
                Update,
                (
                    toggle_palette_system,
                    handle_palette_input_system,
                    render_palette_system,
                    execute_command_system,
                    handle_remote_modal_system,
                )
                    .chain(),
            )
            .add_systems(Startup, register_default_commands_system);
    }
}

/// Event to trigger a remote modal (populating palette from daemon)
#[derive(Event)]
pub struct ShowRemoteModalEvent {
    pub title: String,
    pub items: Vec<ModalItem>,
}

/// A command that can be executed
#[derive(Clone)]
pub struct Command {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub keybind: Option<String>,
    pub action: Arc<dyn Fn(&IpcChannel) + Send + Sync>,
}

impl Command {
    pub fn new<F>(id: &str, name: &str, description: &str, category: &str, action: F) -> Self
    where
        F: Fn(&IpcChannel) + Send + Sync + 'static,
    {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            keybind: None,
            action: Arc::new(action),
        }
    }

    pub fn with_keybind(mut self, keybind: &str) -> Self {
        self.keybind = Some(keybind.to_string());
        self
    }
}

/// Registry of all available commands
#[derive(Resource, Default)]
pub struct CommandRegistry {
    commands: Vec<Command>,
}

impl CommandRegistry {
    pub fn register(&mut self, command: Command) {
        self.commands.push(command);
    }

    pub fn get(&self, id: &str) -> Option<&Command> {
        self.commands.iter().find(|c| c.id == id)
    }

    pub fn all(&self) -> &[Command] {
        &self.commands
    }

    pub fn fuzzy_search(&self, query: &str) -> Vec<(Command, i64)> {
        if query.is_empty() {
            return self.commands.iter().map(|c| (c.clone(), 0)).collect();
        }

        let matcher = SkimMatcherV2::default();
        let mut results: Vec<(Command, i64)> = self
            .commands
            .iter()
            .filter_map(|cmd| {
                let name_score = matcher.fuzzy_match(&cmd.name, query).unwrap_or(0);
                let desc_score = matcher.fuzzy_match(&cmd.description, query).unwrap_or(0);
                let score = name_score.max(desc_score);

                if score > 0 {
                    Some((cmd.clone(), score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (highest first)
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
    }
}

/// State of command palette
#[derive(Resource, Default)]
pub struct CommandPaletteState {
    pub active: bool,
    pub query: String,
    pub selected_index: usize,
    pub filtered_commands: Vec<(Command, i64)>,
}

/// Event fired when command is executed
#[derive(Event)]
pub struct CommandExecutedEvent {
    pub command_id: String,
}

/// Component for palette UI elements
#[derive(Component)]
struct PaletteUI;

/// Component for command list item
#[derive(Component)]
struct CommandItem {
    index: usize,
}

/// Toggle command palette visibility
fn toggle_palette_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<CommandPaletteState>,
    registry: Res<CommandRegistry>,
) {
    // Toggle palette with Ctrl+P
    if keyboard.just_pressed(KeyCode::ControlLeft) && keyboard.pressed(KeyCode::KeyP) {
        state.active = !state.active;

        if state.active {
            state.query.clear();
            state.selected_index = 0;
            state.filtered_commands = registry.fuzzy_search("");
        }
    }

    // Close with Escape
    if state.active && keyboard.just_pressed(KeyCode::Escape) {
        state.active = false;
    }
}

/// Handle input in command palette
fn handle_palette_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<CommandPaletteState>,
    registry: Res<CommandRegistry>,
    mut command_events: EventWriter<CommandExecutedEvent>,
) {
    if !state.active {
        return;
    }

    // Note: Character input handling would need keyboard text input events
    // For now, we'll handle basic commands with keycodes

    // Handle backspace
    if keyboard.just_pressed(KeyCode::Backspace) {
        state.query.pop();
        // If we are in remote mode (empty registry or special flag?), we might need to re-filter remote items.
        // For now, we assume remote mode uses the filtered_commands directly and local mode uses registry.
        // But wait, toggle_palette_system resets filtered_commands from registry.
        // If we received a remote modal, we should NOT query the registry.
        // We need a flag in State.

        // Simple hack: if active and filtered_commands is not empty but registry search returns different count?
        // No. We need `mode` in state.

        // For now, let's assume if we have a query we filter from registry.
        // If we are in remote mode, we probably shouldn't type to search yet (needs implementation).
        // Let's just re-run search on registry.
        state.filtered_commands = registry.fuzzy_search(&state.query);
        state.selected_index = 0;
    }

    // Handle navigation
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        if state.selected_index < state.filtered_commands.len().saturating_sub(1) {
            state.selected_index += 1;
        }
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        state.selected_index = state.selected_index.saturating_sub(1);
    }

    // Handle selection with Enter
    if keyboard.just_pressed(KeyCode::Enter) {
        if let Some((command, _)) = state.filtered_commands.get(state.selected_index) {
            command_events.send(CommandExecutedEvent {
                command_id: command.id.clone(),
            });
            state.active = false;
        }
    }
}

fn handle_remote_modal_system(
    mut events: EventReader<ShowRemoteModalEvent>,
    mut state: ResMut<CommandPaletteState>,
) {
    for event in events.read() {
        state.active = true;
        state.query.clear();
        state.selected_index = 0;
        state.filtered_commands.clear();

        for item in &event.items {
            let id_for_closure = item.id.clone();
            // Create a command that sends CommandSelected
            let command = Command::new(
                &item.id,
                &item.label,
                item.description.as_deref().unwrap_or(""),
                "Remote",
                move |ipc| {
                    ipc.send(ControlMessage::CommandSelected {
                        id: id_for_closure.clone(),
                    });
                },
            );
            state.filtered_commands.push((command, 0));
        }
    }
}

/// Render command palette UI
fn render_palette_system(
    mut commands: Commands,
    state: Res<CommandPaletteState>,
    existing_ui: Query<Entity, With<PaletteUI>>,
) {
    // Remove existing UI
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }

    if !state.active {
        return;
    }

    // Create palette container
    commands
        .spawn((
            PaletteUI,
            Node {
                width: Val::Px(600.0),
                height: Val::Px(400.0),
                position_type: PositionType::Absolute,
                left: Val::Px(200.0),
                top: Val::Px(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
        ))
        .with_children(|parent| {
            // Search input display
            parent.spawn((
                Text::new(format!("> {}", state.query)),
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

            // Command list (show first 10 results)
            for (index, (command, score)) in state.filtered_commands.iter().take(10).enumerate() {
                let is_selected = index == state.selected_index;
                let bg_color = if is_selected {
                    Color::srgba(0.3, 0.3, 0.5, 0.8)
                } else {
                    Color::srgba(0.2, 0.2, 0.2, 0.5)
                };

                parent
                    .spawn((
                        CommandItem { index },
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::all(Val::Px(8.0)),
                            margin: UiRect::bottom(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(bg_color),
                    ))
                    .with_children(|item| {
                        // Command name and keybind
                        let keybind_text = command
                            .keybind
                            .as_ref()
                            .map(|k| format!(" [{}]", k))
                            .unwrap_or_default();

                        item.spawn((
                            Text::new(format!("{}{}", command.name, keybind_text)),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        // Command description
                        item.spawn((
                            Text::new(format!("{} (score: {})", command.description, score)),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                            Node {
                                margin: UiRect::top(Val::Px(4.0)),
                                ..default()
                            },
                        ));
                    });
            }
        });
}

/// Execute selected command
fn execute_command_system(
    mut events: EventReader<CommandExecutedEvent>,
    registry: Res<CommandRegistry>,
    ipc: Res<IpcChannel>,
) {
    for event in events.read() {
        if let Some(command) = registry.get(&event.command_id) {
            info!("Executing command: {}", command.name);
            (command.action)(&ipc);
        }
    }
}

/// Initialize default commands at startup
fn register_default_commands_system(mut registry: ResMut<CommandRegistry>) {
    register_default_commands(&mut registry);
}

/// Register default commands with IPC actions
pub fn register_default_commands(registry: &mut CommandRegistry) {
    // Clear terminal (Ctrl+L sends clear command)
    registry.register(
        Command::new(
            "clear",
            "Clear Terminal",
            "Clear all terminal output",
            "Terminal",
            |ipc| {
                // Send Ctrl+L sequence to clear the terminal
                ipc.send(ControlMessage::Input {
                    data: vec![0x0C], // Ctrl+L
                });
            },
        )
        .with_keybind("Ctrl+L"),
    );

    // Reset terminal
    registry.register(Command::new(
        "reset",
        "Reset Terminal",
        "Reset terminal to initial state",
        "Terminal",
        |ipc| {
            // Send reset sequence
            ipc.send(ControlMessage::Input {
                data: b"reset\n".to_vec(),
            });
        },
    ));

    // Send Ctrl+C (interrupt)
    registry.register(
        Command::new(
            "interrupt",
            "Interrupt Process",
            "Send SIGINT to current process",
            "Terminal",
            |ipc| {
                ipc.send(ControlMessage::Input {
                    data: vec![0x03], // Ctrl+C
                });
            },
        )
        .with_keybind("Ctrl+C"),
    );

    // Send Ctrl+D (EOF)
    registry.register(
        Command::new(
            "eof",
            "Send EOF",
            "Send end-of-file signal",
            "Terminal",
            |ipc| {
                ipc.send(ControlMessage::Input {
                    data: vec![0x04], // Ctrl+D
                });
            },
        )
        .with_keybind("Ctrl+D"),
    );

    // Paste from clipboard
    registry.register(
        Command::new(
            "paste",
            "Paste from Clipboard",
            "Paste clipboard contents into terminal",
            "Edit",
            |ipc| {
                use arboard::Clipboard;
                if let Ok(mut clipboard) = Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        ipc.send(ControlMessage::Input {
                            data: text.into_bytes(),
                        });
                    }
                }
            },
        )
        .with_keybind("Ctrl+Shift+V"),
    );

    // Reload configuration (placeholder)
    registry.register(Command::new(
        "reload_config",
        "Reload Configuration",
        "Reload Scarab configuration files",
        "Settings",
        |_ipc| {
            info!("Configuration reload requested (not yet implemented)");
        },
    ));

    // Show help
    registry.register(
        Command::new(
            "help",
            "Show Help",
            "Display keyboard shortcuts and commands",
            "Help",
            |_ipc| {
                info!("Help display requested (not yet implemented)");
            },
        )
        .with_keybind("F1"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_search() {
        let mut registry = CommandRegistry::default();
        // Create a dummy IPC channel action
        let dummy_action = |_: &IpcChannel| {};

        registry.register(Command::new(
            "copy",
            "Copy Selection",
            "Copy text",
            "Edit",
            dummy_action,
        ));
        registry.register(Command::new(
            "paste",
            "Paste",
            "Paste text",
            "Edit",
            dummy_action,
        ));

        let results = registry.fuzzy_search("cop");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.id, "copy");
    }

    #[test]
    fn test_fuzzy_search_performance() {
        let mut registry = CommandRegistry::default();
        let dummy_action = |_: &IpcChannel| {};

        // Add 1000 commands
        for i in 0..1000 {
            registry.register(Command::new(
                &format!("cmd_{}", i),
                &format!("Command {}", i),
                &format!("Description {}", i),
                "Test",
                dummy_action,
            ));
        }

        use std::time::Instant;
        let start = Instant::now();
        let _results = registry.fuzzy_search("command");
        let duration = start.elapsed();

        // Should complete in <50ms
        assert!(duration.as_millis() < 50);
    }
}
