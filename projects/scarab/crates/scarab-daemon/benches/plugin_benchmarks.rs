//! Comprehensive Plugin System Performance Benchmarks
//!
//! This benchmark suite measures the performance overhead of the plugin system
//! across various scenarios to identify bottlenecks and optimization opportunities.

use criterion::{
    async_executor::FuturesExecutor, black_box, criterion_group, criterion_main, BenchmarkId,
    Criterion, Throughput,
};
use fusabi_frontend::{Compiler, Lexer, Parser};
use fusabi_vm::{ChunkBuilder, Instruction, Value, Vm};
use scarab_daemon::plugin_manager::{
    fusabi_adapter::{FusabiBytecodePlugin, FusabiScriptPlugin},
    PluginManager,
};
use scarab_plugin_api::{
    context::{PluginContext, PluginSharedState},
    Action, Plugin, PluginMetadata,
};
use std::{
    io::Write,
    sync::Arc,
    time::{Duration, Instant},
};
use tempfile::NamedTempFile;

// ============================================================================
// Benchmark Helpers
// ============================================================================

/// Create a test plugin context
fn create_test_context() -> Arc<PluginContext> {
    let state = Arc::new(parking_lot::Mutex::new(PluginSharedState::new(80, 24)));
    Arc::new(PluginContext::new(
        Default::default(),
        state,
        "bench_plugin",
    ))
}

/// Create a plugin manager for benchmarking
fn create_test_manager() -> PluginManager {
    let context = create_test_context();
    let registry = scarab_daemon::ipc::ClientRegistry::new();
    PluginManager::new(context, registry)
}

/// Create minimal valid bytecode
fn create_minimal_bytecode() -> Vec<u8> {
    let chunk = ChunkBuilder::new().build();
    fusabi_vm::serialize_chunk(&chunk).unwrap()
}

/// Create bytecode with hook function that returns Continue
fn create_bytecode_with_hook() -> Vec<u8> {
    let chunk = ChunkBuilder::new()
        .constant(Value::Bool(true))
        .instruction(Instruction::LoadConst(0))
        .instruction(Instruction::Return)
        .build();
    fusabi_vm::serialize_chunk(&chunk).unwrap()
}

/// Create bytecode with complex arithmetic operations
fn create_complex_bytecode() -> Vec<u8> {
    let mut builder = ChunkBuilder::new();

    // Simulate complex processing: (a + b) * (c - d) / e
    for val in [10, 20, 30, 5, 3] {
        builder = builder.constant(Value::Int(val));
    }

    builder = builder
        .instruction(Instruction::LoadConst(0)) // 10
        .instruction(Instruction::LoadConst(1)) // 20
        .instruction(Instruction::Add) // 30
        .instruction(Instruction::LoadConst(2)) // 30
        .instruction(Instruction::LoadConst(3)) // 5
        .instruction(Instruction::Sub) // 25
        .instruction(Instruction::Mul) // 750
        .instruction(Instruction::LoadConst(4)) // 3
        .instruction(Instruction::Div) // 250
        .instruction(Instruction::Return);

    let chunk = builder.build();
    fusabi_vm::serialize_chunk(&chunk).unwrap()
}

/// Generate test output lines
fn generate_output_lines(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            format!(
                "[{:06}] Processing request #{} with status: {}",
                i,
                i,
                if i % 3 == 0 { "OK" } else { "PENDING" }
            )
        })
        .collect()
}

/// Generate test input bytes
fn generate_input_data(count: usize) -> Vec<Vec<u8>> {
    (0..count)
        .map(|i| format!("command_{}\n", i).into_bytes())
        .collect()
}

/// Simple no-op plugin for baseline measurements
struct NoOpPlugin {
    metadata: PluginMetadata,
}

impl NoOpPlugin {
    fn new() -> Self {
        Self {
            metadata: PluginMetadata::new("noop", "1.0.0", "No-op plugin", "Benchmark"),
        }
    }
}

#[async_trait::async_trait]
impl Plugin for NoOpPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_load(&mut self, _ctx: &mut PluginContext) -> scarab_plugin_api::Result<()> {
        Ok(())
    }

    async fn on_output(
        &mut self,
        _line: &str,
        _ctx: &PluginContext,
    ) -> scarab_plugin_api::Result<Action> {
        Ok(Action::Continue)
    }

    async fn on_input(
        &mut self,
        _input: &[u8],
        _ctx: &PluginContext,
    ) -> scarab_plugin_api::Result<Action> {
        Ok(Action::Continue)
    }

    async fn on_resize(
        &mut self,
        _cols: u16,
        _rows: u16,
        _ctx: &PluginContext,
    ) -> scarab_plugin_api::Result<()> {
        Ok(())
    }
}

