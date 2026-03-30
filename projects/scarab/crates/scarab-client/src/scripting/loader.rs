//! Script loader - discovers and loads .fsx scripts from disk
use bevy::prelude::*;

use super::error::{ScriptError, ScriptResult};
use super::runtime::LoadedScript;
use std::path::{Path, PathBuf};

/// Discovers and loads scripts from the filesystem
pub struct ScriptLoader {
    scripts_directory: PathBuf,
}

impl ScriptLoader {
    /// Create a new script loader for the given directory
    pub fn new(scripts_directory: PathBuf) -> Self {
        Self { scripts_directory }
    }

    /// Ensure the scripts directory exists
    pub fn ensure_directory(&self) -> ScriptResult<()> {
        if !self.scripts_directory.exists() {
            std::fs::create_dir_all(&self.scripts_directory).map_err(|e| {
                ScriptError::IoError(format!(
                    "Failed to create scripts directory '{}': {}",
                    self.scripts_directory.display(),
                    e
                ))
            })?;
            info!("Created scripts directory: {}", self.scripts_directory.display());
        }
        Ok(())
    }

    /// Discover all .fsx scripts in the directory
    pub fn discover_scripts(&self) -> ScriptResult<Vec<PathBuf>> {
        if !self.scripts_directory.exists() {
            return Ok(Vec::new());
        }

        let mut scripts = Vec::new();

        let entries = std::fs::read_dir(&self.scripts_directory).map_err(|e| {
            ScriptError::IoError(format!(
                "Failed to read scripts directory '{}': {}",
                self.scripts_directory.display(),
                e
            ))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                ScriptError::IoError(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();

            // Only include .fsx files
            if path.extension().and_then(|s| s.to_str()) == Some("fsx") {
                scripts.push(path);
            }
        }

        // Sort for consistent ordering
        scripts.sort();

        Ok(scripts)
    }

    /// Load a single script file
    pub fn load_script(&self, path: &Path) -> ScriptResult<LoadedScript> {
        LoadedScript::from_file(path)
    }

    /// Load all scripts in the directory
    pub fn load_all_scripts(&self) -> ScriptResult<Vec<LoadedScript>> {
        let paths = self.discover_scripts()?;
        let mut loaded = Vec::new();

        for path in paths {
            match self.load_script(&path) {
                Ok(script) => {
                    info!("Loaded script: {}", script.name);
                    loaded.push(script);
                }
                Err(e) => {
                    error!("Failed to load script '{}': {}", path.display(), e);
                }
            }
        }

        Ok(loaded)
    }

    /// Get the scripts directory path
    pub fn scripts_directory(&self) -> &Path {
        &self.scripts_directory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_loader_creation() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ScriptLoader::new(temp_dir.path().to_path_buf());
        assert_eq!(loader.scripts_directory(), temp_dir.path());
    }

    #[test]
    fn test_ensure_directory() {
        let temp_dir = TempDir::new().unwrap();
        let scripts_path = temp_dir.path().join("scripts");
        let loader = ScriptLoader::new(scripts_path.clone());

        assert!(!scripts_path.exists());
        loader.ensure_directory().unwrap();
        assert!(scripts_path.exists());
    }

    #[test]
    fn test_discover_scripts() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ScriptLoader::new(temp_dir.path().to_path_buf());

        // Create some test files
        fs::write(temp_dir.path().join("script1.fsx"), "// Script 1").unwrap();
        fs::write(temp_dir.path().join("script2.fsx"), "// Script 2").unwrap();
        fs::write(temp_dir.path().join("not_a_script.txt"), "text").unwrap();

        let scripts = loader.discover_scripts().unwrap();
        assert_eq!(scripts.len(), 2);

        // Check that only .fsx files were discovered
        for script in &scripts {
            assert_eq!(script.extension().unwrap(), "fsx");
        }
    }

    #[test]
    fn test_load_script() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test.fsx");
        let content = "// Test script\nScarab.setWindowTitle \"Test\"";
        fs::write(&script_path, content).unwrap();

        let loader = ScriptLoader::new(temp_dir.path().to_path_buf());
        let script = loader.load_script(&script_path).unwrap();

        assert_eq!(script.name, "test");
        assert_eq!(script.source, content);
    }
}
