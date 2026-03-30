//! Plugin Inspector UI for debugging and monitoring plugins
//!
//! Provides a visual debugger for plugin developers with:
//! - List of loaded plugins with status indicators
//! - Plugin metadata and configuration
//! - Failure tracking and error messages
//! - Hook execution history and timing
//! - Real-time log output
//! - Enable/disable/reload controls

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use scarab_protocol::{ControlMessage, DaemonMessage, PluginInspectorInfo};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::ipc::{IpcChannel, RemoteMessageEvent};

/// Maximum number of log entries to retain
const MAX_LOG_ENTRIES: usize = 1000;
/// Maximum number of hook executions to track
const MAX_HOOK_HISTORY: usize = 500;

/// Resource tracking plugin inspector state
#[derive(Resource, Default)]
pub struct PluginInspectorState {
    /// Whether the inspector window is visible
    pub visible: bool,
    /// List of plugins from daemon
    pub plugins: Vec<InspectedPlugin>,
    /// Selected plugin index
    pub selected_plugin: Option<usize>,
    /// Log buffer for plugin output
    pub logs: VecDeque<LogEntry>,
    /// Hook execution history
    pub hook_history: VecDeque<HookExecution>,
    /// Filter text for plugins
    pub filter_text: String,
    /// Filter text for logs
    pub log_filter: String,
    /// Whether to auto-scroll logs
    pub auto_scroll_logs: bool,
    /// Selected tab in details panel
    pub selected_tab: InspectorTab,
}

/// Plugin inspection data
#[derive(Clone, Debug)]
pub struct InspectedPlugin {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub enabled: bool,
    pub failure_count: u32,
    pub api_version: String,
    pub min_scarab_version: String,
    /// Computed metrics
    pub total_executions: u64,
    pub total_execution_time: Duration,
    pub last_error: Option<String>,
}

impl From<PluginInspectorInfo> for InspectedPlugin {
    fn from(info: PluginInspectorInfo) -> Self {
        Self {
            name: info.name.to_string(),
            version: info.version.to_string(),
            description: info.description.to_string(),
            author: info.author.to_string(),
            homepage: info.homepage.as_ref().map(|s| s.to_string()),
            enabled: info.enabled,
            failure_count: info.failure_count,
            api_version: info.api_version.to_string(),
            min_scarab_version: info.min_scarab_version.to_string(),
            total_executions: 0,
            total_execution_time: Duration::ZERO,
            last_error: None,
        }
    }
}

/// Log entry from plugin or system
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: Instant,
    pub level: LogLevel,
    pub plugin: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn color(&self) -> egui::Color32 {
        match self {
            LogLevel::Trace => egui::Color32::GRAY,
            LogLevel::Debug => egui::Color32::from_rgb(150, 150, 255),
            LogLevel::Info => egui::Color32::WHITE,
            LogLevel::Warn => egui::Color32::from_rgb(255, 200, 0),
            LogLevel::Error => egui::Color32::from_rgb(255, 80, 80),
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            LogLevel::Trace => "◦",
            LogLevel::Debug => "▸",
            LogLevel::Info => "ℹ",
            LogLevel::Warn => "⚠",
            LogLevel::Error => "✖",
        }
    }
}

/// Hook execution record
#[derive(Clone, Debug)]
pub struct HookExecution {
    pub timestamp: Instant,
    pub plugin: String,
    pub hook_type: String,
    pub duration: Duration,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectorTab {
    Overview,
    Metadata,
    Hooks,
    Logs,
    Source,
}

impl PluginInspectorState {
    pub fn new() -> Self {
        Self {
            visible: false,
            plugins: Vec::new(),
            selected_plugin: None,
            logs: VecDeque::new(),
            hook_history: VecDeque::new(),
            filter_text: String::new(),
            log_filter: String::new(),
            auto_scroll_logs: true,
            selected_tab: InspectorTab::Overview,
        }
    }

