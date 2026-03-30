//! Fusabi plugin adapter for bridging Fusabi runtime to Plugin trait
//!
//! This module provides adapters for both .fzb (compiled bytecode) and .fsx (scripts)
//! to implement the scarab-plugin-api Plugin trait.

use async_trait::async_trait;
use scarab_plugin_api::{Action, Plugin, PluginContext, PluginError, PluginMetadata, Result};
use std::cell::RefCell;
use std::path::Path;

// Import Fusabi VM (official runtime from crates.io)
use fusabi_vm::{Value, Vm};

// Import Fusabi Frontend (F# script parser/compiler)
use fusabi_frontend::{Compiler, Lexer, Parser};

/// Thread-local storage for VM instances to work around !Send constraints
/// The Fusabi VM uses Rc which is !Send, so we use thread-local storage
thread_local! {
    static VM_CACHE: RefCell<Option<Vm>> = RefCell::new(None);
}

/// Adapter for compiled Fusabi bytecode (.fzb files)
pub struct FusabiBytecodePlugin {
    metadata: PluginMetadata,
    bytecode: Vec<u8>,
    // Note: Vm contains Rc which is !Send, so we can't store it directly
    // We use thread-local VM_CACHE for VM instances
}

impl std::fmt::Debug for FusabiBytecodePlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FusabiBytecodePlugin")
            .field("metadata", &self.metadata)
            .field("bytecode_len", &self.bytecode.len())
            .finish()
    }
}

impl FusabiBytecodePlugin {
    /// Load a .fzb bytecode file
    pub fn load(path: &Path) -> Result<Self> {
        // Read bytecode file
        let bytecode = std::fs::read(path)
            .map_err(|e| PluginError::LoadError(format!("Failed to read bytecode file: {}", e)))?;

        // Validate we can deserialize the bytecode
        let _ = fusabi_vm::deserialize_chunk(&bytecode)
            .map_err(|e| PluginError::LoadError(format!("Invalid Fusabi bytecode: {}", e)))?;

        // Extract metadata from bytecode
        let plugin_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let metadata =
            PluginMetadata::new(plugin_name, "0.1.0", "Fusabi bytecode plugin", "Fusabi VM");

        Ok(Self { metadata, bytecode })
    }

    /// Call a hook function in the bytecode VM
    ///
    /// This function mirrors the implementation of FusabiScriptPlugin::call_hook_function()
    /// but works with pre-compiled bytecode instead of parsing/compiling F# source.
    ///
    /// Returns Ok(None) if the function doesn't exist (not an error)
    /// Returns Ok(Some(value)) if the function exists and was called successfully
    fn call_hook_function(
        &self,
        function_name: &str,
        args: &[Value],
        _ctx: &PluginContext,
    ) -> Result<Option<Value>> {
        // Use thread-local VM to avoid Send issues
        VM_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();

            // Create a new VM if not cached
            if cache.is_none() {
                *cache = Some(Vm::new());
            }

            let vm = cache.as_mut().unwrap();

            // Deserialize and execute the main bytecode chunk to populate globals
            let chunk = fusabi_vm::deserialize_chunk(&self.bytecode)
                .map_err(|e| PluginError::Other(anyhow::anyhow!("Deserialization failed: {}", e)))?;

            vm.execute(chunk).map_err(|e| {
                PluginError::Other(anyhow::anyhow!("Bytecode execution failed: {}", e))
            })?;

            // Check if the hook function exists in globals
            if !vm.globals.contains_key(function_name) {
                log::trace!(
                    "Hook function '{}' not defined in bytecode plugin '{}'",
                    function_name,
                    self.metadata.name
                );
                return Ok(None);
            }

            // Get the function value from globals
            let func_value = vm.globals.get(function_name)
                .ok_or_else(|| PluginError::Other(anyhow::anyhow!("Function '{}' not found in globals", function_name)))?
                .clone();

            // Call the function with provided arguments
            let result = vm.call_value(func_value, args).map_err(|e| {
                PluginError::Other(anyhow::anyhow!("Hook execution failed: {}", e))
            })?;

            Ok(Some(result))
        })
    }
}