/// Plugin that performs simple string processing
struct ProcessingPlugin {
    metadata: PluginMetadata,
    counter: usize,
}

impl ProcessingPlugin {
    fn new() -> Self {
        Self {
            metadata: PluginMetadata::new("processor", "1.0.0", "Processing plugin", "Benchmark"),
            counter: 0,
        }
    }
}

#[async_trait::async_trait]
impl Plugin for ProcessingPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_load(&mut self, _ctx: &mut PluginContext) -> scarab_plugin_api::Result<()> {
        Ok(())
    }

    async fn on_output(
        &mut self,
        line: &str,
        _ctx: &PluginContext,
    ) -> scarab_plugin_api::Result<Action> {
        // Simulate work: count characters, check patterns
        self.counter += line.len();
        if line.contains("ERROR") || line.contains("WARN") {
            self.counter += 10;
        }
        Ok(Action::Continue)
    }

    async fn on_input(
        &mut self,
        input: &[u8],
        _ctx: &PluginContext,
    ) -> scarab_plugin_api::Result<Action> {
        self.counter += input.len();
        Ok(Action::Continue)
    }

    async fn on_resize(
        &mut self,
        cols: u16,
        rows: u16,
        _ctx: &PluginContext,
    ) -> scarab_plugin_api::Result<()> {
        self.counter = (cols as usize) * (rows as usize);
        Ok(())
    }
}

// ============================================================================
// Benchmark Group 1: Plugin Loading
// ============================================================================

fn bench_plugin_loading_bytecode(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_loading_bytecode");

    // Benchmark minimal bytecode loading
    group.bench_function("minimal_bytecode", |b| {
        let bytecode = create_minimal_bytecode();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&bytecode).unwrap();
        temp_file.flush().unwrap();
        let path = temp_file.path().to_path_buf();

        b.iter(|| {
            let plugin = FusabiBytecodePlugin::load(&path);
            black_box(plugin)
        });
    });

    // Benchmark complex bytecode loading
    group.bench_function("complex_bytecode", |b| {
        let bytecode = create_complex_bytecode();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&bytecode).unwrap();
        temp_file.flush().unwrap();
        let path = temp_file.path().to_path_buf();

        b.iter(|| {
            let plugin = FusabiBytecodePlugin::load(&path);
            black_box(plugin)
        });
    });

    group.finish();
}

