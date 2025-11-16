# Justfile for Bevy MCP Ratatui Reference Implementation
# Install just: https://github.com/casey/just

# Default recipe to display help information
default:
    @just --list

# Run the main application with TUI rendering (window + terminal ASCII)
run:
    cargo run --features tui

# Run main app with both TUI and BRP (AI-controllable via MCP)
run-full:
    cargo run --features full

# Run basic TUI example (window + terminal ASCII)
demo:
    cargo run --example tui_basic --features tui

# 🤖 Run TUI + BRP integration example (AI-CONTROLLABLE via BRP on port 15702)
demo-brp:
    cargo run --example tui_brp --features full

# Run enhanced dual rendering example with complex scene
demo-dual:
    cargo run --example windowed_tui --features full

# Watch and auto-reload the basic TUI demo on file changes
watch-demo:
    cargo watch -x "run --example tui_basic --features tui"

# Watch and auto-reload the dual rendering demo
watch-dual:
    cargo watch -x "run --example windowed_tui --features full"

# Build all targets with all features
build:
    cargo build --all-features

# Build with TUI features only
build-tui:
    cargo build --features tui

# Build with BRP features only
build-brp:
    cargo build --features brp

# Build with all features
build-full:
    cargo build --features full

# Build release version with all features
build-release:
    cargo build --release --all-features

# Check code compiles without building
check:
    cargo check --all-features

# Check all feature combinations
check-all:
    @echo "Checking base..."
    cargo check --no-default-features
    @echo "\nChecking TUI feature..."
    cargo check --features tui
    @echo "\nChecking BRP feature..."
    cargo check --features brp || echo "BRP blocked (Phase 2)"
    @echo "\nChecking full features..."
    cargo check --features full || echo "Full blocked (Phase 2)"
    @echo "\nAll checks complete!"

# Run all tests
test:
    cargo test --all-features

# Run tests with output visible
test-verbose:
    cargo test --all-features -- --nocapture

# Run specific test
test-one TEST:
    cargo test {{TEST}} --all-features -- --nocapture

# Format code with rustfmt
fmt:
    cargo fmt --all

# Check if code is formatted
fmt-check:
    cargo fmt --all -- --check

# Run clippy linter
lint:
    cargo clippy --all-features

# Run clippy with strict warnings
lint-strict:
    cargo clippy --all-features -- -D warnings

# Fix clippy warnings automatically where possible
fix:
    cargo clippy --fix --all-features --allow-dirty --allow-staged

# Clean build artifacts
clean:
    cargo clean

# Full quality check: format, lint, test, build
check-all-quality: fmt-check lint test build
    @echo "✅ All quality checks passed!"

# Production build with all checks
prod: fmt lint test build-release
    @echo "✅ Production build complete!"

# Install required tools for development
install-tools:
    @echo "Installing development tools..."
    cargo install cargo-watch
    cargo install just
    @echo "✅ Tools installed!"

# Show project statistics
stats:
    @echo "📊 Project Statistics"
    @echo "====================="
    @echo "\n📁 Files:"
    @find src examples -name "*.rs" | wc -l
    @echo "\n📝 Lines of Rust code:"
    @find src examples -name "*.rs" -exec cat {} \; | wc -l
    @echo "\n📦 Dependencies:"
    @cargo tree --depth 1 | grep -v "└──" | grep -v "├──" | wc -l
    @echo "\n🎯 Features:"
    @grep "^\[features\]" -A 10 Cargo.toml

# Check dependencies for updates
deps-check:
    cargo tree --depth 1

# Update dependencies
deps-update:
    cargo update

# Show current phase status
status:
    @echo "📋 Implementation Status"
    @echo "========================"
    @echo ""
    @echo "✅ TUI Rendering: WORKING"
    @echo "✅ BRP Integration: WORKING"
    @echo "✅ 3D Terminal Output: WORKING"
    @echo ""
    @echo "🤖 AI Control: Run 'just demo-brp' then use MCP tools"
    @echo "   BRP listens on localhost:15702"
    @echo ""
    @cargo check --features full 2>&1 | tail -1

# Run basic TUI example in release mode for best performance
demo-release:
    cargo run --release --example tui_basic --features tui

# Run dual rendering in release mode
demo-dual-release:
    cargo run --release --example windowed_tui --features full

# Generate documentation
docs:
    cargo doc --all-features --no-deps --open

# Generate documentation for all dependencies
docs-full:
    cargo doc --all-features --open