#[async_trait]
impl Plugin for FusabiBytecodePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()> {
        ctx.log(
            scarab_plugin_api::context::LogLevel::Info,
            &format!("Loading Fusabi bytecode plugin: {}", self.metadata.name),
        );

        // Execute the plugin bytecode and call on_load hook if defined
        // Expected signature: let on_load = fun () -> ()
        match self.call_hook_function("on_load", &[Value::Unit], ctx) {
            Ok(Some(result)) => {
                log::debug!(
                    "Bytecode plugin '{}' on_load returned: {:?}",
                    self.metadata.name,
                    result
                );
            }
            Ok(None) => {
                log::trace!("Bytecode plugin '{}' has no on_load hook", self.metadata.name);
            }
            Err(e) => {
                log::warn!(
                    "Bytecode plugin '{}' on_load hook failed: {}",
                    self.metadata.name,
                    e
                );
                return Err(e);
            }
        }

        log::debug!("Plugin bytecode loaded, {} bytes", self.bytecode.len());
        Ok(())
    }

    async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action> {
        // Call the on_output hook if defined
        // Expected signature: let on_output = fun line -> bool (true = continue, false = suppress)
        let args = vec![Value::Str(line.to_string())];

        match self.call_hook_function("on_output", &args, ctx) {
            Ok(Some(result)) => {
                log::trace!(
                    "Bytecode plugin '{}' on_output returned: {:?}",
                    self.metadata.name,
                    result
                );

                // Check if result indicates we should suppress output
                match result {
                    Value::Bool(false) => Ok(Action::Stop),
                    _ => Ok(Action::Continue),
                }
            }
            Ok(None) => {
                log::trace!("Bytecode plugin '{}' processing output", self.metadata.name);
                Ok(Action::Continue)
            }
            Err(e) => {
                log::warn!(
                    "Bytecode plugin '{}' on_output hook failed: {}",
                    self.metadata.name,
                    e
                );
                Ok(Action::Continue)
            }
        }
    }

    async fn on_input(&mut self, input: &[u8], ctx: &PluginContext) -> Result<Action> {
        // Call the on_input hook if defined
        // Expected signature: let on_input = fun bytes_str -> bool
        let input_str = String::from_utf8_lossy(input);
        let args = vec![Value::Str(input_str.to_string())];

        match self.call_hook_function("on_input", &args, ctx) {
            Ok(Some(result)) => {
                log::trace!(
                    "Bytecode plugin '{}' on_input returned: {:?}",
                    self.metadata.name,
                    result
                );

                match result {
                    Value::Bool(false) => Ok(Action::Stop),
                    _ => Ok(Action::Continue),
                }
            }
            Ok(None) => {
                log::trace!(
                    "Bytecode plugin '{}' processing input ({} bytes)",
                    self.metadata.name,
                    input.len()
                );
                Ok(Action::Continue)
            }
            Err(e) => {
                log::warn!(
                    "Bytecode plugin '{}' on_input hook failed: {}",
                    self.metadata.name,
                    e
                );
                Ok(Action::Continue)
            }
        }
    }

    async fn on_resize(&mut self, cols: u16, rows: u16, ctx: &PluginContext) -> Result<()> {
        // Call the on_resize hook if defined
        // Expected signature: let on_resize = fun cols rows -> ()
        let args = vec![Value::Int(cols as i64), Value::Int(rows as i64)];

        match self.call_hook_function("on_resize", &args, ctx) {
            Ok(Some(result)) => {
                log::trace!(
                    "Bytecode plugin '{}' on_resize returned: {:?}",
                    self.metadata.name,
                    result
                );
            }
            Ok(None) => {
                log::trace!(
                    "Bytecode plugin '{}' handling resize: {}x{}",
                    self.metadata.name,
                    cols,
                    rows
                );
            }
            Err(e) => {
                log::warn!(
                    "Bytecode plugin '{}' on_resize hook failed: {}",
                    self.metadata.name,
                    e
                );
            }
        }

        Ok(())
    }

    async fn on_unload(&mut self) -> Result<()> {
        log::info!("Unloading Fusabi bytecode plugin: {}", self.metadata.name);

        // Clear VM cache to release resources
        VM_CACHE.with(|cache| {
            *cache.borrow_mut() = None;
        });

        // Note: We can't call the on_unload hook here because we don't have access to ctx
        // The Plugin trait's on_unload doesn't provide a context parameter
        // This matches the behavior of FusabiScriptPlugin

        Ok(())
    }
}

