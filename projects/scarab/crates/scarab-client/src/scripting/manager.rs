//! Script manager - coordinates loading, execution, and hot-reloading
//!
//! This is the central coordinator for the scripting system

use super::api::ScriptEvent;
use super::context::RuntimeContext;
use super::error::{ScriptError, ScriptResult};
use super::loader::ScriptLoader;
use super::runtime::{LoadedScript, ScriptRuntime};
use super::watcher::ScriptWatcher;
use bevy::prelude::*;
use std::collections::HashMap;
use std::path::Path;

/// Manages all scripts and their lifecycle
#[derive(Resource)]
pub struct ScriptManager {
    loader: ScriptLoader,
    runtime: ScriptRuntime,
    watcher: ScriptWatcher,
    scripts: HashMap<String, LoadedScript>,
    initialized: bool,
}

impl ScriptManager {
    /// Create a new script manager
    pub fn new(scripts_directory: std::path::PathBuf) -> Self {
        Self {
            loader: ScriptLoader::new(scripts_directory),
            runtime: ScriptRuntime::new(),
            watcher: ScriptWatcher::new(),
            scripts: HashMap::new(),
            initialized: false,
        }
    }

    /// Initialize the script manager - load all scripts
    pub fn initialize(&mut self, scripts_directory: &Path) -> ScriptResult<usize> {
        info!("Initializing script manager from: {}", scripts_directory.display());

        // Ensure directory exists
        self.loader.ensure_directory()?;

        // Load all scripts
        let loaded_scripts = self.loader.load_all_scripts()?;
        let count = loaded_scripts.len();

        // Register scripts for watching
        for script in loaded_scripts {
            self.watcher.watch(script.path.clone())?;
            self.scripts.insert(script.name.clone(), script);
        }

        self.initialized = true;
        info!("Loaded {} scripts", count);

        Ok(count)
    }

    /// Check for script reloads
    pub fn check_reloads(&mut self) -> ScriptResult<()> {
        if !self.initialized {
            return Ok(());
        }

        let changed_paths = self.watcher.check_changes()?;

        for path in changed_paths {
            // Find the script by path
            let script_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            if let Some(script) = self.scripts.get_mut(script_name) {
                info!("Reloading script: {}", script_name);
                match script.reload() {
                    Ok(_) => {
                        info!("Successfully reloaded: {}", script_name);
                    }
                    Err(e) => {
                        error!("Failed to reload '{}': {}", script_name, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute all loaded scripts
    pub fn execute_all(&self, context: &RuntimeContext) -> Vec<ScriptResult<()>> {
        let mut results = Vec::new();

        for (name, script) in &self.scripts {
            debug!("Executing script: {}", name);  // Changed from info! to debug!
            let result = self
                .runtime
                .execute_source(&script.source, name, context.context());
            results.push(result);
        }

        results
    }

    /// Execute a specific script by name
    pub fn execute_script(&self, name: &str, context: &RuntimeContext) -> ScriptResult<()> {
        let script = self.scripts.get(name).ok_or_else(|| {
            ScriptError::ResourceNotFound {
                resource_type: "script".to_string(),
                name: name.to_string(),
            }
        })?;

        self.runtime
            .execute_source(&script.source, name, context.context())
    }

    /// Execute pending scripts (called from Bevy system)
    pub fn execute_pending(&mut self, context: &RuntimeContext, _commands: &mut Commands) {
        // This method is intentionally left empty now
        // Scripts should be executed on-demand or on specific events, not every frame
        // TODO: Implement event-based script execution
        let _ = (context, _commands);
    }

    /// Handle a script event
    pub fn handle_event(&mut self, event: &ScriptEvent) -> ScriptResult<()> {
        // Events are collected from the runtime and processed by Bevy systems
        debug!("Handling script event: {:?}", event);
        Ok(())
    }

    /// Collect events from the runtime
    pub fn collect_events(&self) -> Vec<ScriptEvent> {
        self.runtime.collect_events()
    }

    /// Get the number of loaded scripts
    pub fn script_count(&self) -> usize {
        self.scripts.len()
    }

    /// Get a list of all script names
    pub fn script_names(&self) -> Vec<String> {
        self.scripts.keys().cloned().collect()
    }

    /// Check if a script is loaded
    pub fn has_script(&self, name: &str) -> bool {
        self.scripts.contains_key(name)
    }
}

impl Default for ScriptManager {
    fn default() -> Self {
        // Default to ~/.config/scarab/scripts
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let scripts_dir = std::path::PathBuf::from(home)
            .join(".config")
            .join("scarab")
            .join("scripts");

        Self::new(scripts_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ScriptManager::new(temp_dir.path().to_path_buf());
        assert_eq!(manager.script_count(), 0);
        assert!(!manager.initialized);
    }

    #[test]
    fn test_initialize() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ScriptManager::new(temp_dir.path().to_path_buf());

        // Create some test scripts
        fs::write(temp_dir.path().join("script1.fsx"), "// Script 1").unwrap();
        fs::write(temp_dir.path().join("script2.fsx"), "// Script 2").unwrap();

        let count = manager.initialize(temp_dir.path()).unwrap();
        assert_eq!(count, 2);
        assert_eq!(manager.script_count(), 2);
        assert!(manager.initialized);
    }

    #[test]
    fn test_script_names() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ScriptManager::new(temp_dir.path().to_path_buf());

        fs::write(temp_dir.path().join("foo.fsx"), "// Foo").unwrap();
        fs::write(temp_dir.path().join("bar.fsx"), "// Bar").unwrap();

        manager.initialize(temp_dir.path()).unwrap();

        let names = manager.script_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"foo".to_string()));
        assert!(names.contains(&"bar".to_string()));
    }

    #[test]
    fn test_has_script() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ScriptManager::new(temp_dir.path().to_path_buf());

        fs::write(temp_dir.path().join("test.fsx"), "// Test").unwrap();
        manager.initialize(temp_dir.path()).unwrap();

        assert!(manager.has_script("test"));
        assert!(!manager.has_script("nonexistent"));
    }
}