fn bench_plugin_loading_script(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_loading_script");

    let scripts = vec![
        ("minimal", "let x = 42"),
        ("with_function", "let add x y = x + y\nlet result = add 10 32"),
        (
            "with_hooks",
            "let on_load = fun _u -> ()\nlet on_output = fun line -> true",
        ),
        (
            "complex",
            r#"
let counter = ref 0
let increment () = counter := !counter + 1
let on_load = fun _u -> ()
let on_output = fun line ->
    increment ()
    true
let on_input = fun data -> true
"#,
        ),
    ];

    for (name, script) in scripts {
        group.bench_function(name, |b| {
            let mut temp_file = NamedTempFile::new().unwrap();
            temp_file.write_all(script.as_bytes()).unwrap();
            temp_file.flush().unwrap();
            let path = temp_file.path().to_path_buf();

            b.iter(|| {
                let plugin = FusabiScriptPlugin::load(&path);
                black_box(plugin)
            });
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark Group 2: Hook Dispatch Latency
// ============================================================================

fn bench_hook_dispatch_output(c: &mut Criterion) {
    let mut group = c.benchmark_group("hook_dispatch_output");

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let test_line = "Test output line with some content";

    // Baseline: no plugins
    group.bench_function("no_plugins", |b| {
        let mut manager = create_test_manager();
        b.to_async(&runtime).iter(|| async {
            let result = manager.dispatch_output(test_line).await;
            black_box(result)
        });
    });

    // Single no-op plugin
    group.bench_function("single_noop", |b| {
        let mut manager = create_test_manager();
        runtime.block_on(async {
            manager.register_plugin(Box::new(NoOpPlugin::new())).await.unwrap();
        });

        b.to_async(&runtime).iter(|| async {
            let result = manager.dispatch_output(test_line).await;
            black_box(result)
        });
    });

    // Single processing plugin
    group.bench_function("single_processing", |b| {
        let mut manager = create_test_manager();
        runtime.block_on(async {
            manager
                .register_plugin(Box::new(ProcessingPlugin::new()))
                .await
                .unwrap();
        });

        b.to_async(&runtime).iter(|| async {
            let result = manager.dispatch_output(test_line).await;
            black_box(result)
        });
    });

    group.finish();
}

fn bench_hook_dispatch_input(c: &mut Criterion) {
    let mut group = c.benchmark_group("hook_dispatch_input");

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let test_input = b"test input data\n";

    // Baseline: no plugins
    group.bench_function("no_plugins", |b| {
        let mut manager = create_test_manager();
        b.to_async(&runtime).iter(|| async {
            let result = manager.dispatch_input(test_input).await;
            black_box(result)
        });
    });

    // Single no-op plugin
    group.bench_function("single_noop", |b| {
        let mut manager = create_test_manager();
        runtime.block_on(async {
            manager.register_plugin(Box::new(NoOpPlugin::new())).await.unwrap();
        });

        b.to_async(&runtime).iter(|| async {
            let result = manager.dispatch_input(test_input).await;
            black_box(result)
        });
    });

    // Single processing plugin
    group.bench_function("single_processing", |b| {
        let mut manager = create_test_manager();
        runtime.block_on(async {
            manager
                .register_plugin(Box::new(ProcessingPlugin::new()))
                .await
                .unwrap();
        });

        b.to_async(&runtime).iter(|| async {
            let result = manager.dispatch_input(test_input).await;
            black_box(result)
        });
    });

    group.finish();
}

fn bench_hook_dispatch_resize(c: &mut Criterion) {
    let mut group = c.benchmark_group("hook_dispatch_resize");

    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Baseline: no plugins
    group.bench_function("no_plugins", |b| {
        let mut manager = create_test_manager();
        b.to_async(&runtime).iter(|| async {
            let result = manager.dispatch_resize(100, 50).await;
            black_box(result)
        });
    });

    // Single no-op plugin
    group.bench_function("single_noop", |b| {
        let mut manager = create_test_manager();
        runtime.block_on(async {
            manager.register_plugin(Box::new(NoOpPlugin::new())).await.unwrap();
        });

        b.to_async(&runtime).iter(|| async {
            let result = manager.dispatch_resize(100, 50).await;
            black_box(result)
        });
    });

    // Single processing plugin
    group.bench_function("single_processing", |b| {
        let mut manager = create_test_manager();
        runtime.block_on(async {
            manager
                .register_plugin(Box::new(ProcessingPlugin::new()))
                .await
                .unwrap();
        });

        b.to_async(&runtime).iter(|| async {
            let result = manager.dispatch_resize(100, 50).await;
            black_box(result)
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark Group 3: Multiple Plugin Chaining
// ============================================================================

fn bench_plugin_chaining(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_chaining");

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let test_line = "Test output line";

    for count in [1, 2, 5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::new("noop_plugins", count), count, |b, &count| {
            let mut manager = create_test_manager();
            runtime.block_on(async {
                for i in 0..count {
                    let mut plugin = NoOpPlugin::new();
                    plugin.metadata.name = format!("noop_{}", i);
                    manager.register_plugin(Box::new(plugin)).await.unwrap();
                }
            });

            b.to_async(&runtime).iter(|| async {
                let result = manager.dispatch_output(test_line).await;
                black_box(result)
            });
        });

        group.bench_with_input(
            BenchmarkId::new("processing_plugins", count),
            count,
            |b, &count| {
                let mut manager = create_test_manager();
                runtime.block_on(async {
                    for i in 0..count {
                        let mut plugin = ProcessingPlugin::new();
                        plugin.metadata.name = format!("processor_{}", i);
                        manager.register_plugin(Box::new(plugin)).await.unwrap();
                    }
                });

                b.to_async(&runtime).iter(|| async {
                    let result = manager.dispatch_output(test_line).await;
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark Group 4: VM Execution Overhead
// ============================================================================

fn bench_vm_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm_execution");

    // Direct VM execution (no plugin overhead)
    group.bench_function("direct_minimal", |b| {
        let bytecode = create_minimal_bytecode();
        b.iter(|| {
            let chunk = fusabi_vm::deserialize_chunk(&bytecode).unwrap();
            let mut vm = Vm::new();
            let result = vm.execute(chunk);
            black_box(result)
        });
    });

    group.bench_function("direct_complex", |b| {
        let bytecode = create_complex_bytecode();
        b.iter(|| {
            let chunk = fusabi_vm::deserialize_chunk(&bytecode).unwrap();
            let mut vm = Vm::new();
            let result = vm.execute(chunk);
            black_box(result)
        });
    });

    // VM execution through plugin adapter
    group.bench_function("adapter_minimal", |b| {
        let bytecode = create_minimal_bytecode();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&bytecode).unwrap();
        temp_file.flush().unwrap();

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let mut plugin = FusabiBytecodePlugin::load(temp_file.path()).unwrap();
        let state = Arc::new(parking_lot::Mutex::new(PluginSharedState::new(80, 24)));
        let ctx = PluginContext::new(Default::default(), state, "test");

        b.to_async(&runtime).iter(|| async {
            let result = plugin.on_output("test", &ctx).await;
            black_box(result)
        });
    });

    group.finish();
}

fn bench_script_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("script_compilation");

    let scripts = vec![
        ("minimal", "let x = 42"),
        ("function", "let add x y = x + y"),
        ("complex", "let f x = let y = x + 10 in let z = y * 2 in z - 5"),
    ];

    for (name, script) in scripts {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut lexer = Lexer::new(script);
                let tokens = lexer.tokenize().unwrap();
                let mut parser = Parser::new(tokens);
                let program = parser.parse_program().unwrap();
                let chunk = Compiler::compile_program(&program).unwrap();
                black_box(chunk)
            });
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark Group 5: Throughput Testing
// ============================================================================

fn bench_output_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("output_throughput");

    let runtime = tokio::runtime::Runtime::new().unwrap();

    for line_count in [10, 100, 1000].iter() {
        let lines = generate_output_lines(*line_count);

        group.throughput(Throughput::Elements(*line_count as u64));

        // Baseline: no plugins
        group.bench_with_input(
            BenchmarkId::new("no_plugins", line_count),
            line_count,
            |b, _| {
                let mut manager = create_test_manager();
                b.to_async(&runtime).iter(|| async {
                    for line in &lines {
                        let _ = manager.dispatch_output(line).await;
                    }
                });
            },
        );

        // Single plugin
        group.bench_with_input(
            BenchmarkId::new("single_plugin", line_count),
            line_count,
            |b, _| {
                let mut manager = create_test_manager();
                runtime.block_on(async {
                    manager
                        .register_plugin(Box::new(ProcessingPlugin::new()))
                        .await
                        .unwrap();
                });

                b.to_async(&runtime).iter(|| async {
                    for line in &lines {
                        let _ = manager.dispatch_output(line).await;
                    }
                });
            },
        );

        // Five plugins
        group.bench_with_input(
            BenchmarkId::new("five_plugins", line_count),
            line_count,
            |b, _| {
                let mut manager = create_test_manager();
                runtime.block_on(async {
                    for i in 0..5 {
                        let mut plugin = ProcessingPlugin::new();
                        plugin.metadata.name = format!("processor_{}", i);
                        manager.register_plugin(Box::new(plugin)).await.unwrap();
                    }
                });

                b.to_async(&runtime).iter(|| async {
                    for line in &lines {
                        let _ = manager.dispatch_output(line).await;
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_input_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("input_throughput");

    let runtime = tokio::runtime::Runtime::new().unwrap();

    for input_count in [10, 100, 1000].iter() {
        let inputs = generate_input_data(*input_count);

        group.throughput(Throughput::Elements(*input_count as u64));

        // Baseline: no plugins
        group.bench_with_input(
            BenchmarkId::new("no_plugins", input_count),
            input_count,
            |b, _| {
                let mut manager = create_test_manager();
                b.to_async(&runtime).iter(|| async {
                    for input in &inputs {
                        let _ = manager.dispatch_input(input).await;
                    }
                });
            },
        );

        // Single plugin
        group.bench_with_input(
            BenchmarkId::new("single_plugin", input_count),
            input_count,
            |b, _| {
                let mut manager = create_test_manager();
                runtime.block_on(async {
                    manager
                        .register_plugin(Box::new(ProcessingPlugin::new()))
                        .await
                        .unwrap();
                });

                b.to_async(&runtime).iter(|| async {
                    for input in &inputs {
                        let _ = manager.dispatch_input(input).await;
                    }
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark Group 6: Thread-Local VM Cache
// ============================================================================

fn bench_vm_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm_cache");

    let script = "let on_output = fun line -> true";

    // Benchmark cache hit (same thread)
    group.bench_function("cache_hit", |b| {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(script.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let mut plugin = FusabiScriptPlugin::load(temp_file.path()).unwrap();
        let state = Arc::new(parking_lot::Mutex::new(PluginSharedState::new(80, 24)));
        let ctx = PluginContext::new(Default::default(), state, "test");

        // Warm up cache
        runtime.block_on(async {
            let _ = plugin.on_output("warmup", &ctx).await;
        });

        b.to_async(&runtime).iter(|| async {
            let result = plugin.on_output("test", &ctx).await;
            black_box(result)
        });
    });

    // Benchmark without cache (reload each time)
    group.bench_function("no_cache", |b| {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(script.as_bytes()).unwrap();
        temp_file.flush().unwrap();
        let path = temp_file.path().to_path_buf();

        let runtime = tokio::runtime::Runtime::new().unwrap();

        b.iter(|| {
            // Reload plugin each time (simulates cache miss)
            let mut plugin = FusabiScriptPlugin::load(&path).unwrap();
            let state = Arc::new(parking_lot::Mutex::new(PluginSharedState::new(80, 24)));
            let ctx = PluginContext::new(Default::default(), state, "test");

            runtime.block_on(async {
                let result = plugin.on_output("test", &ctx).await;
                black_box(result)
            });
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark Group 7: Realistic Workload Simulation
// ============================================================================

fn bench_realistic_terminal_session(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_workload");
    group.sample_size(20); // Fewer samples for long-running benchmarks
    group.measurement_time(Duration::from_secs(10));

    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Simulate a realistic terminal session:
    // - 100 output lines
    // - 20 input events
    // - 2 resize events
    let output_lines = generate_output_lines(100);
    let input_events = generate_input_data(20);

    // No plugins
    group.bench_function("no_plugins", |b| {
        b.to_async(&runtime).iter(|| async {
            let mut manager = create_test_manager();

            // Process outputs
            for line in &output_lines {
                let _ = manager.dispatch_output(line).await;
            }

            // Process inputs
            for input in &input_events {
                let _ = manager.dispatch_input(input).await;
            }

            // Process resizes
            let _ = manager.dispatch_resize(100, 50).await;
            let _ = manager.dispatch_resize(120, 60).await;
        });
    });

    // With 3 plugins
    group.bench_function("three_plugins", |b| {
        b.to_async(&runtime).iter(|| async {
            let mut manager = create_test_manager();

            // Register plugins
            for i in 0..3 {
                let mut plugin = ProcessingPlugin::new();
                plugin.metadata.name = format!("processor_{}", i);
                manager.register_plugin(Box::new(plugin)).await.unwrap();
            }

            // Process outputs
            for line in &output_lines {
                let _ = manager.dispatch_output(line).await;
            }

            // Process inputs
            for input in &input_events {
                let _ = manager.dispatch_input(input).await;
            }

            // Process resizes
            let _ = manager.dispatch_resize(100, 50).await;
            let _ = manager.dispatch_resize(120, 60).await;
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark Configuration
// ============================================================================

criterion_group!(
    loading_benches,
    bench_plugin_loading_bytecode,
    bench_plugin_loading_script
);

criterion_group!(
    dispatch_benches,
    bench_hook_dispatch_output,
    bench_hook_dispatch_input,
    bench_hook_dispatch_resize
);

criterion_group!(chaining_benches, bench_plugin_chaining);

criterion_group!(
    vm_benches,
    bench_vm_execution,
    bench_script_compilation,
    bench_vm_cache_performance
);

criterion_group!(
    throughput_benches,
    bench_output_throughput,
    bench_input_throughput
);

criterion_group!(workload_benches, bench_realistic_terminal_session);

criterion_main!(
    loading_benches,
    dispatch_benches,
    chaining_benches,
    vm_benches,
    throughput_benches,
    workload_benches
);
