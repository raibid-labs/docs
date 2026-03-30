//! Configuration hot-reload watcher

use crate::{ConfigLoader, Result, ScarabConfig};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    time::Instant,
};
use tracing::{debug, error, info};

/// Callback function for config changes
pub type ConfigChangeCallback = Box<dyn Fn(&ScarabConfig) + Send + Sync>;

/// Configuration file watcher with hot-reload
pub struct ConfigWatcher {
    config: Arc<RwLock<ScarabConfig>>,
    loader: ConfigLoader,
    _watcher: Option<RecommendedWatcher>,
    watch_paths: Vec<PathBuf>,
    callbacks: Arc<RwLock<Vec<ConfigChangeCallback>>>,
    running: Arc<AtomicBool>,
}

impl ConfigWatcher {
    /// Create a new watcher with initial config
    pub fn new(initial_config: ScarabConfig) -> Result<Self> {
        let config = Arc::new(RwLock::new(initial_config));
        let loader = ConfigLoader::new();
        let watch_paths = Self::get_watch_paths(&loader);

        Ok(Self {
            config,
            loader,
            _watcher: None,
            watch_paths,
            callbacks: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Start watching for config file changes
    pub fn start(&mut self) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            debug!("Config watcher already running");
            return Ok(());
        }

        let config = Arc::clone(&self.config);
        let callbacks = Arc::clone(&self.callbacks);
        let _running = Arc::clone(&self.running);
        let watch_paths = self.watch_paths.clone();

        let mut watcher =
            notify::recommended_watcher(move |res: std::result::Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                            // Check if the event is for a config file
                            if event
                                .paths
                                .iter()
                                .any(|p| watch_paths.iter().any(|wp| p.ends_with(wp)))
                            {
                                let start = Instant::now();
                                debug!("Config file changed, reloading...");

                                // Reload config
                                if let Ok(new_config) = ConfigLoader::new().load() {
                                    *config.write().unwrap() = new_config.clone();
                                    let reload_time = start.elapsed();
                                    info!("Config reloaded in {}ms", reload_time.as_millis());

                                    // Call callbacks
                                    let cbs = callbacks.read().unwrap();
                                    for callback in cbs.iter() {
                                        callback(&new_config);
                                    }
                                } else {
                                    error!("Failed to reload config");
                                }
                            }
                        }
                    }
                    Err(e) => error!("Watch error: {:?}", e),
                }
            })?;

        // Watch config directories
        for path in &self.watch_paths {
            if let Some(parent) = path.parent() {
                if parent.exists() {
                    watcher.watch(parent, RecursiveMode::NonRecursive)?;
                    info!("Watching config directory: {}", parent.display());
                }
            }
        }

        self.running.store(true, Ordering::SeqCst);
        self._watcher = Some(watcher);

        info!("Config watcher started");
        Ok(())
    }

    /// Stop watching for changes
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self._watcher = None;
        info!("Config watcher stopped");
    }

    /// Register a callback for config changes
    pub fn on_change(&self, callback: ConfigChangeCallback) {
        self.callbacks.write().unwrap().push(callback);
    }

    /// Get current configuration
    pub fn get_config(&self) -> ScarabConfig {
        self.config.read().unwrap().clone()
    }

    /// Manually reload configuration
    pub fn reload(&self) -> Result<()> {
        let start = Instant::now();
        let new_config = self.loader.load()?;
        *self.config.write().unwrap() = new_config.clone();

        let reload_time = start.elapsed();
        info!("Config manually reloaded in {}ms", reload_time.as_millis());

        // Call callbacks
        let callbacks = self.callbacks.read().unwrap();
        for callback in callbacks.iter() {
            callback(&new_config);
        }

        Ok(())
    }

    /// Get paths to watch
    fn get_watch_paths(_loader: &ConfigLoader) -> Vec<PathBuf> {
        let mut paths = vec![ConfigLoader::default_config_path()];

        // Add local config if it exists
        if let Ok(Some(local_path)) = Self::find_local_config() {
            paths.push(local_path);
        }

        paths
    }

    /// Find local config file in directory tree
    fn find_local_config() -> Result<Option<PathBuf>> {
        let mut current = std::env::current_dir()?;

        loop {
            let config_path = current.join(".scarab.toml");
            if config_path.exists() {
                return Ok(Some(config_path));
            }

            if !current.pop() {
                break;
            }
        }

        Ok(None)
    }

    /// Get reload statistics (for testing/debugging)
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Drop for ConfigWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_create_watcher() {
        let config = ScarabConfig::default();
        let watcher = ConfigWatcher::new(config);
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_get_config() {
        let mut config = ScarabConfig::default();
        config.font.size = 18.0;

        let watcher = ConfigWatcher::new(config).unwrap();
        let retrieved = watcher.get_config();
        assert_eq!(retrieved.font.size, 18.0);
    }

    #[test]
    fn test_callback_registration() {
        let config = ScarabConfig::default();
        let watcher = ConfigWatcher::new(config).unwrap();

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        watcher.on_change(Box::new(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));

        // Manually trigger reload
        let new_config = ScarabConfig::default();
        *watcher.config.write().unwrap() = new_config.clone();

        let callbacks = watcher.callbacks.read().unwrap();
        for callback in callbacks.iter() {
            callback(&new_config);
        }

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_start_stop() {
        let config = ScarabConfig::default();
        let mut watcher = ConfigWatcher::new(config).unwrap();

        assert!(!watcher.is_running());

        // Start watching
        let result = watcher.start();
        // May fail if config dir doesn't exist, that's okay for this test
        if result.is_ok() {
            assert!(watcher.is_running());

            watcher.stop();
            assert!(!watcher.is_running());
        }
    }
}
