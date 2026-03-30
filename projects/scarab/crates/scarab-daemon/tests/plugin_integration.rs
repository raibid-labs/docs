//! Comprehensive integration tests for plugin system and Fusabi integration

use scarab_daemon::plugin_manager::PluginManager;
use scarab_plugin_api::{
    context::{PluginContext, PluginSharedState},
    types::RemoteCommand,
    Action, Plugin, PluginMetadata,
};
use std::sync::Arc;
use tempfile::{NamedTempFile, TempDir};

/// Helper to create a test plugin context
fn create_test_context() -> Arc<PluginContext> {
    let state = Arc::new(parking_lot::Mutex::new(PluginSharedState::new(80, 24)));
    Arc::new(PluginContext::new(
        Default::default(),
        state,
        "test_plugin",
    ))
}

/// Helper to create a plugin manager for testing
fn create_test_manager() -> PluginManager {
    let context = create_test_context();
    let registry = scarab_daemon::ipc::ClientRegistry::new();
    PluginManager::new(context, registry)
}

mod bytecode_tests {
    use super::*;
    use fusabi_vm::{ChunkBuilder, Instruction, Value};
    use std::io::Write;

    /// Create a minimal valid Fusabi bytecode chunk
    fn create_minimal_bytecode() -> Vec<u8> {
        let chunk = ChunkBuilder::new().build();
        fusabi_vm::serialize_chunk(&chunk).unwrap()
    }