/// Adapter for Fusabi script files (.fsx files)
///
/// This adapter parses and compiles F# scripts using fusabi-frontend,
/// then executes them with the fusabi-vm. It supports hot-reloading
/// and calling hook functions defined in the script.
#[derive(Debug)]
pub struct FusabiScriptPlugin {
    metadata: PluginMetadata,
    script_source: String,
    script_path: std::path::PathBuf,
    /// Compiled bytecode (serialized, Send-safe)
    bytecode: Option<Vec<u8>>,
}

impl FusabiScriptPlugin {
    /// Load a .fsx script file
    pub fn load(path: &Path) -> Result<Self> {
        // Read script file
        let script_source = std::fs::read_to_string(path)
            .map_err(|e| PluginError::LoadError(format!("Failed to read script file: {}", e)))?;

        // Parse and compile the script
        let bytecode = Self::compile_script(&script_source)?;

        // Extract metadata from script comments
        let metadata = Self::extract_metadata(path, &script_source);

        Ok(Self {
            metadata,
            script_source,
            script_path: path.to_path_buf(),
            bytecode: Some(bytecode),
        })
    }

    /// Parse and compile F# script source to bytecode
    fn compile_script(source: &str) -> Result<Vec<u8>> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer
            .tokenize()
            .map_err(|e| PluginError::LoadError(format!("Lexer error: {}", e)))?;

        let mut parser = Parser::new(tokens);
        let program = parser
            .parse_program()
            .map_err(|e| PluginError::LoadError(format!("Parse error: {}", e)))?;

        let chunk = Compiler::compile_program(&program)
            .map_err(|e| PluginError::LoadError(format!("Compile error: {}", e)))?;

        // Serialize to bytes for Send-safe storage
        let bytecode = fusabi_vm::serialize_chunk(&chunk)
            .map_err(|e| PluginError::LoadError(format!("Serialization error: {}", e)))?;