    pub fn add_log(&mut self, level: LogLevel, plugin: Option<String>, message: String) {
        self.logs.push_back(LogEntry {
            timestamp: Instant::now(),
            level,
            plugin,
            message,
        });

        if self.logs.len() > MAX_LOG_ENTRIES {
            self.logs.pop_front();
        }
    }

    pub fn add_hook_execution(&mut self, execution: HookExecution) {
        self.hook_history.push_back(execution);

        if self.hook_history.len() > MAX_HOOK_HISTORY {
            self.hook_history.pop_front();
        }

        // Update plugin metrics
        if let Some(plugin) = self
            .plugins
            .iter_mut()
            .find(|p| p.name == execution.plugin)
        {
            plugin.total_executions += 1;
            plugin.total_execution_time += execution.duration;
            if !execution.success {
                plugin.last_error = execution.error.clone();
            }
        }
    }

    pub fn filtered_plugins(&self) -> Vec<(usize, &InspectedPlugin)> {
        let filter_lower = self.filter_text.to_lowercase();
        self.plugins
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                filter_lower.is_empty()
                    || p.name.to_lowercase().contains(&filter_lower)
                    || p.description.to_lowercase().contains(&filter_lower)
                    || p.author.to_lowercase().contains(&filter_lower)
            })
            .collect()
    }

    pub fn filtered_logs(&self) -> Vec<&LogEntry> {
        let filter_lower = self.log_filter.to_lowercase();
        self.logs
            .iter()
            .filter(|log| {
                filter_lower.is_empty()
                    || log.message.to_lowercase().contains(&filter_lower)
                    || log
                        .plugin
                        .as_ref()
                        .map_or(false, |p| p.to_lowercase().contains(&filter_lower))
            })
            .collect()
    }

    pub fn selected_plugin(&self) -> Option<&InspectedPlugin> {
        self.selected_plugin.and_then(|idx| self.plugins.get(idx))
    }

    pub fn update_plugins(&mut self, plugin_list: Vec<PluginInspectorInfo>) {
        // Preserve metrics from existing plugins
        let old_plugins: std::collections::HashMap<String, InspectedPlugin> = self
            .plugins
            .drain(..)
            .map(|p| (p.name.clone(), p))
            .collect();

        self.plugins = plugin_list
            .into_iter()
            .map(|info| {
                let mut plugin = InspectedPlugin::from(info);
                // Restore metrics if plugin existed before
                if let Some(old) = old_plugins.get(&plugin.name) {
                    plugin.total_executions = old.total_executions;
                    plugin.total_execution_time = old.total_execution_time;
                    plugin.last_error = old.last_error.clone();
                }
                plugin
            })
            .collect();
    }
}

/// System to toggle inspector with Ctrl+Shift+P
pub fn toggle_inspector_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<PluginInspectorState>,
    ipc: Option<Res<IpcChannel>>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if ctrl && shift && keys.just_pressed(KeyCode::KeyP) {
        state.visible = !state.visible;
        if state.visible {
            state.add_log(
                LogLevel::Info,
                None,
                "Plugin Inspector opened".to_string(),
            );

            // Request fresh plugin list from daemon
            if let Some(ipc) = ipc {
                ipc.send(ControlMessage::PluginListRequest);
            }
        }
    }
}

/// System to render the plugin inspector UI
pub fn render_inspector_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<PluginInspectorState>,
    ipc: Option<Res<IpcChannel>>,
) {
    if !state.visible {
        return;
    }

    let ctx = contexts.ctx_mut();

    egui::Window::new("Plugin Inspector")
        .resizable(true)
        .default_width(1000.0)
        .default_height(700.0)
        .show(ctx, |ui| {
            // Main layout: sidebar + content
            egui::TopBottomPanel::top("toolbar")
                .exact_height(40.0)
                .show_inside(ui, |ui| {
                    render_toolbar(ui, &mut state, ipc.as_ref());
                });

            egui::SidePanel::left("plugin_list")
                .default_width(300.0)
                .resizable(true)
                .show_inside(ui, |ui| {
                    render_plugin_list(ui, &mut state);
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                render_plugin_details(ui, &mut state, ipc.as_ref());
            });
        });
}

