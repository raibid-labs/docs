//! File watcher for hot-reloading scripts
//!
//! Monitors the scripts directory for changes and triggers reloads

use bevy::prelude::*;
use super::error::{ScriptError, ScriptResult};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Watches script files for changes
pub struct ScriptWatcher {
    watched_files: Vec<WatchedFile>,
    check_interval: Duration,
    last_check: SystemTime,
}

struct WatchedFile {
    path: PathBuf,
    last_modified: SystemTime,
}

impl ScriptWatcher {
    /// Create a new script watcher
    pub fn new() -> Self {
        Self {
            watched_files: Vec::new(),
            check_interval: Duration::from_millis(500), // Check every 500ms
            last_check: SystemTime::now(),
        }
    }

    /// Add a file to watch
    pub fn watch(&mut self, path: PathBuf) -> ScriptResult<()> {
        let metadata = std::fs::metadata(&path).map_err(|e| ScriptError::WatcherError(
            format!("Failed to read metadata for '{}': {}", path.display(), e),
        ))?;

        let last_modified = metadata.modified().map_err(|e| {
            ScriptError::WatcherError(format!(
                "Failed to get modification time for '{}': {}",
                path.display(),
                e
            ))
        })?;

        // Don't add duplicates
        if !self.watched_files.iter().any(|f| f.path == path) {
            self.watched_files.push(WatchedFile {
                path: path.clone(),
                last_modified,
            });
            debug!("Watching script: {}", path.display());
        }

        Ok(())
    }

    /// Remove a file from watching
    pub fn unwatch(&mut self, path: &Path) {
        self.watched_files.retain(|f| f.path != path);
    }

    /// Check for changes in watched files
    /// Returns a list of paths that have been modified
    pub fn check_changes(&mut self) -> ScriptResult<Vec<PathBuf>> {
        let now = SystemTime::now();

        // Rate limit checks
        if now.duration_since(self.last_check).unwrap_or(Duration::ZERO) < self.check_interval {
            return Ok(Vec::new());
        }

        self.last_check = now;

        let mut changed = Vec::new();

        for watched in &mut self.watched_files {
            match std::fs::metadata(&watched.path) {
                Ok(metadata) => {
                    if let Ok(modified) = metadata.modified() {
                        if modified > watched.last_modified {
                            info!("Script modified: {}", watched.path.display());
                            watched.last_modified = modified;
                            changed.push(watched.path.clone());
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to check file '{}': {}", watched.path.display(), e);
                }
            }
        }

        Ok(changed)
    }

    /// Get all watched file paths
    pub fn watched_paths(&self) -> Vec<&Path> {
        self.watched_files.iter().map(|f| f.path.as_path()).collect()
    }

    /// Clear all watched files
    pub fn clear(&mut self) {
        self.watched_files.clear();
    }

    /// Set the check interval
    pub fn set_check_interval(&mut self, interval: Duration) {
        self.check_interval = interval;
    }
}

impl Default for ScriptWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use tempfile::TempDir;

    #[test]
    fn test_watcher_creation() {
        let watcher = ScriptWatcher::new();
        assert_eq!(watcher.watched_paths().len(), 0);
    }

    #[test]
    fn test_watch_file() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test.fsx");
        fs::write(&script_path, "// Test").unwrap();

        let mut watcher = ScriptWatcher::new();
        watcher.watch(script_path.clone()).unwrap();

        assert_eq!(watcher.watched_paths().len(), 1);
        assert_eq!(watcher.watched_paths()[0], script_path);
    }

    #[test]
    fn test_detect_changes() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test.fsx");
        fs::write(&script_path, "// Test").unwrap();

        let mut watcher = ScriptWatcher::new();
        watcher.watch(script_path.clone()).unwrap();

        // No changes initially
        let changed = watcher.check_changes().unwrap();
        assert_eq!(changed.len(), 0);

        // Modify the file
        thread::sleep(Duration::from_millis(100));
        fs::write(&script_path, "// Modified").unwrap();

        // Should detect the change
        thread::sleep(Duration::from_millis(600)); // Wait for check interval
        let changed = watcher.check_changes().unwrap();
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0], script_path);
    }

    #[test]
    fn test_unwatch() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test.fsx");
        fs::write(&script_path, "// Test").unwrap();

        let mut watcher = ScriptWatcher::new();
        watcher.watch(script_path.clone()).unwrap();
        assert_eq!(watcher.watched_paths().len(), 1);

        watcher.unwatch(&script_path);
        assert_eq!(watcher.watched_paths().len(), 0);
    }
}