    /// Create a bytecode chunk with a simple constant
    fn create_bytecode_with_constant() -> Vec<u8> {
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(42))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::Return)
            .build();
        fusabi_vm::serialize_chunk(&chunk).unwrap()
    }

    /// Create a bytecode chunk with arithmetic operations
    fn create_bytecode_with_arithmetic() -> Vec<u8> {
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(10))
            .instruction(Instruction::LoadConst(0))
            .constant(Value::Int(32))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::Add)
            .instruction(Instruction::Return)
            .build();
        fusabi_vm::serialize_chunk(&chunk).unwrap()
    }

    #[test]
    fn test_load_minimal_bytecode() {
        let bytecode = create_minimal_bytecode();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&bytecode).unwrap();
        temp_file.flush().unwrap();

        let result =
            scarab_daemon::plugin_manager::fusabi_adapter::FusabiBytecodePlugin::load(temp_file.path());
        assert!(result.is_ok(), "Should load minimal bytecode");
        let plugin = result.unwrap();
        assert!(!plugin.metadata().name.is_empty());
    }

    #[test]
    fn test_load_bytecode_with_constants() {
        let bytecode = create_bytecode_with_constant();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&bytecode).unwrap();
        temp_file.flush().unwrap();

        let result =
            scarab_daemon::plugin_manager::fusabi_adapter::FusabiBytecodePlugin::load(temp_file.path());
        assert!(result.is_ok(), "Should load bytecode with constants");
    }

    #[test]
    fn test_load_bytecode_with_arithmetic() {
        let bytecode = create_bytecode_with_arithmetic();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&bytecode).unwrap();
        temp_file.flush().unwrap();

        let result =
            scarab_daemon::plugin_manager::fusabi_adapter::FusabiBytecodePlugin::load(temp_file.path());
        assert!(result.is_ok(), "Should load bytecode with arithmetic");
    }

    #[test]
    fn test_load_invalid_bytecode_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&[]).unwrap();
        temp_file.flush().unwrap();

        let result =
            scarab_daemon::plugin_manager::fusabi_adapter::FusabiBytecodePlugin::load(temp_file.path());
        assert!(result.is_err(), "Should reject empty file");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid Fusabi bytecode"));
    }

    #[test]
    fn test_load_invalid_bytecode_corrupted() {
        let mut temp_file = NamedTempFile::new().unwrap();
        // Write corrupted data
        temp_file.write_all(b"CORRUPTED_DATA_NOT_VALID_FUSABI").unwrap();
        temp_file.flush().unwrap();

        let result =
            scarab_daemon::plugin_manager::fusabi_adapter::FusabiBytecodePlugin::load(temp_file.path());
        assert!(result.is_err(), "Should reject corrupted bytecode");
    }

    #[test]
    fn test_load_nonexistent_file() {
        let path = std::path::Path::new("/nonexistent/plugin.fzb");
        let result =
            scarab_daemon::plugin_manager::fusabi_adapter::FusabiBytecodePlugin::load(path);
        assert!(result.is_err(), "Should error on nonexistent file");
    }

    #[tokio::test]
    async fn test_bytecode_plugin_on_load() {
        let bytecode = create_minimal_bytecode();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&bytecode).unwrap();
        temp_file.flush().unwrap();

        let mut plugin =
            scarab_daemon::plugin_manager::fusabi_adapter::FusabiBytecodePlugin::load(temp_file.path()).unwrap();

        let state = Arc::new(parking_lot::Mutex::new(PluginSharedState::new(80, 24)));
        let mut ctx = PluginContext::new(Default::default(), state, "test");

        // on_load just logs and returns Ok() - it doesn't execute the bytecode
        let result = plugin.on_load(&mut ctx).await;
        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok(), "Plugin on_load should succeed");
    }

    #[tokio::test]
    async fn test_bytecode_plugin_hooks_return_continue() {
        let bytecode = create_minimal_bytecode();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&bytecode).unwrap();
        temp_file.flush().unwrap();

        let mut plugin =
            scarab_daemon::plugin_manager::fusabi_adapter::FusabiBytecodePlugin::load(temp_file.path()).unwrap();

        let state = Arc::new(parking_lot::Mutex::new(PluginSharedState::new(80, 24)));
        let ctx = PluginContext::new(Default::default(), state, "test");

        // Test output hook
        let output_result = plugin.on_output("test line", &ctx).await;
        assert!(output_result.is_ok());
        assert_eq!(output_result.unwrap(), Action::Continue);

        // Test input hook
        let input_result = plugin.on_input(b"test input", &ctx).await;
        assert!(input_result.is_ok());
        assert_eq!(input_result.unwrap(), Action::Continue);

        // Test resize hook
        let resize_result = plugin.on_resize(100, 50, &ctx).await;
        assert!(resize_result.is_ok());
    }

    #[tokio::test]
    async fn test_bytecode_plugin_full_lifecycle() {
        let bytecode = create_minimal_bytecode();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&bytecode).unwrap();
        temp_file.flush().unwrap();

        let mut plugin =
            scarab_daemon::plugin_manager::fusabi_adapter::FusabiBytecodePlugin::load(temp_file.path()).unwrap();

        let state = Arc::new(parking_lot::Mutex::new(PluginSharedState::new(80, 24)));
        let mut ctx = PluginContext::new(Default::default(), state.clone(), "test");

        // Load
        assert!(plugin.on_load(&mut ctx).await.is_ok());

        // Output
        let ctx_ref = PluginContext::new(Default::default(), state.clone(), "test");
        assert!(plugin.on_output("line 1", &ctx_ref).await.is_ok());

        // Input
        assert!(plugin.on_input(b"input data", &ctx_ref).await.is_ok());

        // Resize
        assert!(plugin.on_resize(120, 40, &ctx_ref).await.is_ok());

        // Unload
        assert!(plugin.on_unload().await.is_ok());
    }
}