fn render_toolbar(
    ui: &mut egui::Ui,
    state: &mut PluginInspectorState,
    ipc: Option<&IpcChannel>,
) {
    ui.horizontal(|ui| {
        ui.heading("Plugin Inspector");

        ui.separator();

        // Statistics
        let enabled_count = state.plugins.iter().filter(|p| p.enabled).count();
        let total_count = state.plugins.len();
        let failed_count = state.plugins.iter().filter(|p| p.failure_count > 0).count();

        ui.label(format!("Total: {}", total_count));
        ui.colored_label(egui::Color32::GREEN, format!("Enabled: {}", enabled_count));
        if failed_count > 0 {
            ui.colored_label(egui::Color32::RED, format!("Failed: {}", failed_count));
        }

        ui.separator();

        // Actions
        if ui.button("Refresh").clicked() {
            if let Some(ipc) = ipc {
                ipc.send(ControlMessage::PluginListRequest);
                state.add_log(LogLevel::Debug, None, "Requesting plugin list...".to_string());
            }
        }

        if ui.button("Clear Logs").clicked() {
            state.logs.clear();
            state.add_log(LogLevel::Info, None, "Logs cleared".to_string());
        }

        if ui.button("Export Debug Info").clicked() {
            export_debug_info(state);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("✖ Close").clicked() {
                state.visible = false;
            }
        });
    });
}

fn render_plugin_list(ui: &mut egui::Ui, state: &mut PluginInspectorState) {
    ui.vertical(|ui| {
        ui.heading("Plugins");

        // Filter input
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut state.filter_text);
        });

        ui.separator();

        // Plugin list
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let filtered = state.filtered_plugins();

                if filtered.is_empty() {
                    ui.colored_label(egui::Color32::GRAY, "No plugins found");
                    return;
                }

                for (idx, plugin) in filtered {
                    let is_selected = state.selected_plugin == Some(idx);

                    let color = if !plugin.enabled {
                        egui::Color32::GRAY
                    } else if plugin.failure_count > 0 {
                        egui::Color32::from_rgb(255, 100, 100)
                    } else {
                        egui::Color32::from_rgb(100, 255, 100)
                    };

                    ui.horizontal(|ui| {
                        // Status indicator
                        ui.colored_label(color, "●");

                        // Plugin name button
                        if ui
                            .selectable_label(is_selected, &plugin.name)
                            .clicked()
                        {
                            state.selected_plugin = Some(idx);
                            state.selected_tab = InspectorTab::Overview;
                        }
                    });

                    // Show brief info
                    ui.indent(idx.to_string(), |ui| {
                        ui.small(&plugin.version);
                        if plugin.failure_count > 0 {
                            ui.small(
                                egui::RichText::new(format!(
                                    "Failures: {}",
                                    plugin.failure_count
                                ))
                                .color(egui::Color32::RED),
                            );
                        }
                    });

                    ui.separator();
                }
            });
    });
}

fn render_plugin_details(ui: &mut egui::Ui, state: &mut PluginInspectorState, ipc: Option<&IpcChannel>) {
    let Some(plugin) = state.selected_plugin() else {
        ui.centered_and_justified(|ui| {
            ui.colored_label(egui::Color32::GRAY, "Select a plugin to view details");
        });
        return;
    };

    let plugin = plugin.clone(); // Clone to avoid borrow issues

    ui.vertical(|ui| {
        // Tab bar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut state.selected_tab, InspectorTab::Overview, "Overview");
            ui.selectable_value(&mut state.selected_tab, InspectorTab::Metadata, "Metadata");
            ui.selectable_value(&mut state.selected_tab, InspectorTab::Hooks, "Hooks");
            ui.selectable_value(&mut state.selected_tab, InspectorTab::Logs, "Logs");
            ui.selectable_value(&mut state.selected_tab, InspectorTab::Source, "Source");
        });

        ui.separator();

        // Tab content
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| match state.selected_tab {
                InspectorTab::Overview => render_overview_tab(ui, &plugin, state, ipc),
                InspectorTab::Metadata => render_metadata_tab(ui, &plugin),
                InspectorTab::Hooks => render_hooks_tab(ui, &plugin, state),
                InspectorTab::Logs => render_logs_tab(ui, state),
                InspectorTab::Source => render_source_tab(ui, &plugin),
            });
    });
}