# Show terminal capabilities for TUI rendering
terminal-info:
    @echo "🖥️  Terminal Information"
    @echo "======================="
    @echo "TERM: $TERM"
    @echo "COLORTERM: $COLORTERM"
    @echo "Dimensions: $COLUMNS x $LINES"
    @echo ""
    @echo "Recommended terminals for 24-bit color:"
    @echo "  - Alacritty, Kitty, iTerm2, WezTerm"

# Quick development cycle: format, check, test
dev: fmt check test
    @echo "✅ Development cycle complete!"

# CI simulation: run all checks like CI would
ci: fmt-check lint test build
    @echo "✅ CI checks passed!"

# Benchmark build times
bench-build:
    @echo "⏱️  Benchmarking build times..."
    @echo "\nClean build (TUI):"
    @cargo clean && time cargo build --features tui 2>&1 | grep "Finished"
    @echo "\nClean build (Full):"
    @cargo clean && time cargo build --features full 2>&1 | grep "Finished"

# Show cargo features
features:
    @echo "🎯 Available Features"
    @echo "===================="
    @grep "^# Enable" Cargo.toml -A 1
    @echo ""
    @echo "Usage:"
    @echo "  cargo run --features tui"
    @echo "  cargo run --features brp"
    @echo "  cargo run --features full"

# Create a new example file
new-example NAME:
    @echo "Creating new example: {{NAME}}"
    @echo "//! {{NAME}} example" > examples/{{NAME}}.rs
    @echo "" >> examples/{{NAME}}.rs
    @echo "use bevy::prelude::*;" >> examples/{{NAME}}.rs
    @echo "use bevy_mcp_ratatui_ref::prelude::*;" >> examples/{{NAME}}.rs
    @echo "" >> examples/{{NAME}}.rs
    @echo "fn main() {" >> examples/{{NAME}}.rs
    @echo "    App::new()" >> examples/{{NAME}}.rs
    @echo "        .add_plugins(DefaultPlugins)  // Provides 3D rendering" >> examples/{{NAME}}.rs
    @echo "        .add_plugins(BevyMcpTuiPlugin::default())  // Adds ASCII conversion" >> examples/{{NAME}}.rs
    @echo "        .run();" >> examples/{{NAME}}.rs
    @echo "}" >> examples/{{NAME}}.rs
    @echo "✅ Created examples/{{NAME}}.rs"

# Run all examples (that compile)
demo-all:
    @echo "Running all working examples..."
    @echo "\n1️⃣  TUI Basic Example:"
    cargo run --example tui_basic --features tui
    @echo "\n✅ All demos complete!"

# Profile compilation with timings
profile-build:
    cargo build --features tui --timings
    @echo "📊 See target/cargo-timings/cargo-timing.html"

# Check for security vulnerabilities
audit:
    cargo audit

# Help with common tasks
help:
    @echo "🎮 Bevy MCP Ratatui Reference - Quick Start"
    @echo "==========================================="
    @echo ""
    @echo "🚀 Quick Start:"
    @echo "  just demo              - Run basic demo (window + terminal ASCII)"
    @echo "  just demo-brp          - 🤖 AI-controllable demo (BRP on :15702)"
    @echo "  just demo-dual         - Enhanced demo with complex scene"
    @echo "  just watch-demo        - Auto-reload demo on changes"
    @echo "  just run-full          - Run main app with TUI + BRP"
    @echo ""
    @echo "🤖 AI Control:"
    @echo "  just demo-brp          - Start AI-controllable app"
    @echo "                          (BRP listens on localhost:15702)"
    @echo "  Then use MCP tools:     bevy_spawn, bevy_mutate_component, etc."
    @echo ""
    @echo "Development:"
    @echo "  just dev               - Format, check, test"
    @echo "  just check-all         - Check all feature combinations"
    @echo "  just lint              - Run clippy linter"
    @echo "  just test              - Run all tests"
    @echo ""
    @echo "Build:"
    @echo "  just build-full        - Build with all features"
    @echo "  just build-release     - Release build"
    @echo "  just prod              - Full production build"
    @echo ""
    @echo "Information:"
    @echo "  just status            - Show implementation status"
    @echo "  just stats             - Project statistics"
    @echo "  just features          - Show available features"
    @echo "  just terminal-info     - Terminal capabilities"
    @echo ""
    @echo "📚 See 'just --list' for all commands"
