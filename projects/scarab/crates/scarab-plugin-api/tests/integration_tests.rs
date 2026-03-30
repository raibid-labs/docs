//! Integration tests for the plugin API

use scarab_plugin_api::*;
use std::sync::Arc;

// Helper to create test context
fn create_test_context() -> Arc<PluginContext> {
    let state = Arc::new(parking_lot::Mutex::new(context::PluginSharedState::new(
        80, 24,
    )));
    Arc::new(PluginContext::new(
        context::PluginConfigData::default(),
        state,
        "test-plugin",
    ))
}

// Mock plugin for testing
struct TestPlugin {
    metadata: PluginMetadata,
    call_count: u32,
}

impl TestPlugin {
    fn new() -> Self {
        Self {
            metadata: PluginMetadata::new("test-plugin", "1.0.0", "Test plugin", "Test Author"),
            call_count: 0,
        }
    }
}

#[async_trait::async_trait]
impl Plugin for TestPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()> {
        ctx.log(context::LogLevel::Info, "Test plugin loaded");
        Ok(())
    }

    async fn on_output(&mut self, line: &str, _ctx: &PluginContext) -> Result<Action> {
        self.call_count += 1;

        if line.contains("MODIFY") {
            Ok(Action::Modify(b"MODIFIED".to_vec()))
        } else if line.contains("STOP") {
            Ok(Action::Stop)
        } else {
            Ok(Action::Continue)
        }
    }

    async fn on_input(&mut self, input: &[u8], _ctx: &PluginContext) -> Result<Action> {
        if input == b"\x18" {
            // Ctrl+X
            Ok(Action::Stop)
        } else {
            Ok(Action::Continue)
        }
    }
}

#[tokio::test]
async fn test_plugin_metadata() {
    let plugin = TestPlugin::new();
    let meta = plugin.metadata();

    assert_eq!(meta.name, "test-plugin");
    assert_eq!(meta.version, "1.0.0");
    assert!(meta.is_compatible(API_VERSION));
}