fn render_overview_tab(
    ui: &mut egui::Ui,
    plugin: &InspectedPlugin,
    state: &mut PluginInspectorState,
    ipc: Option<&IpcChannel>,
) {
    ui.heading(&plugin.name);
    ui.label(&plugin.description);

    ui.add_space(10.0);

    // Status card
    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(30, 30, 40))
        .show(ui, |ui| {
            ui.heading("Status");

            egui::Grid::new("status_grid")
                .num_columns(2)
                .spacing([20.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Enabled:");
                    if plugin.enabled {
                        ui.colored_label(egui::Color32::GREEN, "Yes");
                    } else {
                        ui.colored_label(egui::Color32::RED, "No");
                    }
                    ui.end_row();

                    ui.label("Failure Count:");
                    if plugin.failure_count > 0 {
                        ui.colored_label(
                            egui::Color32::RED,
                            format!("{}", plugin.failure_count),
                        );
                    } else {
                        ui.colored_label(egui::Color32::GREEN, "0");
                    }
                    ui.end_row();

                    ui.label("Total Executions:");
                    ui.label(format!("{}", plugin.total_executions));
                    ui.end_row();

                    ui.label("Total Execution Time:");
                    ui.label(format!("{:.2}ms", plugin.total_execution_time.as_secs_f64() * 1000.0));
                    ui.end_row();

                    if plugin.total_executions > 0 {
                        ui.label("Avg Execution Time:");
                        let avg_ms = plugin.total_execution_time.as_secs_f64() * 1000.0
                            / plugin.total_executions as f64;
                        ui.label(format!("{:.3}ms", avg_ms));
                        ui.end_row();
                    }
                });
        });

    ui.add_space(10.0);

    // Last error
    if let Some(error) = &plugin.last_error {
        egui::Frame::group(ui.style())
            .fill(egui::Color32::from_rgb(60, 20, 20))
            .show(ui, |ui| {
                ui.heading("Last Error");
                ui.colored_label(egui::Color32::RED, error);
            });
    }

    ui.add_space(10.0);

    // Actions
    ui.horizontal(|ui| {
        if ui.button(if plugin.enabled { "Disable" } else { "Enable" }).clicked() {
            if let Some(ipc) = ipc {
                let msg = if plugin.enabled {
                    ControlMessage::PluginDisable {
                        name: plugin.name.clone(),
                    }
                } else {
                    ControlMessage::PluginEnable {
                        name: plugin.name.clone(),
                    }
                };
                ipc.send(msg);
                state.add_log(
                    LogLevel::Info,
                    Some(plugin.name.clone()),
                    format!("{} plugin", if plugin.enabled { "Disabling" } else { "Enabling" }),
                );
            }
        }

        if ui.button("Reload").clicked() {
            if let Some(ipc) = ipc {
                ipc.send(ControlMessage::PluginReload {
                    name: plugin.name.clone(),
                });
                state.add_log(
                    LogLevel::Info,
                    Some(plugin.name.clone()),
                    "Reloading plugin".to_string(),
                );
            }
        }
    });
}