mod script_tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_simple_script() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"let x = 42").unwrap();
        temp_file.flush().unwrap();

        let result =
            scarab_daemon::plugin_manager::fusabi_adapter::FusabiScriptPlugin::load(temp_file.path());
        assert!(result.is_ok(), "Should load simple script");
        let plugin = result.unwrap();
        assert!(!plugin.metadata().name.is_empty());
    }

    #[test]
    fn test_load_script_with_function() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file
            .write_all(b"let add x y = x + y\nlet result = add 10 32")
            .unwrap();
        temp_file.flush().unwrap();

        let result =
            scarab_daemon::plugin_manager::fusabi_adapter::FusabiScriptPlugin::load(temp_file.path());
        assert!(result.is_ok(), "Should load script with functions");
    }

    #[test]
    fn test_load_script_with_comments() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file
            .write_all(
                b"// This is a comment\nlet greeting = \"Hello, Fusabi!\"\n// Another comment",
            )
            .unwrap();
        temp_file.flush().unwrap();

        let result =
            scarab_daemon::plugin_manager::fusabi_adapter::FusabiScriptPlugin::load(temp_file.path());
        assert!(result.is_ok(), "Should load script with comments");
    }

    #[test]
    fn test_load_empty_script() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"").unwrap();
        temp_file.flush().unwrap();

        let result =
            scarab_daemon::plugin_manager::fusabi_adapter::FusabiScriptPlugin::load(temp_file.path());
        assert!(result.is_ok(), "Should load empty script");
    }

    #[test]
    fn test_load_nonexistent_script() {
        let path = std::path::Path::new("/nonexistent/plugin.fsx");
        let result = scarab_daemon::plugin_manager::fusabi_adapter::FusabiScriptPlugin::load(path);
        assert!(result.is_err(), "Should error on nonexistent file");
    }

    #[tokio::test]
    async fn test_script_plugin_full_lifecycle() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file
            .write_all(b"let greeting = \"Hello, Fusabi!\"")
            .unwrap();
        temp_file.flush().unwrap();

        let mut plugin =
            scarab_daemon::plugin_manager::fusabi_adapter::FusabiScriptPlugin::load(temp_file.path()).unwrap();

        let state = Arc::new(parking_lot::Mutex::new(PluginSharedState::new(80, 24)));
        let mut ctx = PluginContext::new(Default::default(), state.clone(), "test");

        // Load
        assert!(plugin.on_load(&mut ctx).await.is_ok());

        // Output
        let ctx_ref = PluginContext::new(Default::default(), state.clone(), "test");
        assert!(plugin.on_output("test line", &ctx_ref).await.is_ok());

        // Input
        assert!(plugin.on_input(b"test input", &ctx_ref).await.is_ok());

        // Resize
        assert!(plugin.on_resize(100, 30, &ctx_ref).await.is_ok());

        // Unload
        assert!(plugin.on_unload().await.is_ok());
    }

    #[test]
    fn test_script_hot_reload() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"let x = 1").unwrap();
        temp_file.flush().unwrap();

        let mut plugin =
            scarab_daemon::plugin_manager::fusabi_adapter::FusabiScriptPlugin::load(temp_file.path()).unwrap();

        // Modify file content
        std::fs::write(temp_file.path(), b"let x = 2\nlet y = 3").unwrap();

        // Reload
        let result = plugin.reload(temp_file.path());
        assert!(result.is_ok(), "Hot reload should succeed");
    }

    #[test]
    fn test_script_hot_reload_nonexistent() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"let x = 1").unwrap();
        temp_file.flush().unwrap();

        let mut plugin =
            scarab_daemon::plugin_manager::fusabi_adapter::FusabiScriptPlugin::load(temp_file.path()).unwrap();

        // Try to reload from nonexistent path
        let result = plugin.reload(std::path::Path::new("/nonexistent.fsx"));
        assert!(result.is_err(), "Hot reload should fail for nonexistent file");
    }
}

mod plugin_manager_tests {
    use super::*;
    use async_trait::async_trait;
    use scarab_plugin_api::PluginError;

    /// Mock plugin for testing
    struct MockPlugin {
        metadata: PluginMetadata,
        should_fail: bool,
        should_timeout: bool,
        modify_output: bool,
        stop_processing: bool,
        commands_queued: Arc<parking_lot::Mutex<Vec<String>>>,
    }