#[tokio::test]
async fn test_plugin_load() {
    let mut plugin = TestPlugin::new();
    let ctx = create_test_context();
    let mut ctx_mut = (*ctx).clone();

    let result = plugin.on_load(&mut ctx_mut).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_output_hook_continue() {
    let mut plugin = TestPlugin::new();
    let ctx = create_test_context();

    let result = plugin.on_output("test line", &ctx).await.unwrap();
    assert_eq!(result, Action::Continue);
    assert_eq!(plugin.call_count, 1);
}

#[tokio::test]
async fn test_output_hook_modify() {
    let mut plugin = TestPlugin::new();
    let ctx = create_test_context();

    let result = plugin.on_output("MODIFY this", &ctx).await.unwrap();
    assert!(result.is_modify());

    if let Action::Modify(data) = result {
        assert_eq!(data, b"MODIFIED");
    } else {
        panic!("Expected Modify action");
    }
}

#[tokio::test]
async fn test_output_hook_stop() {
    let mut plugin = TestPlugin::new();
    let ctx = create_test_context();

    let result = plugin.on_output("STOP here", &ctx).await.unwrap();
    assert!(result.is_stop());
}

#[tokio::test]
async fn test_input_hook() {
    let mut plugin = TestPlugin::new();
    let ctx = create_test_context();

    // Normal input
    let result = plugin.on_input(b"a", &ctx).await.unwrap();
    assert_eq!(result, Action::Continue);

    // Ctrl+X
    let result = plugin.on_input(b"\x18", &ctx).await.unwrap();
    assert!(result.is_stop());
}

#[test]
fn test_version_compatibility() {
    let meta = PluginMetadata::new("test", "1.0.0", "Test", "Author").with_api_version("0.1.0");

    // Same version
    assert!(meta.is_compatible("0.1.0"));

    // Newer minor version
    assert!(meta.is_compatible("0.2.0"));
    assert!(meta.is_compatible("0.10.0"));

    // Older minor version
    assert!(!meta.is_compatible("0.0.1"));

    // Different major version
    assert!(!meta.is_compatible("1.0.0"));
}

#[test]
fn test_plugin_config_data() {
    use std::collections::HashMap;

    let mut data = HashMap::new();
    data.insert("threshold".to_string(), toml::Value::Integer(42));
    data.insert("enabled".to_string(), toml::Value::Boolean(true));

    let config = context::PluginConfigData { data };

    // Get existing value
    let threshold: i64 = config.get("threshold").unwrap();
    assert_eq!(threshold, 42);

    // Get optional existing value
    let enabled: bool = config.get_opt("enabled").unwrap();
    assert!(enabled);

    // Get missing optional value
    let missing: Option<String> = config.get_opt("missing");
    assert!(missing.is_none());
}

#[test]
fn test_plugin_discovery() {
    let _discovery = PluginDiscovery::new();

    // Test default paths
    let plugin_dir = PluginDiscovery::default_plugin_dir();
    assert!(plugin_dir
        .to_string_lossy()
        .contains(".config/scarab/plugins"));

    let config_path = PluginDiscovery::default_config_path();
    assert!(config_path
        .to_string_lossy()
        .contains(".config/scarab/plugins.toml"));
}

#[test]
fn test_cell_default() {
    let cell = types::Cell::default();
    assert_eq!(cell.c, ' ');
    assert_eq!(cell.fg, (255, 255, 255));
    assert_eq!(cell.bg, (0, 0, 0));
    assert!(!cell.bold);
    assert!(!cell.italic);
    assert!(!cell.underline);
}

#[test]
fn test_shared_state() {
    let mut state = context::PluginSharedState::new(80, 24);

    // Test get/set cell
    assert!(state.get_cell(0, 0).is_some());
    assert!(state.get_cell(80, 0).is_none()); // Out of bounds

    let cell = types::Cell {
        c: 'X',
        fg: (255, 0, 0),
        bg: (0, 0, 0),
        bold: true,
        italic: false,
        underline: false,
    };

    assert!(state.set_cell(0, 0, cell));
    assert!(!state.set_cell(100, 100, cell)); // Out of bounds

    let retrieved = state.get_cell(0, 0).unwrap();
    assert_eq!(retrieved.c, 'X');
    assert_eq!(retrieved.fg, (255, 0, 0));
    assert!(retrieved.bold);
}

#[test]
fn test_plugin_context_data_storage() {
    let ctx = create_test_context();

    // Store data
    ctx.set_data("key1", "value1");
    ctx.set_data("key2", "value2");

    // Retrieve data
    assert_eq!(ctx.get_data("key1"), Some("value1".to_string()));
    assert_eq!(ctx.get_data("key2"), Some("value2".to_string()));
    assert_eq!(ctx.get_data("missing"), None);

    // Overwrite data
    ctx.set_data("key1", "new_value");
    assert_eq!(ctx.get_data("key1"), Some("new_value".to_string()));
}

#[test]
fn test_hook_type() {
    let all_hooks = HookType::all();
    assert_eq!(all_hooks.len(), 7);

    assert_eq!(HookType::PreOutput.name(), "pre-output");
    assert_eq!(HookType::PostInput.name(), "post-input");
    assert_eq!(HookType::OnResize.name(), "on-resize");
}

#[test]
fn test_action_checks() {
    assert!(!Action::Continue.is_modify());
    assert!(!Action::Continue.is_stop());

    assert!(!Action::Stop.is_modify());
    assert!(Action::Stop.is_stop());

    let modify = Action::Modify(vec![1, 2, 3]);
    assert!(modify.is_modify());
    assert!(!modify.is_stop());
}

#[test]
fn test_plugin_info() {
    let info = types::PluginInfo::new("test", "1.0.0", "Test plugin", "Author");

    assert_eq!(info.name, "test");
    assert_eq!(info.version, "1.0.0");
    assert!(info.enabled);
    assert_eq!(info.failure_count, 0);
}