fn render_metadata_tab(ui: &mut egui::Ui, plugin: &InspectedPlugin) {
    egui::Grid::new("metadata_grid")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Name:").strong());
            ui.label(&plugin.name);
            ui.end_row();

            ui.label(egui::RichText::new("Version:").strong());
            ui.label(&plugin.version);
            ui.end_row();

            ui.label(egui::RichText::new("Description:").strong());
            ui.label(&plugin.description);
            ui.end_row();

            ui.label(egui::RichText::new("Author:").strong());
            ui.label(&plugin.author);
            ui.end_row();

            ui.label(egui::RichText::new("Homepage:").strong());
            if let Some(homepage) = &plugin.homepage {
                ui.hyperlink(homepage);
            } else {
                ui.colored_label(egui::Color32::GRAY, "N/A");
            }
            ui.end_row();

            ui.label(egui::RichText::new("API Version:").strong());
            ui.label(&plugin.api_version);
            ui.end_row();

            ui.label(egui::RichText::new("Min Scarab Version:").strong());
            ui.label(&plugin.min_scarab_version);
            ui.end_row();
        });
}

fn render_hooks_tab(ui: &mut egui::Ui, plugin: &InspectedPlugin, state: &PluginInspectorState) {
    ui.heading("Hook Execution History");

    // Filter to selected plugin
    let plugin_hooks: Vec<_> = state
        .hook_history
        .iter()
        .filter(|h| h.plugin == plugin.name)
        .collect();

    if plugin_hooks.is_empty() {
        ui.colored_label(egui::Color32::GRAY, "No hook executions recorded");
        return;
    }

    // Statistics
    let total = plugin_hooks.len();
    let successful = plugin_hooks.iter().filter(|h| h.success).count();
    let failed = total - successful;
    let total_time: Duration = plugin_hooks.iter().map(|h| h.duration).sum();
    let avg_time = total_time / total as u32;

    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(30, 30, 40))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Total: {}", total));
                ui.separator();
                ui.colored_label(egui::Color32::GREEN, format!("Success: {}", successful));
                ui.separator();
                ui.colored_label(egui::Color32::RED, format!("Failed: {}", failed));
                ui.separator();
                ui.label(format!("Avg Time: {:.3}ms", avg_time.as_secs_f64() * 1000.0));
            });
        });

    ui.add_space(10.0);

    // Hook list
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for hook in plugin_hooks.iter().rev().take(100) {
                let elapsed = hook.timestamp.elapsed();
                let color = if hook.success {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::RED
                };

                ui.horizontal(|ui| {
                    ui.colored_label(color, if hook.success { "✓" } else { "✖" });
                    ui.label(format!("{:.1}s ago", elapsed.as_secs_f32()));
                    ui.label(&hook.hook_type);
                    ui.label(format!("{:.3}ms", hook.duration.as_secs_f64() * 1000.0));

                    if let Some(error) = &hook.error {
                        ui.colored_label(egui::Color32::RED, error);
                    }
                });
                ui.separator();
            }
        });
}

fn render_logs_tab(ui: &mut egui::Ui, state: &mut PluginInspectorState) {
    // Log controls
    ui.horizontal(|ui| {
        ui.label("Filter:");
        ui.text_edit_singleline(&mut state.log_filter);

        ui.checkbox(&mut state.auto_scroll_logs, "Auto-scroll");

        if ui.button("Clear").clicked() {
            state.logs.clear();
        }
    });

    ui.separator();

    // Log entries
    let mut scroll_area = egui::ScrollArea::vertical().auto_shrink([false; 2]);

    if state.auto_scroll_logs {
        scroll_area = scroll_area.stick_to_bottom(true);
    }

    scroll_area.show(ui, |ui| {
        let logs = state.filtered_logs();

        if logs.is_empty() {
            ui.colored_label(egui::Color32::GRAY, "No logs");
            return;
        }

        for log in logs {
            let elapsed = log.timestamp.elapsed();

            ui.horizontal(|ui| {
                ui.colored_label(log.level.color(), log.level.icon());

                ui.label(format!("{:.1}s", elapsed.as_secs_f32()));

                if let Some(plugin) = &log.plugin {
                    ui.colored_label(egui::Color32::from_rgb(100, 200, 255), plugin);
                }

                ui.label(&log.message);
            });
        }
    });
}

