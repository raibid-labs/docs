//! VTE parsing performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use scarab_daemon::vte_optimized::{BatchProcessor, OptimizedPerformer};
use vte::{Parser, Perform};

/// Test performer that processes VTE sequences
struct BenchPerformer {
    operations: usize,
}

impl BenchPerformer {
    fn new() -> Self {
        Self { operations: 0 }
    }

    fn reset(&mut self) {
        self.operations = 0;
    }
}

impl Perform for BenchPerformer {
    fn print(&mut self, _c: char) {
        self.operations += 1;
    }

    fn execute(&mut self, _byte: u8) {
        self.operations += 1;
    }

    fn csi_dispatch(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        self.operations += 1;
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        self.operations += 1;
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        self.operations += 1;
    }

    fn put(&mut self, _byte: u8) {
        self.operations += 1;
    }

    fn unhook(&mut self) {
        self.operations += 1;
    }
}

fn generate_text_data(size: usize) -> Vec<u8> {
    // Generate plain text data
    let text = "The quick brown fox jumps over the lazy dog. ";
    text.bytes().cycle().take(size).collect()
}

fn generate_ansi_colors(count: usize) -> Vec<u8> {
    // Generate ANSI color escape sequences
    let mut data = Vec::new();
    for i in 0..count {
        let color = 30 + (i % 8);
        data.extend_from_slice(format!("\x1b[{}mColor{}\x1b[0m ", color, i).as_bytes());
    }
    data
}

fn generate_cursor_movements(count: usize) -> Vec<u8> {
    // Generate cursor movement sequences
    let mut data = Vec::new();
    for i in 0..count {
        match i % 4 {
            0 => data.extend_from_slice(b"\x1b[A"), // Up
            1 => data.extend_from_slice(b"\x1b[B"), // Down
            2 => data.extend_from_slice(b"\x1b[C"), // Right
            _ => data.extend_from_slice(b"\x1b[D"), // Left
        }
        data.push(b'X'); // Character after movement
    }
    data
}

fn generate_mixed_sequences(size: usize) -> Vec<u8> {
    // Generate a realistic mix of text and escape sequences
    let mut data = Vec::new();
    let mut remaining = size;
    let mut i = 0;

    while remaining > 0 {
        match i % 5 {
            0 => {
                // Plain text
                let text = "Hello, World! ";
                let bytes = text.as_bytes();
                if remaining >= bytes.len() {
                    data.extend_from_slice(bytes);
                    remaining -= bytes.len();
                } else {
                    data.extend_from_slice(&bytes[..remaining]);
                    remaining = 0;
                }
            }
            1 => {
                // Color change
                let seq = format!("\x1b[{}m", 30 + (i % 8)).into_bytes();
                if remaining >= seq.len() {
                    data.extend_from_slice(&seq);
                    remaining -= seq.len();
                } else {
                    break;
                }
            }
            2 => {
                // Cursor position
                let seq = format!("\x1b[{};{}H", (i % 24) + 1, (i % 80) + 1).into_bytes();
                if remaining >= seq.len() {
                    data.extend_from_slice(&seq);
                    remaining -= seq.len();
                } else {
                    break;
                }
            }
            3 => {
                // Clear line
                let seq = b"\x1b[2K";
                if remaining >= seq.len() {
                    data.extend_from_slice(seq);
                    remaining -= seq.len();
                } else {
                    break;
                }
            }
            _ => {
                // Tab and newline
                let seq = b"\t\n";
                if remaining >= seq.len() {
                    data.extend_from_slice(seq);
                    remaining -= seq.len();
                } else {
                    break;
                }
            }
        }
        i += 1;
    }

    data
}

fn generate_scrollback_buffer(lines: usize) -> Vec<u8> {
    // Generate data simulating scrollback buffer content
    let mut data = Vec::new();
    for i in 0..lines {
        data.extend_from_slice(format!("[{:06}] Log line with some content here\n", i).as_bytes());
        if i % 10 == 0 {
            // Add some color every 10 lines
            data.extend_from_slice(
                format!("\x1b[31m[ERROR]\x1b[0m Error at line {}\n", i).as_bytes(),
            );
        }
    }
    data
}

