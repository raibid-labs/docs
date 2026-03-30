//! Text rendering performance benchmarks

use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, SwashCache};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;

fn create_font_system() -> FontSystem {
    FontSystem::new()
}

fn bench_text_shaping(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_shaping");
    let mut font_system = create_font_system();

    for text_len in [10, 100, 1000, 10000].iter() {
        let text = "a".repeat(*text_len);
        group.throughput(Throughput::Bytes(text.len() as u64));

        group.bench_with_input(BenchmarkId::from_parameter(text_len), text_len, |b, _| {
            let mut buffer = Buffer::new(&mut font_system, Metrics::new(14.0, 20.0));

            b.iter(|| {
                buffer.set_text(
                    &mut font_system,
                    &text,
                    Attrs::new(),
                    cosmic_text::Shaping::Advanced,
                );
                buffer.shape_until_scroll(&mut font_system, false);
                black_box(&buffer);
            });
        });
    }

    group.finish();
}

fn bench_line_wrapping(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_wrapping");
    let mut font_system = create_font_system();

    for width in [40.0, 80.0, 120.0, 200.0].iter() {
        let text = "The quick brown fox jumps over the lazy dog. ".repeat(100);

        group.bench_with_input(BenchmarkId::from_parameter(width), width, |b, &width| {
            let mut buffer = Buffer::new(&mut font_system, Metrics::new(14.0, 20.0));
            buffer.set_size(&mut font_system, Some(width), None);

            b.iter(|| {
                buffer.set_text(
                    &mut font_system,
                    &text,
                    Attrs::new(),
                    cosmic_text::Shaping::Advanced,
                );
                buffer.shape_until_scroll(&mut font_system, false);
                black_box(&buffer);
            });
        });
    }

    group.finish();
}

fn bench_unicode_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("unicode_processing");
    let mut font_system = create_font_system();

    let test_texts = vec![
        ("ascii", "Hello World! ".repeat(100)),
        ("emoji", "😀🎉🚀💻 ".repeat(100)),
        ("cjk", "你好世界 こんにちは世界 ".repeat(100)),
        ("arabic", "مرحبا بالعالم ".repeat(100)),
        ("mixed", "Hello 😀 世界 مرحبا ".repeat(100)),
    ];

    for (name, text) in test_texts {
        group.throughput(Throughput::Bytes(text.len() as u64));

        group.bench_with_input(BenchmarkId::from_parameter(name), &text, |b, text| {
            let mut buffer = Buffer::new(&mut font_system, Metrics::new(14.0, 20.0));

            b.iter(|| {
                buffer.set_text(
                    &mut font_system,
                    text,
                    Attrs::new(),
                    cosmic_text::Shaping::Advanced,
                );
                buffer.shape_until_scroll(&mut font_system, false);
                black_box(&buffer);
            });
        });
    }

    group.finish();
}

fn bench_glyph_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("glyph_cache");
    let mut font_system = create_font_system();
    let mut cache = SwashCache::new();

    // Generate text with varying character repetition
    let texts = vec![
        (
            "unique_chars",
            (0..1000)
                .map(|i| ((i % 94) + 33) as u8 as char)
                .collect::<String>(),
        ),
        ("repeated_chars", "abcdefghij".repeat(100)),
        ("single_char", "a".repeat(1000)),
    ];

    for (name, text) in texts {
        group.bench_with_input(BenchmarkId::from_parameter(name), &text, |b, text| {
            let mut buffer = Buffer::new(&mut font_system, Metrics::new(14.0, 20.0));
            buffer.set_text(
                &mut font_system,
                text,
                Attrs::new(),
                cosmic_text::Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut font_system, false);

            b.iter(|| {
                // Clear cache to measure cold performance
                cache = SwashCache::new();

                for run in buffer.layout_runs() {
                    for glyph in run.glyphs.iter() {
                        cache.get_image(&mut font_system, glyph.cache_key);
                    }
                }
                black_box(&cache);
            });
        });
    }

    group.finish();
}

