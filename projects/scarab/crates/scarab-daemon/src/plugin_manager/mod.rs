//! Plugin lifecycle management and hook dispatch

use crate::ipc::ClientRegistry;
use scarab_plugin_api::{
    context::{LogLevel, NotifyLevel},
    delight,
    types::RemoteCommand,
    Achievement, Action, Plugin, PluginConfig, PluginContext, PluginDiscovery, PluginError,
    PluginInfo, PluginMood, Result,
};
use scarab_protocol::DaemonMessage;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::time::timeout;

pub mod fusabi_adapter;
use fusabi_adapter::{FusabiBytecodePlugin, FusabiScriptPlugin};

/// Plugin wrapper with failure tracking and personality
pub struct ManagedPlugin {
    /// The actual plugin instance
    pub plugin: Box<dyn Plugin>,
    /// Plugin configuration (retained for future use in hot-reload)
    pub config: PluginConfig,
    /// Number of consecutive failures
    pub failure_count: u32,
    /// Whether plugin is currently enabled
    pub enabled: bool,
    /// Maximum failures before auto-disable
    max_failures: u32,
    /// Total successful hook executions
    success_count: u64,
}

impl ManagedPlugin {
    fn new(plugin: Box<dyn Plugin>, config: PluginConfig) -> Self {
        Self {
            plugin,
            config,
            failure_count: 0,
            enabled: true,
            max_failures: 3,
            success_count: 0,
        }
    }

    /// Record a failure and potentially disable the plugin
    fn record_failure(&mut self) {
        self.failure_count += 1;
        if self.failure_count >= self.max_failures {
            let mood = PluginMood::from_failure_count(
                self.failure_count,
                self.max_failures,
                self.enabled,
            );
            log::error!(
                "{} Plugin '{}' disabled after {} consecutive failures - {}",
                mood.emoji(),
                self.plugin.metadata().display_name(),
                self.failure_count,
                mood.description()
            );
            self.enabled = false;
        } else {
            let mood = PluginMood::from_failure_count(
                self.failure_count,
                self.max_failures,
                self.enabled,
            );
            log::warn!(
                "{} Plugin '{}' - {} (failures: {})",
                mood.emoji(),
                self.plugin.metadata().display_name(),
                mood.description(),
                self.failure_count
            );
        }
    }

    /// Record successful execution
    fn record_success(&mut self) {
        self.failure_count = 0;
        self.success_count += 1;

        // Celebrate zero failures achievement
        if self.success_count == 100 && self.failure_count == 0 {
            let achievement = Achievement::ZeroFailures;
            log::info!("\n{}", achievement.format());
        }
    }

    /// Get plugin info
    fn info(&self) -> PluginInfo {
        let meta = self.plugin.metadata();
        PluginInfo {
            name: meta.name.clone(),
            version: meta.version.clone(),
            description: meta.description.clone(),
            author: meta.author.clone(),
            homepage: meta.homepage.clone(),
            api_version: meta.api_version.clone(),
            min_scarab_version: meta.min_scarab_version.clone(),
            enabled: self.enabled,
            failure_count: self.failure_count,
            emoji: meta.emoji.clone(),
            color: meta.color.clone(),
            catchphrase: meta.catchphrase.clone(),
        }
    }

    /// Get plugin mood
    fn mood(&self) -> PluginMood {
        PluginMood::from_failure_count(self.failure_count, self.max_failures, self.enabled)
    }
}

/// Plugin manager for loading, managing, and dispatching to plugins
pub struct PluginManager {
    /// Loaded plugins
    pub plugins: Vec<ManagedPlugin>,
    /// Plugin discovery
    discovery: PluginDiscovery,
    /// Hook execution timeout (milliseconds)
    pub hook_timeout: Duration,
    /// Plugin context
    context: Arc<PluginContext>,
    /// Registry for sending commands to clients
    client_registry: ClientRegistry,
    /// Total number of plugins ever loaded (for achievements)
    total_loaded: usize,
}

impl PluginManager {
    /// Create new plugin manager
    pub fn new(context: Arc<PluginContext>, client_registry: ClientRegistry) -> Self {
        // Check for special date messages
        if let Some(special_msg) = delight::special_date_message() {
            log::info!("{}", special_msg);
        }

        Self {
            plugins: Vec::new(),
            discovery: PluginDiscovery::new(),
            hook_timeout: Duration::from_millis(1000),
            context,
            client_registry,
            total_loaded: 0,
        }
    }