        Ok(bytecode)
    }

    /// Extract plugin metadata from script comments
    ///
    /// Looks for special comments like:
    /// // @name: My Plugin
    /// // @version: 1.0.0
    /// // @description: Does something cool
    /// // @author: Jane Doe
    fn extract_metadata(path: &Path, source: &str) -> PluginMetadata {
        let mut name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let mut version = "0.1.0".to_string();
        let mut description = "Fusabi script plugin".to_string();
        let mut author = "Fusabi Frontend".to_string();

        // Parse metadata from comments
        for line in source.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("//").or_else(|| line.strip_prefix("(*")) {
                let rest = rest.trim();
                if let Some(value) = rest.strip_prefix("@name:") {
                    name = value.trim().to_string();
                } else if let Some(value) = rest.strip_prefix("@version:") {
                    version = value.trim().to_string();
                } else if let Some(value) = rest.strip_prefix("@description:") {
                    description = value.trim().to_string();
                } else if let Some(value) = rest.strip_prefix("@author:") {
                    author = value.trim().to_string();
                }
            }
        }

        PluginMetadata::new(name, version, description, author)
    }

    /// Create a VM with the compiled script and call a hook function
    ///
    /// Returns Ok(None) if the function doesn't exist (not an error)
    /// Returns Ok(Some(value)) if the function exists and was called successfully
    fn call_hook_function(
        &self,
        function_name: &str,
        args_source: &str,
        _ctx: &PluginContext,
    ) -> Result<Option<Value>> {
        // Use thread-local VM to avoid Send issues
        VM_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();

            // Create a new VM if not cached
            if cache.is_none() {
                *cache = Some(Vm::new());
            }

            let vm = cache.as_mut().unwrap();

            // Deserialize and execute the main script chunk to populate globals
            let bytecode = self
                .bytecode
                .as_ref()
                .ok_or_else(|| PluginError::LoadError("Bytecode not compiled".to_string()))?;

            let chunk = fusabi_vm::deserialize_chunk(bytecode)
                .map_err(|e| PluginError::Other(anyhow::anyhow!("Deserialization failed: {}", e)))?;

            vm.execute(chunk).map_err(|e| {
                PluginError::Other(anyhow::anyhow!("Script execution failed: {}", e))
            })?;

            // Check if the hook function exists in globals
            if !vm.globals.contains_key(function_name) {
                log::trace!(
                    "Hook function '{}' not defined in plugin '{}'",
                    function_name,
                    self.metadata.name
                );
                return Ok(None);
            }

            // Build and compile the function call expression
            let call_source = format!("{} {}", function_name, args_source);

            let mut lexer = Lexer::new(&call_source);
            let tokens = lexer.tokenize().map_err(|e| {
                PluginError::Other(anyhow::anyhow!("Failed to tokenize call: {}", e))
            })?;

            let mut parser = Parser::new(tokens);
            let expr = parser.parse().map_err(|e| {
                PluginError::Other(anyhow::anyhow!("Failed to parse call: {}", e))
            })?;

            let call_chunk = Compiler::compile(&expr).map_err(|e| {
                PluginError::Other(anyhow::anyhow!("Failed to compile call: {}", e))
            })?;

            // Execute the call
            let result = vm.execute(call_chunk).map_err(|e| {
                PluginError::Other(anyhow::anyhow!("Hook execution failed: {}", e))
            })?;

            Ok(Some(result))
        })
    }

    /// Hot-reload the script from disk
    pub fn reload(&mut self, path: &Path) -> Result<()> {
        let script_source = std::fs::read_to_string(path)
            .map_err(|e| PluginError::LoadError(format!("Failed to reload script: {}", e)))?;

        // Compile the new script
        let bytecode = Self::compile_script(&script_source)?;

        // Extract updated metadata
        let metadata = Self::extract_metadata(path, &script_source);

        // Update state
        self.script_source = script_source;
        self.bytecode = Some(bytecode);
        self.metadata = metadata;

        // Clear VM cache to force reload on next call
        VM_CACHE.with(|cache| {
            *cache.borrow_mut() = None;
        });

        log::info!("Hot-reloaded script plugin: {}", self.metadata.name);

        Ok(())
    }
}
#[async_trait]
impl Plugin for FusabiScriptPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()> {
        ctx.log(
            scarab_plugin_api::context::LogLevel::Info,
            &format!("Loading Fusabi script plugin: {}", self.metadata.name),
        );

        // Call the on_load hook if defined
        // Expected signature: let on_load = fun _u -> ()
        match self.call_hook_function("on_load", "()", ctx) {
            Ok(Some(result)) => {
                log::debug!(
                    "Plugin '{}' on_load returned: {:?}",
                    self.metadata.name,
                    result
                );
            }
            Ok(None) => {
                log::trace!("Plugin '{}' has no on_load hook", self.metadata.name);
            }
            Err(e) => {
                log::warn!(
                    "Plugin '{}' on_load hook failed: {}",
                    self.metadata.name,
                    e
                );
                return Err(e);
            }
        }