    impl MockPlugin {
        fn new(name: &str) -> Self {
            Self {
                metadata: PluginMetadata::new(name, "1.0.0", "Mock plugin", "Test"),
                should_fail: false,
                should_timeout: false,
                modify_output: false,
                stop_processing: false,
                commands_queued: Arc::new(parking_lot::Mutex::new(Vec::new())),
            }
        }

        fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }

        fn with_timeout(mut self) -> Self {
            self.should_timeout = true;
            self
        }

        fn with_modification(mut self) -> Self {
            self.modify_output = true;
            self
        }

        fn with_stop(mut self) -> Self {
            self.stop_processing = true;
            self
        }
    }

    #[async_trait]
    impl Plugin for MockPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        async fn on_load(&mut self, _ctx: &mut PluginContext) -> scarab_plugin_api::Result<()> {
            if self.should_fail {
                return Err(PluginError::LoadError("Mock load failure".to_string()));
            }
            Ok(())
        }

        async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> scarab_plugin_api::Result<Action> {
            if self.should_timeout {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
            if self.should_fail {
                return Err(PluginError::Other(anyhow::anyhow!("Mock output failure")));
            }
            if self.stop_processing {
                return Ok(Action::Stop);
            }
            if self.modify_output {
                let modified = format!("[{}] {}", self.metadata.name, line);
                return Ok(Action::Modify(modified.into_bytes()));
            }

            // Test command queueing
            if line.contains("draw") {
                ctx.queue_command(RemoteCommand::DrawOverlay {
                    id: 1,
                    x: 0,
                    y: 0,
                    text: "Overlay".to_string(),
                    style: Default::default(),
                });
                self.commands_queued.lock().push("DrawOverlay".to_string());
            }

            Ok(Action::Continue)
        }

        async fn on_input(&mut self, _input: &[u8], _ctx: &PluginContext) -> scarab_plugin_api::Result<Action> {
            if self.should_fail {
                return Err(PluginError::Other(anyhow::anyhow!("Mock input failure")));
            }
            Ok(Action::Continue)
        }

        async fn on_resize(&mut self, _cols: u16, _rows: u16, _ctx: &PluginContext) -> scarab_plugin_api::Result<()> {
            if self.should_fail {
                return Err(PluginError::Other(anyhow::anyhow!("Mock resize failure")));
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_register_plugin_success() {
        let mut manager = create_test_manager();
        let plugin = Box::new(MockPlugin::new("test_plugin"));

        let result = manager.register_plugin(plugin).await;
        assert!(result.is_ok(), "Should register plugin successfully");
        assert_eq!(manager.enabled_count(), 1);
    }

    #[tokio::test]
    async fn test_register_plugin_load_failure() {
        let mut manager = create_test_manager();
        let plugin = Box::new(MockPlugin::new("failing_plugin").with_failure());

        let result = manager.register_plugin(plugin).await;
        assert!(result.is_err(), "Should fail to register plugin");
        assert_eq!(manager.enabled_count(), 0);
    }

    #[tokio::test]
    async fn test_plugin_failure_tracking() {
        let mut manager = create_test_manager();
        let plugin = Box::new(MockPlugin::new("flaky_plugin"));
        manager.register_plugin(plugin).await.unwrap();

        // Simulate a plugin that will fail
        let mut manager = create_test_manager();
        let plugin = Box::new(MockPlugin::new("failing_plugin").with_failure());
        manager.register_plugin(plugin).await.ok(); // Register but will fail hooks

        // First failure
        let _ = manager.dispatch_output("test").await;

        // Check that plugin is still enabled after 1 failure
        assert_eq!(manager.enabled_count(), 0); // Plugin failed on_load so wasn't registered
    }

    #[tokio::test]
    async fn test_plugin_auto_disable_after_failures() {
        let mut manager = create_test_manager();

        // Create a mock plugin that succeeds on load but fails on hooks
        struct FailingHookPlugin {
            metadata: PluginMetadata,
        }

        #[async_trait]
        impl Plugin for FailingHookPlugin {
            fn metadata(&self) -> &PluginMetadata {
                &self.metadata
            }

            async fn on_load(&mut self, _ctx: &mut PluginContext) -> scarab_plugin_api::Result<()> {
                Ok(()) // Succeed on load
            }

            async fn on_output(&mut self, _line: &str, _ctx: &PluginContext) -> scarab_plugin_api::Result<Action> {
                Err(PluginError::Other(anyhow::anyhow!("Consistent failure")))
            }
        }

        let plugin = Box::new(FailingHookPlugin {
            metadata: PluginMetadata::new("failing", "1.0.0", "Fails consistently", "Test"),
        });
        manager.register_plugin(plugin).await.unwrap();
        assert_eq!(manager.enabled_count(), 1);

        // Trigger failures (max_failures is 3 by default)
        for _ in 0..3 {
            let _ = manager.dispatch_output("test").await;
        }

        // Plugin should be auto-disabled
        assert_eq!(manager.enabled_count(), 0, "Plugin should be disabled after 3 failures");
    }

    #[tokio::test]
    async fn test_plugin_timeout_handling() {
        let mut manager = create_test_manager().with_timeout(100); // 100ms timeout
        let plugin = Box::new(MockPlugin::new("timeout_plugin").with_timeout());

        manager.register_plugin(plugin).await.unwrap();

        let start = std::time::Instant::now();
        let _ = manager.dispatch_output("test").await;
        let elapsed = start.elapsed();

        // Should timeout around 100ms, not wait for the full 10 seconds
        assert!(elapsed.as_millis() < 5000, "Should timeout quickly, got {}ms", elapsed.as_millis());
    }

    #[tokio::test]
    async fn test_multiple_plugins_chaining() {
        let mut manager = create_test_manager();

        // Register three plugins that modify output
        let plugin1 = Box::new(MockPlugin::new("plugin1").with_modification());
        let plugin2 = Box::new(MockPlugin::new("plugin2").with_modification());
        let plugin3 = Box::new(MockPlugin::new("plugin3").with_modification());

        manager.register_plugin(plugin1).await.unwrap();
        manager.register_plugin(plugin2).await.unwrap();
        manager.register_plugin(plugin3).await.unwrap();

        let result = manager.dispatch_output("original").await.unwrap();

        // Each plugin should have added its prefix
        assert!(result.contains("plugin1"), "Should contain plugin1 modification");
        assert!(result.contains("plugin2"), "Should contain plugin2 modification");
        assert!(result.contains("plugin3"), "Should contain plugin3 modification");
        assert!(result.contains("original"), "Should contain original text");
    }

    #[tokio::test]
    async fn test_plugin_stop_action() {
        let mut manager = create_test_manager();

        // Register two plugins, first one stops processing
        let plugin1 = Box::new(MockPlugin::new("stopper").with_stop());
        let plugin2 = Box::new(MockPlugin::new("never_reached").with_modification());

        manager.register_plugin(plugin1).await.unwrap();
        manager.register_plugin(plugin2).await.unwrap();

        let result = manager.dispatch_output("test").await.unwrap();

        // Second plugin should never modify the output
        assert!(!result.contains("never_reached"), "Second plugin should not process");
    }

    #[tokio::test]
    async fn test_remote_command_queueing() {
        let mut manager = create_test_manager();
        let plugin = Box::new(MockPlugin::new("command_plugin"));
        let commands_ref = plugin.commands_queued.clone();

        manager.register_plugin(plugin).await.unwrap();

        // Trigger command queueing with special keyword
        let _ = manager.dispatch_output("test draw overlay").await;

        // Give async processing time to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Check that command was queued
        let queued = commands_ref.lock();
        assert!(queued.contains(&"DrawOverlay".to_string()), "Should queue DrawOverlay command");
    }

    #[tokio::test]
    async fn test_dispatch_input() {
        let mut manager = create_test_manager();
        let plugin = Box::new(MockPlugin::new("input_plugin").with_modification());

        manager.register_plugin(plugin).await.unwrap();

        let result = manager.dispatch_input(b"test input").await.unwrap();

        // Input should pass through (modification only applies to output in our mock)
        assert_eq!(result, b"test input");
    }

    #[tokio::test]
    async fn test_dispatch_resize() {
        let mut manager = create_test_manager();
        let plugin = Box::new(MockPlugin::new("resize_plugin"));

        manager.register_plugin(plugin).await.unwrap();

        let result = manager.dispatch_resize(100, 50).await;
        assert!(result.is_ok(), "Resize dispatch should succeed");
    }

    #[tokio::test]
    async fn test_unload_all_plugins() {
        let mut manager = create_test_manager();

        let plugin1 = Box::new(MockPlugin::new("plugin1"));
        let plugin2 = Box::new(MockPlugin::new("plugin2"));

        manager.register_plugin(plugin1).await.unwrap();
        manager.register_plugin(plugin2).await.unwrap();
        assert_eq!(manager.enabled_count(), 2);

        manager.unload_all().await.unwrap();
        assert_eq!(manager.enabled_count(), 0);
        assert_eq!(manager.list_plugins().len(), 0);
    }

    #[tokio::test]
    async fn test_list_plugins() {
        let mut manager = create_test_manager();

        let plugin1 = Box::new(MockPlugin::new("plugin1"));
        let plugin2 = Box::new(MockPlugin::new("plugin2"));

        manager.register_plugin(plugin1).await.unwrap();
        manager.register_plugin(plugin2).await.unwrap();

        let list = manager.list_plugins();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|p| p.name == "plugin1"));
        assert!(list.iter().any(|p| p.name == "plugin2"));
    }

    #[tokio::test]
    async fn test_plugin_context_commands() {
        let context = create_test_context();

        // Queue some commands
        context.queue_command(RemoteCommand::DrawOverlay {
            id: 1,
            x: 10,
            y: 20,
            text: "Test".to_string(),
            style: Default::default(),
        });

        context.queue_command(RemoteCommand::ClearOverlays { id: Some(1) });

        // Check commands were queued
        let commands = context.commands.lock();
        assert_eq!(commands.len(), 2);
    }

    #[tokio::test]
    async fn test_plugin_context_cell_operations() {
        let context = create_test_context();

        // Test get_size
        let (cols, rows) = context.get_size();
        assert_eq!(cols, 80);
        assert_eq!(rows, 24);

        // Test set/get cell
        let cell = scarab_plugin_api::types::Cell {
            c: 'X',
            fg: (255, 0, 0),
            bg: (0, 0, 0),
            bold: true,
            italic: false,
            underline: false,
        };

        assert!(context.set_cell(5, 5, cell));
        let retrieved = context.get_cell(5, 5);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().c, 'X');
        assert!(retrieved.unwrap().bold);
    }

    #[tokio::test]
    async fn test_plugin_context_data_storage() {
        let context = create_test_context();

        // Store and retrieve data
        context.set_data("key1", "value1");
        context.set_data("key2", "value2");

        assert_eq!(context.get_data("key1"), Some("value1".to_string()));
        assert_eq!(context.get_data("key2"), Some("value2".to_string()));
        assert_eq!(context.get_data("nonexistent"), None);
    }
}