fn generate_repetitive_sequences(count: usize) -> Vec<u8> {
    // Generate highly repetitive sequences (best case for caching)
    let mut data = Vec::new();
    let sequences = [
        b"\x1b[31m",   // Red
        b"\x1b[32m",   // Green
        b"\x1b[33m",   // Yellow
        b"\x1b[0m",    // Reset
    ];

    for i in 0..count {
        data.extend_from_slice(sequences[i % sequences.len()]);
        data.extend_from_slice(b"Text ");
    }
    data
}

fn bench_plain_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("vte_plain_text");

    for size in [100, 1000, 10000, 100000].iter() {
        let data = generate_text_data(*size);
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let mut parser = Parser::new();
            let mut performer = BenchPerformer::new();

            b.iter(|| {
                performer.reset();
                for byte in &data {
                    parser.advance(&mut performer, *byte);
                }
                black_box(&performer.operations);
            });
        });
    }

    group.finish();
}

fn bench_ansi_colors(c: &mut Criterion) {
    let mut group = c.benchmark_group("vte_ansi_colors");

    for count in [10, 100, 1000, 10000].iter() {
        let data = generate_ansi_colors(*count);
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, _| {
            let mut parser = Parser::new();
            let mut performer = BenchPerformer::new();

            b.iter(|| {
                performer.reset();
                for byte in &data {
                    parser.advance(&mut performer, *byte);
                }
                black_box(&performer.operations);
            });
        });
    }

    group.finish();
}

fn bench_cursor_movements(c: &mut Criterion) {
    let mut group = c.benchmark_group("vte_cursor");

    for count in [100, 1000, 10000].iter() {
        let data = generate_cursor_movements(*count);
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, _| {
            let mut parser = Parser::new();
            let mut performer = BenchPerformer::new();

            b.iter(|| {
                performer.reset();
                for byte in &data {
                    parser.advance(&mut performer, *byte);
                }
                black_box(&performer.operations);
            });
        });
    }

    group.finish();
}

fn bench_mixed_sequences(c: &mut Criterion) {
    let mut group = c.benchmark_group("vte_mixed");

    for size in [1000, 10000, 100000].iter() {
        let data = generate_mixed_sequences(*size);
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let mut parser = Parser::new();
            let mut performer = BenchPerformer::new();

            b.iter(|| {
                performer.reset();
                for byte in &data {
                    parser.advance(&mut performer, *byte);
                }
                black_box(&performer.operations);
            });
        });
    }

    group.finish();
}

fn bench_scrollback(c: &mut Criterion) {
    let mut group = c.benchmark_group("vte_scrollback");

    for lines in [100, 1000, 10000].iter() {
        let data = generate_scrollback_buffer(*lines);
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(lines), lines, |b, _| {
            let mut parser = Parser::new();
            let mut performer = BenchPerformer::new();

            b.iter(|| {
                performer.reset();
                for byte in &data {
                    parser.advance(&mut performer, *byte);
                }
                black_box(&performer.operations);
            });
        });
    }

    group.finish();
}

fn bench_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("vte_batch");

    for chunk_size in [1, 10, 100, 1000, 4096].iter() {
        let data = generate_mixed_sequences(100000);
        group.bench_with_input(
            BenchmarkId::from_parameter(chunk_size),
            chunk_size,
            |b, &chunk_size| {
                let mut parser = Parser::new();
                let mut performer = BenchPerformer::new();

                b.iter(|| {
                    performer.reset();
                    for chunk in data.chunks(chunk_size) {
                        for byte in chunk {
                            parser.advance(&mut performer, *byte);
                        }
                    }
                    black_box(&performer.operations);
                });
            },
        );
    }

    group.finish();
}

// NEW BENCHMARKS FOR CACHE PERFORMANCE

fn bench_cache_baseline_vs_optimized(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_comparison");

    for size in [1000, 10000, 50000].iter() {
        let data = generate_ansi_colors(*size);

        // Baseline: Standard VTE parser (no cache)
        group.bench_with_input(
            BenchmarkId::new("baseline", size),
            size,
            |b, _| {
                let mut parser = Parser::new();
                let mut performer = BenchPerformer::new();

                b.iter(|| {
                    performer.reset();
                    for byte in &data {
                        parser.advance(&mut performer, *byte);
                    }
                    black_box(&performer.operations);
                });
            },
        );

        // Optimized: With LRU cache
        group.bench_with_input(
            BenchmarkId::new("cached", size),
            size,
            |b, _| {
                let mut processor = BatchProcessor::new();

                b.iter(|| {
                    processor.reset_cache_stats();
                    processor.add_data(&data);
                    processor.flush();
                    black_box(processor.cache_stats());
                });
            },
        );
    }

    group.finish();
}