    /// Set hook execution timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.hook_timeout = Duration::from_millis(timeout_ms);
        self
    }

    /// Check and celebrate achievements
    fn check_achievements(&self) {
        let enabled_count = self.enabled_count();

        let achievement = match enabled_count {
            1 => Some(Achievement::FirstPlugin),
            10 => Some(Achievement::TenPlugins),
            50 => Some(Achievement::FiftyPlugins),
            100 => Some(Achievement::HundredPlugins),
            _ => None,
        };

        if let Some(ach) = achievement {
            let msg = delight::celebration_message(&ach.format(), ach.ascii_art());
            log::info!("{}", msg);
        }
    }

    /// Convert plugin-api LogLevel to protocol LogLevel
    fn convert_log_level(level: LogLevel) -> scarab_protocol::LogLevel {
        match level {
            LogLevel::Error => scarab_protocol::LogLevel::Error,
            LogLevel::Warn => scarab_protocol::LogLevel::Warn,
            LogLevel::Info => scarab_protocol::LogLevel::Info,
            LogLevel::Debug => scarab_protocol::LogLevel::Debug,
        }
    }

    /// Convert plugin-api NotifyLevel to protocol NotifyLevel
    fn convert_notify_level(level: NotifyLevel) -> scarab_protocol::NotifyLevel {
        match level {
            NotifyLevel::Error => scarab_protocol::NotifyLevel::Error,
            NotifyLevel::Warning => scarab_protocol::NotifyLevel::Warning,
            NotifyLevel::Info => scarab_protocol::NotifyLevel::Info,
            NotifyLevel::Success => scarab_protocol::NotifyLevel::Success,
        }
    }

    /// Process any pending commands queued by plugins
    async fn process_pending_commands(&self) {
        let commands = {
            let mut cmds = self.context.commands.lock();
            std::mem::take(&mut *cmds)
        };

        for cmd in commands {
            match cmd {
                RemoteCommand::DrawOverlay {
                    id,
                    x,
                    y,
                    text,
                    style,
                } => {
                    // Broadcast to all clients for now
                    self.client_registry
                        .broadcast(DaemonMessage::DrawOverlay {
                            id,
                            x,
                            y,
                            text,
                            style,
                        })
                        .await;
                }
                RemoteCommand::ClearOverlays { id } => {
                    self.client_registry
                        .broadcast(DaemonMessage::ClearOverlays { id })
                        .await;
                }
                RemoteCommand::ShowModal { title, items } => {
                    self.client_registry
                        .broadcast(DaemonMessage::ShowModal { title, items })
                        .await;
                }
                RemoteCommand::PluginLog {
                    plugin_name,
                    level,
                    message,
                } => {
                    // Broadcast plugin log to all clients
                    self.client_registry
                        .broadcast(DaemonMessage::PluginLog {
                            plugin_name: plugin_name.into(),
                            level: Self::convert_log_level(level),
                            message: message.into(),
                        })
                        .await;
                }
                RemoteCommand::PluginNotify { title, body, level } => {
                    // Broadcast notification to all clients
                    self.client_registry
                        .broadcast(DaemonMessage::PluginNotification {
                            title: title.into(),
                            body: body.into(),
                            level: Self::convert_notify_level(level),
                        })
                        .await;
                }
            }
        }
    }

    /// Refresh aggregated command list from all plugins
    pub fn refresh_commands(&self) {
        let mut all_commands = Vec::new();
        for managed in &self.plugins {
            if managed.enabled {
                all_commands.extend(managed.plugin.get_commands());
            }
        }
        self.context.state.lock().commands = all_commands;
    }

    /// Load plugins from configuration file
    pub async fn load_from_config(&mut self, config_path: Option<&Path>) -> Result<usize> {
        let configs = self.discovery.load_config(config_path)?;
        let mut loaded = 0;

        log::info!("🔍 Discovering plugins...");

        for config in configs {
            if !config.enabled {
                log::info!("⏭️  Skipping disabled plugin: {}", config.name);
                continue;
            }

            log::info!("⏳ {}", delight::random_loading_message());

            match self.load_plugin_from_config(config).await {
                Ok(_) => loaded += 1,
                Err(e) => {
                    log::error!("{}", e.friendly_message());
                }
            }
        }

        if loaded > 0 {
            log::info!(
                "✨ {} Loaded {} plugin{}!",
                delight::random_success_message(),
                loaded,
                if loaded == 1 { "" } else { "s" }
            );

            // Show a random developer tip
            // TODO: Re-enable when rand dependency is added
            // if rand::random::<f32>() < 0.3 {
            //     log::info!("💡 {}", delight::random_developer_tip());
            // }
        } else {
            log::info!("No plugins loaded. Time to create your first one!");
        }

        Ok(loaded)
    }

    /// Discover and load all plugins from search paths
    pub async fn discover_and_load(&mut self) -> Result<usize> {
        let plugin_files = self.discovery.discover();
        let mut loaded = 0;

        log::info!("🔍 Scanning plugin directories...");

        for path in plugin_files {
            // Create minimal config for discovered plugin
            let config = PluginConfig {
                name: path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                path: path.clone(),
                enabled: true,
                config: Default::default(),
            };

            log::info!("⏳ {}", delight::random_loading_message());

            match self.load_plugin_from_config(config).await {
                Ok(_) => loaded += 1,
                Err(e) => log::warn!("Failed to load plugin from {:?}: {}", path, e),
            }
        }

        if loaded > 0 {
            log::info!("✨ {} Discovered and loaded {} plugin{}!",
                delight::random_success_message(),
                loaded,
                if loaded == 1 { "" } else { "s" }
            );
        }

        Ok(loaded)
    }

    /// Load a single plugin from configuration
    pub async fn load_plugin_from_config(&mut self, config: PluginConfig) -> Result<()> {
        let path = config.expanded_path();

        log::debug!("📦 Loading plugin: {} from {:?}", config.name, path);

        // Check if file exists
        if !path.exists() {
            return Err(PluginError::NotFound(format!("{:?}", path)));
        }

        // Load plugin based on file extension
        let plugin: Box<dyn Plugin> = match path.extension().and_then(|e| e.to_str()) {
            Some("fzb") => {
                log::debug!("⚡ Loading compiled bytecode plugin: {:?}", path);
                Box::new(FusabiBytecodePlugin::load(&path)?)
            }
            Some("fsx") => {
                log::debug!("📜 Loading script plugin: {:?}", path);
                Box::new(FusabiScriptPlugin::load(&path)?)
            }
            _ => {
                return Err(PluginError::LoadError(format!(
                    "Unsupported plugin format: {:?}",
                    path
                )))
            }
        };

        // Register the loaded plugin
        self.register_plugin(plugin).await
    }

    /// Manually register a plugin
    pub async fn register_plugin(&mut self, mut plugin: Box<dyn Plugin>) -> Result<()> {
        // Clone metadata values we need before calling on_load
        let plugin_name = plugin.metadata().display_name();
        let plugin_version = plugin.metadata().version.clone();
        let api_version = plugin.metadata().api_version.clone();
        let catchphrase = plugin.metadata().catchphrase.clone();

        // Check API compatibility
        if !plugin
            .metadata()
            .is_compatible(scarab_plugin_api::API_VERSION)
        {
            return Err(PluginError::VersionIncompatible {
                required: scarab_plugin_api::API_VERSION.to_string(),
                actual: api_version,
            });
        }

        log::info!("🎯 Registering plugin: {} v{}", plugin_name, plugin_version);
        if let Some(phrase) = &catchphrase {
            log::info!("   💬 \"{}\"", phrase);
        }

        // Call on_load with timeout
        let mut ctx = (*self.context).clone();
        let timeout_duration = self.hook_timeout;

        // Call on_load directly with timeout
        let load_result = timeout(timeout_duration, plugin.on_load(&mut ctx)).await;

        match load_result {
            Ok(Ok(_)) => {
                let config = PluginConfig {
                    name: plugin.metadata().name.clone(),
                    path: PathBuf::new(),
                    enabled: true,
                    config: Default::default(),
                };
                self.plugins.push(ManagedPlugin::new(plugin, config));
                self.total_loaded += 1;

                log::info!(
                    "✅ {} Plugin '{}' is ready!",
                    delight::random_success_message(),
                    plugin_name
                );

                // Refresh command list
                self.refresh_commands();

                // Check for achievements
                self.check_achievements();

                // Process commands that might have been queued during on_load
                self.process_pending_commands().await;

                Ok(())
            }
            Ok(Err(e)) => {
                log::error!("❌ Failed to initialize plugin '{}': {}", plugin_name, e);
                Err(e)
            }
            Err(_) => {
                let error = PluginError::Timeout(timeout_duration.as_millis() as u64);
                log::error!(
                    "⏱️  Plugin '{}' initialization timed out",
                    plugin_name
                );
                Err(error)
            }
        }
    }

    /// Dispatch output hook to all enabled plugins
    pub async fn dispatch_output(&mut self, line: &str) -> Result<String> {
        let mut data = line.to_string();

        for managed in &mut self.plugins {
            if !managed.enabled {
                continue;
            }

            let plugin_name = managed.plugin.metadata().display_name();
            let current_data = data.clone();
            let ctx = self.context.clone();

            // Apply timeout to plugin call
            let result = timeout(
                self.hook_timeout,
                managed.plugin.on_output(&current_data, &ctx),
            )
            .await;

            match result {
                Ok(Ok(Action::Continue)) => {
                    managed.record_success();
                }
                Ok(Ok(Action::Stop)) => {
                    managed.record_success();
                    break;
                }
                Ok(Ok(Action::Modify(new_data))) => {
                    managed.record_success();
                    data = String::from_utf8(new_data).unwrap_or(data);
                }
                Ok(Err(e)) => {
                    log::error!(
                        "{} Plugin '{}' output hook failed: {}",
                        managed.mood().emoji(),
                        plugin_name,
                        e
                    );
                    managed.record_failure();
                }
                Err(_) => {
                    log::error!(
                        "⏱️  Plugin '{}' output hook timed out",
                        plugin_name
                    );
                    managed.record_failure();
                }
            }
        }

        // Process pending commands from all plugins
        self.process_pending_commands().await;

        Ok(data)
    }

    /// Dispatch input hook to all enabled plugins
    pub async fn dispatch_input(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        let mut data = input.to_vec();

        for managed in &mut self.plugins {
            if !managed.enabled {
                continue;
            }

            let plugin_name = managed.plugin.metadata().display_name();
            let current_data = data.clone();
            let ctx = self.context.clone();

            let result = timeout(
                self.hook_timeout,
                managed.plugin.on_input(&current_data, &ctx),
            )
            .await;

            match result {
                Ok(Ok(Action::Continue)) => {
                    managed.record_success();
                }
                Ok(Ok(Action::Stop)) => {
                    managed.record_success();
                    break;
                }
                Ok(Ok(Action::Modify(new_data))) => {
                    managed.record_success();
                    data = new_data;
                }
                Ok(Err(e)) => {
                    log::error!(
                        "{} Plugin '{}' input hook failed: {}",
                        managed.mood().emoji(),
                        plugin_name,
                        e
                    );
                    managed.record_failure();
                }
                Err(_) => {
                    log::error!("⏱️  Plugin '{}' input hook timed out", plugin_name);
                    managed.record_failure();
                }
            }
        }

        // Process pending commands
        self.process_pending_commands().await;

        Ok(data)
    }

    /// Dispatch resize event to all enabled plugins
    pub async fn dispatch_resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        for managed in &mut self.plugins {
            if !managed.enabled {
                continue;
            }

            let plugin_name = managed.plugin.metadata().display_name();
            let ctx = self.context.clone();

            let result = timeout(
                self.hook_timeout,
                managed.plugin.on_resize(cols, rows, &ctx),
            )
            .await;

            match result {
                Ok(Ok(_)) => managed.record_success(),
                Ok(Err(e)) => {
                    log::error!(
                        "{} Plugin '{}' resize hook failed: {}",
                        managed.mood().emoji(),
                        plugin_name,
                        e
                    );
                    managed.record_failure();
                }
                Err(_) => {
                    log::error!("⏱️  Plugin '{}' resize hook timed out", plugin_name);
                    managed.record_failure();
                }
            }
        }

        // Process pending commands
        self.process_pending_commands().await;

        Ok(())
    }

    /// Get information about all loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.iter().map(|p| p.info()).collect()
    }

    /// Get count of enabled plugins
    pub fn enabled_count(&self) -> usize {
        self.plugins.iter().filter(|p| p.enabled).count()
    }

    /// Unload all plugins
    pub async fn unload_all(&mut self) -> Result<()> {
        log::info!("👋 Saying goodbye to {} plugins...", self.plugins.len());

        for managed in &mut self.plugins {
            let plugin_name = managed.plugin.metadata().display_name();

            let result = timeout(self.hook_timeout, managed.plugin.on_unload()).await;

            match result {
                Ok(Ok(_)) => {
                    log::debug!("✅ Plugin '{}' unloaded cleanly", plugin_name);
                }
                Ok(Err(e)) => {
                    log::error!("❌ Error unloading plugin '{}': {}", plugin_name, e);
                }
                Err(_) => {
                    log::error!("⏱️  Plugin '{}' unload timed out", plugin_name);
                }
            }
        }

        self.plugins.clear();
        self.refresh_commands();
        log::info!("✨ All plugins unloaded successfully!");
        Ok(())
    }

    /// Dispatch remote command to all enabled plugins
    pub async fn dispatch_remote_command(&mut self, id: &str) -> Result<()> {
        for managed in &mut self.plugins {
            if !managed.enabled {
                continue;
            }

            let plugin_name = managed.plugin.metadata().display_name();
            let ctx = self.context.clone();

            let result = timeout(
                self.hook_timeout,
                managed.plugin.on_remote_command(id, &ctx),
            )
            .await;

            match result {
                Ok(Ok(_)) => managed.record_success(),
                Ok(Err(e)) => {
                    log::error!(
                        "{} Plugin '{}' remote command hook failed: {}",
                        managed.mood().emoji(),
                        plugin_name,
                        e
                    );
                    managed.record_failure();
                }
                Err(_) => {
                    log::error!(
                        "⏱️  Plugin '{}' remote command hook timed out",
                        plugin_name
                    );
                    managed.record_failure();
                }
            }
        }

        // Process pending commands (e.g. if command triggers another UI update)
        self.process_pending_commands().await;

        Ok(())
    }
}