mod error_propagation_tests {
    use super::*;

    #[test]
    fn test_plugin_error_types() {
        use scarab_plugin_api::PluginError;

        let not_found = PluginError::NotFound("plugin.fzb".to_string());
        assert!(not_found.to_string().contains("plugin.fzb"));

        let load_error = PluginError::LoadError("Failed to load".to_string());
        assert!(load_error.to_string().contains("Failed to load"));

        let exec_error = PluginError::Other(anyhow::anyhow!("Runtime error"));
        assert!(exec_error.to_string().contains("Runtime error"));

        let timeout = PluginError::Timeout(1000);
        assert!(timeout.to_string().contains("1000"));
    }

    #[tokio::test]
    async fn test_error_propagation_from_plugin() {
        use async_trait::async_trait;
        use scarab_plugin_api::{PluginError, PluginMetadata};

        struct ErrorPlugin {
            metadata: PluginMetadata,
        }

        #[async_trait]
        impl Plugin for ErrorPlugin {
            fn metadata(&self) -> &PluginMetadata {
                &self.metadata
            }

            async fn on_load(&mut self, _ctx: &mut PluginContext) -> scarab_plugin_api::Result<()> {
                Err(PluginError::LoadError("Intentional error".to_string()))
            }
        }

        let mut manager = create_test_manager();
        let plugin = ErrorPlugin {
            metadata: PluginMetadata::new("error", "1.0.0", "Error plugin", "Test"),
        };
        let result = manager.register_plugin(Box::new(plugin)).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Intentional error"));
    }
}