fn render_source_tab(ui: &mut egui::Ui, plugin: &InspectedPlugin) {
    ui.heading("Plugin Source");

    ui.colored_label(
        egui::Color32::GRAY,
        "Source code viewing not yet implemented",
    );

    ui.add_space(10.0);

    ui.label("In a future version, this tab will display:");
    ui.label("- Plugin source code (.fsx files)");
    ui.label("- Compiled bytecode information (.fzb files)");
    ui.label("- Plugin configuration");
    ui.label("- Dependency tree");

    ui.add_space(10.0);

    ui.label(format!("Plugin: {}", plugin.name));
}

fn export_debug_info(state: &PluginInspectorState) {
    // Export debug information to a file
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let filename = format!("scarab-plugin-debug-{}.txt", timestamp);

    let mut content = String::new();
    content.push_str("Scarab Plugin Inspector Debug Export\n");
    content.push_str(&format!("Timestamp: {}\n\n", timestamp));

    content.push_str("=== PLUGINS ===\n");
    for plugin in &state.plugins {
        content.push_str(&format!("\nName: {}\n", plugin.name));
        content.push_str(&format!("Version: {}\n", plugin.version));
        content.push_str(&format!("Enabled: {}\n", plugin.enabled));
        content.push_str(&format!("Failures: {}\n", plugin.failure_count));
        content.push_str(&format!("Total Executions: {}\n", plugin.total_executions));
        if let Some(error) = &plugin.last_error {
            content.push_str(&format!("Last Error: {}\n", error));
        }
    }

    content.push_str("\n=== LOGS ===\n");
    for log in &state.logs {
        content.push_str(&format!(
            "[{:?}] {:?}: {}\n",
            log.level, log.plugin, log.message
        ));
    }

    // Write to file
    if let Err(e) = std::fs::write(&filename, content) {
        eprintln!("Failed to export debug info: {}", e);
    } else {
        println!("Debug info exported to: {}", filename);
    }
}

/// System to handle daemon messages related to plugins
pub fn handle_plugin_messages(
    mut events: EventReader<RemoteMessageEvent>,
    mut state: ResMut<PluginInspectorState>,
) {
    for event in events.read() {
        match &event.0 {
            DaemonMessage::PluginList { plugins } => {
                state.update_plugins(plugins.clone());
                state.add_log(
                    LogLevel::Info,
                    None,
                    format!("Received plugin list: {} plugins", plugins.len()),
                );
            }
            DaemonMessage::PluginStatusChanged { name, enabled } => {
                if let Some(plugin) = state.plugins.iter_mut().find(|p| p.name == name.as_str()) {
                    plugin.enabled = *enabled;
                    state.add_log(
                        LogLevel::Info,
                        Some(name.to_string()),
                        format!("Plugin {} {}", name, if *enabled { "enabled" } else { "disabled" }),
                    );
                }
            }
            DaemonMessage::PluginError { name, error } => {
                if let Some(plugin) = state.plugins.iter_mut().find(|p| p.name == name.as_str()) {
                    plugin.last_error = Some(error.to_string());
                    state.add_log(
                        LogLevel::Error,
                        Some(name.to_string()),
                        error.to_string(),
                    );
                }
            }
            _ => {}
        }
    }
}

/// Bevy plugin for the plugin inspector
pub struct PluginInspectorPlugin;

impl Plugin for PluginInspectorPlugin {
    fn build(&self, app: &mut App) {
        // Add egui plugin if not already present
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }

        app.insert_resource(PluginInspectorState::new())
            .add_systems(
                Update,
                (
                    toggle_inspector_input,
                    render_inspector_ui,
                    handle_plugin_messages,
                ),
            );

        println!("Plugin Inspector initialized (Ctrl+Shift+P to toggle)");
    }
}