        Ok(())
    }

    async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action> {
        // Call the on_output hook if defined
        // Expected signature: let on_output = fun line -> bool (true = continue, false = suppress)
        let escaped_line = line.replace('\\', "\\\\").replace('"', "\\\"");
        let args = format!("\"{}\"", escaped_line);

        match self.call_hook_function("on_output", &args, ctx) {
            Ok(Some(result)) => {
                log::trace!(
                    "Plugin '{}' on_output returned: {:?}",
                    self.metadata.name,
                    result
                );

                // Check if result indicates we should suppress output
                match result {
                    Value::Bool(false) => Ok(Action::Stop),
                    _ => Ok(Action::Continue),
                }
            }
            Ok(None) => Ok(Action::Continue),
            Err(e) => {
                log::warn!(
                    "Plugin '{}' on_output hook failed: {}",
                    self.metadata.name,
                    e
                );
                Ok(Action::Continue)
            }
        }
    }

    async fn on_input(&mut self, input: &[u8], ctx: &PluginContext) -> Result<Action> {
        // Call the on_input hook if defined
        // Expected signature: let on_input = fun bytes -> bool
        let input_str = String::from_utf8_lossy(input);
        let escaped_input = input_str.replace('\\', "\\\\").replace('"', "\\\"");
        let args = format!("\"{}\"", escaped_input);

        match self.call_hook_function("on_input", &args, ctx) {
            Ok(Some(result)) => {
                log::trace!(
                    "Plugin '{}' on_input returned: {:?}",
                    self.metadata.name,
                    result
                );

                match result {
                    Value::Bool(false) => Ok(Action::Stop),
                    _ => Ok(Action::Continue),
                }
            }
            Ok(None) => Ok(Action::Continue),
            Err(e) => {
                log::warn!(
                    "Plugin '{}' on_input hook failed: {}",
                    self.metadata.name,
                    e
                );
                Ok(Action::Continue)
            }
        }
    }

    async fn on_resize(&mut self, cols: u16, rows: u16, ctx: &PluginContext) -> Result<()> {
        // Call the on_resize hook if defined
        // Expected signature: let on_resize = fun cols rows -> ()
        let args = format!("{} {}", cols, rows);

        match self.call_hook_function("on_resize", &args, ctx) {
            Ok(Some(result)) => {
                log::trace!(
                    "Plugin '{}' on_resize returned: {:?}",
                    self.metadata.name,
                    result
                );
            }
            Ok(None) => {}
            Err(e) => {
                log::warn!(
                    "Plugin '{}' on_resize hook failed: {}",
                    self.metadata.name,
                    e
                );
            }
        }

        Ok(())
    }

    async fn on_unload(&mut self) -> Result<()> {
        log::info!("Unloading Fusabi script plugin: {}", self.metadata.name);

        // Clear VM cache
        VM_CACHE.with(|cache| {
            *cache.borrow_mut() = None;
        });

        // Note: We can't call the on_unload hook here because we don't have access to ctx
        // The Plugin trait's on_unload doesn't provide a context parameter

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_invalid_bytecode() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"INVALID").unwrap();
        temp_file.flush().unwrap();

        let result = FusabiBytecodePlugin::load(temp_file.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid Fusabi bytecode"));
    }

    #[test]
    fn test_load_valid_bytecode() {
        use fusabi_vm::{Chunk, ChunkBuilder};

        // Create a valid Fusabi chunk
        let chunk = ChunkBuilder::new().build();
        let bytecode = fusabi_vm::serialize_chunk(&chunk).unwrap();

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&bytecode).unwrap();
        temp_file.flush().unwrap();

        let result = FusabiBytecodePlugin::load(temp_file.path());
        assert!(result.is_ok());
        let plugin = result.unwrap();
        assert!(plugin.metadata.name.len() > 0);
    }

    #[test]
    fn test_load_script() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file
            .write_all(b"let greeting = \"Hello, Fusabi!\"")
            .unwrap();
        temp_file.flush().unwrap();

        let result = FusabiScriptPlugin::load(temp_file.path());
        assert!(result.is_ok());
        let plugin = result.unwrap();
        assert!(plugin.metadata.name.len() > 0);
        // bytecode is serialized
        assert!(plugin.bytecode.is_some());
    }

    #[test]
    fn test_metadata_extraction() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file
            .write_all(
                b"// @name: TestPlugin\n// @version: 1.2.3\n// @description: A test plugin\n// @author: Test Author\nlet x = 42",
            )
            .unwrap();
        temp_file.flush().unwrap();

        let plugin = FusabiScriptPlugin::load(temp_file.path()).unwrap();
        assert_eq!(plugin.metadata.name, "TestPlugin");
        assert_eq!(plugin.metadata.version, "1.2.3");
        assert_eq!(plugin.metadata.description, "A test plugin");
        assert_eq!(plugin.metadata.author, "Test Author");
    }

    #[tokio::test]
    async fn test_bytecode_plugin_lifecycle() {
        use fusabi_vm::{Chunk, ChunkBuilder};

        // Create valid bytecode
        let chunk = ChunkBuilder::new().build();
        let bytecode = fusabi_vm::serialize_chunk(&chunk).unwrap();

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&bytecode).unwrap();
        temp_file.flush().unwrap();

        let mut plugin = FusabiBytecodePlugin::load(temp_file.path()).unwrap();

        // Create test context
        let state = std::sync::Arc::new(parking_lot::Mutex::new(
            scarab_plugin_api::context::PluginSharedState::new(80, 24),
        ));
        let mut ctx = PluginContext::new(Default::default(), state, "test");

        // Test lifecycle
        assert!(plugin.on_load(&mut ctx).await.is_ok());
        assert!(plugin.on_output("test line", &ctx).await.is_ok());
        assert!(plugin.on_input(b"test input", &ctx).await.is_ok());
        assert!(plugin.on_resize(100, 30, &ctx).await.is_ok());
        assert!(plugin.on_unload().await.is_ok());
    }

    #[tokio::test]
    async fn test_script_plugin_lifecycle() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file
            .write_all(b"let greeting = \"Hello, Fusabi!\"")
            .unwrap();
        temp_file.flush().unwrap();

        let mut plugin = FusabiScriptPlugin::load(temp_file.path()).unwrap();

        // Create test context
        let state = std::sync::Arc::new(parking_lot::Mutex::new(
            scarab_plugin_api::context::PluginSharedState::new(80, 24),
        ));
        let mut ctx = PluginContext::new(Default::default(), state, "test");

        // Test lifecycle
        assert!(plugin.on_load(&mut ctx).await.is_ok());
        assert!(plugin.on_output("test line", &ctx).await.is_ok());
        assert!(plugin.on_input(b"test input", &ctx).await.is_ok());
        assert!(plugin.on_resize(100, 30, &ctx).await.is_ok());
        assert!(plugin.on_unload().await.is_ok());
    }

    #[tokio::test]
    async fn test_script_with_hooks() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file
            .write_all(
                b"// Plugin with hooks\nlet on_load = fun _u -> ()\nlet on_output = fun line -> true",
            )
            .unwrap();
        temp_file.flush().unwrap();

        let mut plugin = FusabiScriptPlugin::load(temp_file.path()).unwrap();

        // Create test context
        let state = std::sync::Arc::new(parking_lot::Mutex::new(
            scarab_plugin_api::context::PluginSharedState::new(80, 24),
        ));
        let mut ctx = PluginContext::new(Default::default(), state, "test");

        // Test that hooks are called
        assert!(plugin.on_load(&mut ctx).await.is_ok());
        let action = plugin.on_output("test", &ctx).await.unwrap();
        assert_eq!(action, Action::Continue);
    }

    #[test]
    fn test_script_hot_reload() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"let x = 1").unwrap();
        temp_file.flush().unwrap();

        let mut plugin = FusabiScriptPlugin::load(temp_file.path()).unwrap();

        // Modify file
        temp_file.write_all(b"\nlet y = 2").unwrap();
        temp_file.flush().unwrap();

        // Reload
        assert!(plugin.reload(temp_file.path()).is_ok());
    }

    #[test]
    fn test_parse_error_handling() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"let = invalid syntax").unwrap();
        temp_file.flush().unwrap();

        let result = FusabiScriptPlugin::load(temp_file.path());
        assert!(result.is_err());
    }
}