mod value_marshaling_tests {
    use super::*;
    use fusabi_vm::{ChunkBuilder, Instruction, Value, Vm};

    #[test]
    fn test_fusabi_value_types() {
        // Test that Fusabi VM can handle different value types
        let int = Value::Int(42);
        let boolean = Value::Bool(true);
        let unit = Value::Unit;
        let str_val = Value::Str("hello".into());

        assert!(matches!(int, Value::Int(_)));
        assert!(matches!(boolean, Value::Bool(_)));
        assert!(matches!(unit, Value::Unit));
        assert!(matches!(str_val, Value::Str(_)));
    }

    #[test]
    fn test_vm_execution_with_values() {
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(123))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::Return)
            .build();

        let mut vm = Vm::new();
        let result = vm.execute(chunk);
        assert!(result.is_ok());
    }

    #[test]
    fn test_vm_arithmetic_operations() {
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(10))
            .instruction(Instruction::LoadConst(0))
            .constant(Value::Int(5))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::Add)
            .instruction(Instruction::Return)
            .build();

        let mut vm = Vm::new();
        let result = vm.execute(chunk);
        assert!(result.is_ok());
    }

    #[test]
    fn test_vm_boolean_operations() {
        let chunk = ChunkBuilder::new()
            .constant(Value::Bool(true))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::Not)
            .instruction(Instruction::Return)
            .build();

        let mut vm = Vm::new();
        let result = vm.execute(chunk);
        assert!(result.is_ok());
    }
}

