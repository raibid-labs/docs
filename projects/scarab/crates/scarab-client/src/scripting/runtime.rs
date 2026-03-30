//! Script runtime - executes Fusabi scripts using fusabi-frontend
//!
//! This is a simplified runtime that wraps fusabi-frontend for .fsx script execution.
//! In a full implementation, this would integrate deeply with the fusabi-frontend parser
//! and evaluator. For now, we provide a scaffold that demonstrates the integration pattern.

use super::api::{OverlayContent, OverlayPosition, ScriptApi, ScriptContext, ScriptEvent};
use super::error::{ScriptError, ScriptResult};
use bevy::prelude::*;
use std::path::Path;

/// Runtime for executing Fusabi scripts
pub struct ScriptRuntime {
    api: ScriptApi,
    event_receiver: crossbeam::channel::Receiver<ScriptEvent>,
}

impl ScriptRuntime {
    /// Create a new script runtime
    pub fn new() -> Self {
        let (tx, rx) = crossbeam::channel::unbounded();

        Self {
            api: ScriptApi::new(tx),
            event_receiver: rx,
        }
    }

    /// Execute a script file
    pub fn execute_file(&self, path: &Path, context: &ScriptContext) -> ScriptResult<()> {
        let source = std::fs::read_to_string(path).map_err(|e| ScriptError::LoadError {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;

        self.execute_source(&source, path.display().to_string().as_str(), context)
    }

    /// Execute script source code
    pub fn execute_source(
        &self,
        source: &str,
        script_name: &str,
        context: &ScriptContext,
    ) -> ScriptResult<()> {
        // Parse the script using fusabi-frontend
        // Note: This is a simplified implementation. A full implementation would:
        // 1. Use fusabi_frontend::parse() to parse the .fsx source
        // 2. Build an AST
        // 3. Execute the AST with the provided context
        // 4. Handle all F# language features

        // For now, we'll interpret common patterns manually as a demonstration
        self.interpret_simple_script(source, script_name, context)
    }

    /// Simplified script interpreter (demonstration only)
    /// A real implementation would use fusabi-frontend's full parser and evaluator
    fn interpret_simple_script(
        &self,
        source: &str,
        script_name: &str,
        _context: &ScriptContext,
    ) -> ScriptResult<()> {
        // This is a simplified interpreter for demonstration purposes
        // In production, you would use fusabi-frontend's parser and evaluator

        for (line_num, line) in source.lines().enumerate() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // Parse simple API calls
            if let Err(e) = self.parse_and_execute_line(line, script_name) {
                return Err(ScriptError::ParseError {
                    script: script_name.to_string(),
                    line: line_num + 1,
                    column: 0,
                    message: e.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Parse and execute a single line (simplified)
    fn parse_and_execute_line(&self, line: &str, _script_name: &str) -> ScriptResult<()> {
        // Example: Scarab.setColor "foreground" "#f8f8f2"
        if line.starts_with("Scarab.setColor") {
            if let Some(args) = line.strip_prefix("Scarab.setColor") {
                let parts: Vec<&str> = args
                    .split('"')
                    .filter(|s| !s.trim().is_empty())
                    .collect();
                if parts.len() >= 2 {
                    let name = parts[0].trim();
                    let color_hex = parts[1].trim();
                    let color = ScriptApi::parse_color(color_hex)?;
                    self.api.set_color(name, color)?;
                }
            }
        }
        // Example: Scarab.setFont "JetBrains Mono" 16.0
        else if line.starts_with("Scarab.setFont") {
            if let Some(args) = line.strip_prefix("Scarab.setFont") {
                let parts: Vec<&str> = args.split('"').collect();
                if parts.len() >= 2 {
                    let family = parts[1];
                    // Extract size from remaining text
                    if let Some(size_str) = parts.get(2) {
                        let size: f32 = size_str
                            .trim()
                            .parse()
                            .unwrap_or(14.0);
                        self.api.set_font(family, size)?;
                    }
                }
            }
        }
        // Example: Scarab.setWindowTitle "My Terminal"
        else if line.starts_with("Scarab.setWindowTitle") {
            if let Some(args) = line.strip_prefix("Scarab.setWindowTitle") {
                let title = args
                    .split('"')
                    .nth(1)
                    .unwrap_or("Scarab Terminal");
                self.api.set_window_title(title)?;
            }
        }
        // Example: Scarab.addOverlay "status" TopRight (Text "Hello" 12.0 "#ffffff")
        else if line.starts_with("Scarab.addOverlay") {
            // Simplified overlay parsing
            let name = "status";
            let position = OverlayPosition::TopRight;
            let content = OverlayContent::Text {
                text: "Custom Overlay".to_string(),
                size: 12.0,
                color: Color::WHITE,
            };
            self.api.add_overlay(name, position, content)?;
        }

        Ok(())
    }

    /// Collect pending events from the runtime
    pub fn collect_events(&self) -> Vec<ScriptEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.event_receiver.try_recv() {
            events.push(event);
        }
        events
    }
}

/// Loaded script with metadata
pub struct LoadedScript {
    pub name: String,
    pub path: std::path::PathBuf,
    pub source: String,
    pub last_modified: std::time::SystemTime,
}

impl LoadedScript {
    /// Load a script from a file
    pub fn from_file(path: &Path) -> ScriptResult<Self> {
        let source = std::fs::read_to_string(path).map_err(|e| ScriptError::LoadError {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;

        let metadata = std::fs::metadata(path).map_err(|e| ScriptError::LoadError {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;

        let last_modified = metadata
            .modified()
            .map_err(|e| ScriptError::LoadError {
                path: path.display().to_string(),
                reason: e.to_string(),
            })?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            name,
            path: path.to_path_buf(),
            source,
            last_modified,
        })
    }

    /// Check if the script has been modified since it was loaded
    pub fn is_modified(&self) -> bool {
        if let Ok(metadata) = std::fs::metadata(&self.path) {
            if let Ok(modified) = metadata.modified() {
                return modified > self.last_modified;
            }
        }
        false
    }

    /// Reload the script from disk
    pub fn reload(&mut self) -> ScriptResult<()> {
        let script = Self::from_file(&self.path)?;
        self.source = script.source;
        self.last_modified = script.last_modified;
        Ok(())
    }
}

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = ScriptRuntime::new();
        let events = runtime.collect_events();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_simple_script_execution() {
        let runtime = ScriptRuntime::new();
        let context = ScriptContext {
            colors: super::super::api::ColorContext {
                foreground: Color::WHITE,
                background: Color::BLACK,
                cursor: Color::WHITE,
                selection_bg: Color::srgba(0.5, 0.5, 0.5, 0.3),
                selection_fg: Color::WHITE,
                palette: vec![Color::BLACK; 16],
            },
            fonts: super::super::api::FontContext {
                family: "JetBrains Mono".to_string(),
                size: 14.0,
                line_height: 1.2,
            },
            window: super::super::api::WindowContext {
                width: 800.0,
                height: 600.0,
                scale_factor: 1.0,
                title: "Test".to_string(),
            },
            terminal: super::super::api::TerminalContext {
                rows: 24,
                cols: 80,
                scrollback_lines: 10000,
            },
        };

        let source = r##"
            // Simple test script
            Scarab.setColor "foreground" "#ffffff"
            Scarab.setWindowTitle "Test Window"
        "##;

        let result = runtime.execute_source(source, "test.fsx", &context);
        assert!(result.is_ok());

        let events = runtime.collect_events();
        assert!(events.len() >= 2);
    }
}