fn bench_cache_hit_rates(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hit_rate");

    // Test different cache scenarios
    let scenarios = vec![
        ("repetitive", generate_repetitive_sequences(10000)),
        ("mixed", generate_mixed_sequences(10000)),
        ("colors", generate_ansi_colors(1000)),
    ];

    for (name, data) in scenarios {
        group.bench_function(name, |b| {
            let mut processor = BatchProcessor::new();

            b.iter(|| {
                processor.reset_cache_stats();
                processor.add_data(&data);
                processor.flush();
                let stats = processor.cache_stats();
                black_box(stats);
            });
        });
    }

    group.finish();
}

fn bench_cache_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_size");

    let data = generate_mixed_sequences(50000);

    for cache_size in [64, 128, 256, 512, 1024].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(cache_size),
            cache_size,
            |b, &cache_size| {
                let mut performer = OptimizedPerformer::with_cache_capacity(cache_size);
                let mut parser = Parser::new();

                b.iter(|| {
                    performer.reset_cache_stats();
                    performer.process_batch(&mut parser, &data);
                    black_box(performer.cache_stats());
                });
            },
        );
    }

    group.finish();
}

fn bench_real_world_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world");

    // Scenario 1: Git diff output (lots of colors)
    let git_diff = {
        let mut data = Vec::new();
        for i in 0..1000 {
            if i % 3 == 0 {
                data.extend_from_slice(b"\x1b[32m+ Added line\x1b[0m\n");
            } else if i % 3 == 1 {
                data.extend_from_slice(b"\x1b[31m- Removed line\x1b[0m\n");
            } else {
                data.extend_from_slice(b"  Context line\n");
            }
        }
        data
    };

    // Scenario 2: Log file with timestamps and levels
    let log_output = {
        let mut data = Vec::new();
        for i in 0..1000 {
            let level = match i % 4 {
                0 => ("\x1b[34m[INFO]\x1b[0m", "Information message"),
                1 => ("\x1b[33m[WARN]\x1b[0m", "Warning message"),
                2 => ("\x1b[31m[ERROR]\x1b[0m", "Error message"),
                _ => ("\x1b[90m[DEBUG]\x1b[0m", "Debug message"),
            };
            data.extend_from_slice(
                format!("2024-01-01 12:00:{:02} {} {}\n", i % 60, level.0, level.1).as_bytes()
            );
        }
        data
    };

    // Scenario 3: Interactive shell with prompts
    let shell_output = {
        let mut data = Vec::new();
        for i in 0..500 {
            data.extend_from_slice(b"\x1b[32m\xE2\x9E\x9C\x1b[0m \x1b[36m~/code\x1b[0m \x1b[33m$\x1b[0m ls -la\n");
            data.extend_from_slice(format!("file{}.txt\n", i).as_bytes());
        }
        data
    };

    for (name, data) in [
        ("git_diff", git_diff),
        ("log_output", log_output),
        ("shell_output", shell_output),
    ] {
        // Baseline
        group.bench_with_input(
            BenchmarkId::new("baseline", name),
            &data,
            |b, data| {
                let mut parser = Parser::new();
                let mut performer = BenchPerformer::new();

                b.iter(|| {
                    performer.reset();
                    for byte in data {
                        parser.advance(&mut performer, *byte);
                    }
                    black_box(&performer.operations);
                });
            },
        );

        // Cached
        group.bench_with_input(
            BenchmarkId::new("cached", name),
            &data,
            |b, data| {
                let mut processor = BatchProcessor::new();

                b.iter(|| {
                    processor.reset_cache_stats();
                    processor.add_data(data);
                    processor.flush();
                    black_box(processor.cache_stats());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_plain_text,
    bench_ansi_colors,
    bench_cursor_movements,
    bench_mixed_sequences,
    bench_scrollback,
    bench_batch_processing,
    bench_cache_baseline_vs_optimized,
    bench_cache_hit_rates,
    bench_cache_sizes,
    bench_real_world_scenarios,
);
criterion_main!(benches);