fn bench_scrolling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scrolling");
    let mut font_system = create_font_system();

    for lines in [100, 500, 1000, 5000].iter() {
        let text = (0..*lines)
            .map(|i| format!("[{:06}] This is a log line with some content\n", i))
            .collect::<String>();

        group.throughput(Throughput::Elements(*lines as u64));

        group.bench_with_input(BenchmarkId::from_parameter(lines), lines, |b, _| {
            let mut buffer = Buffer::new(&mut font_system, Metrics::new(14.0, 20.0));
            buffer.set_text(
                &mut font_system,
                &text,
                Attrs::new(),
                cosmic_text::Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut font_system, false);

            b.iter(|| {
                // Simulate scrolling through the buffer
                for i in 0..10 {
                    buffer.set_scroll(i * 10);
                    buffer.shape_until_scroll(&mut font_system, false);
                }
                black_box(&buffer);
            });
        });
    }

    group.finish();
}

fn bench_syntax_highlighting(c: &mut Criterion) {
    let mut group = c.benchmark_group("syntax_highlighting");
    let mut font_system = create_font_system();

    let code = r#"
fn main() {
    let mut x = 42;
    for i in 0..100 {
        println!("Value: {}", i + x);
        x = x * 2;
    }
    // This is a comment
    let s = "This is a string";
}
"#
    .repeat(10);

    // Simulate different highlighting scenarios
    let scenarios = vec![
        ("no_highlighting", vec![Attrs::new()]),
        (
            "basic_highlighting",
            vec![
                Attrs::new().color(cosmic_text::Color::rgb(255, 0, 0)), // Keywords
                Attrs::new().color(cosmic_text::Color::rgb(0, 255, 0)), // Strings
                Attrs::new().color(cosmic_text::Color::rgb(0, 0, 255)), // Numbers
                Attrs::new().color(cosmic_text::Color::rgb(128, 128, 128)), // Comments
            ],
        ),
        (
            "complex_highlighting",
            (0..20)
                .map(|i| {
                    Attrs::new()
                        .color(cosmic_text::Color::rgb(
                            (i * 12) as u8,
                            (i * 8) as u8,
                            (i * 16) as u8,
                        ))
                        .weight(if i % 2 == 0 {
                            cosmic_text::Weight::BOLD
                        } else {
                            cosmic_text::Weight::NORMAL
                        })
                })
                .collect(),
        ),
    ];

    for (name, attrs_list) in scenarios {
        group.bench_with_input(BenchmarkId::from_parameter(name), &code, |b, code| {
            let mut buffer = Buffer::new(&mut font_system, Metrics::new(14.0, 20.0));

            b.iter(|| {
                // Apply different attributes to different parts of the text
                let attrs = attrs_list[code.len() % attrs_list.len()].clone();
                buffer.set_text(
                    &mut font_system,
                    code,
                    attrs,
                    cosmic_text::Shaping::Advanced,
                );
                buffer.shape_until_scroll(&mut font_system, false);
                black_box(&buffer);
            });
        });
    }

    group.finish();
}

fn bench_font_fallback(c: &mut Criterion) {
    let mut group = c.benchmark_group("font_fallback");
    let mut font_system = create_font_system();

    // Text that requires font fallback
    let texts = vec![
        ("latin_only", "Hello World ".repeat(100)),
        ("with_emoji", "Hello 😀 World 🎉 ".repeat(100)),
        ("multilingual", "Hello 你好 مرحبا こんにちは ".repeat(100)),
        ("symbols", "→ ← ↑ ↓ ⇒ ⇐ ⇑ ⇓ ".repeat(100)),
    ];

    for (name, text) in texts {
        group.throughput(Throughput::Bytes(text.len() as u64));

        group.bench_with_input(BenchmarkId::from_parameter(name), &text, |b, text| {
            let mut buffer = Buffer::new(&mut font_system, Metrics::new(14.0, 20.0));

            b.iter(|| {
                buffer.set_text(
                    &mut font_system,
                    text,
                    Attrs::new(),
                    cosmic_text::Shaping::Advanced,
                );
                buffer.shape_until_scroll(&mut font_system, false);
                black_box(&buffer);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_text_shaping,
    bench_line_wrapping,
    bench_unicode_processing,
    bench_glyph_cache,
    bench_scrolling,
    bench_syntax_highlighting,
    bench_font_fallback
);
criterion_main!(benches);