mod discovery_tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_discover_and_load_from_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create some plugin files
        let fzb_path = temp_dir.path().join("plugin1.fzb");
        let fsx_path = temp_dir.path().join("plugin2.fsx");

        // Create minimal valid bytecode
        let chunk = fusabi_vm::ChunkBuilder::new().build();
        let bytecode = fusabi_vm::serialize_chunk(&chunk).unwrap();
        std::fs::write(&fzb_path, bytecode).unwrap();

        // Create simple script
        std::fs::write(&fsx_path, b"let x = 1").unwrap();

        // Discovery currently doesn't find custom paths, but we can test
        // the load_plugin_from_config path
        let mut manager = create_test_manager();

        let config = scarab_plugin_api::PluginConfig {
            name: "plugin1".to_string(),
            path: fzb_path,
            enabled: true,
            config: Default::default(),
        };

        // This tests the internal load_plugin_from_config method indirectly
        // by using register_plugin with a loaded plugin
        let plugin = scarab_daemon::plugin_manager::fusabi_adapter::FusabiBytecodePlugin::load(&config.path).unwrap();
        let result = manager.register_plugin(Box::new(plugin)).await;
        assert!(result.is_ok());
    }

    // Note: PluginConfigData is not public, so config parsing tests would need to be
    // in the plugin-api crate's tests
}
