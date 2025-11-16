# Bevy MCP Reference - Justfile
# Utility commands for building and running the game

# Default recipe - show available commands
default:
    @just --list

# Build the project in debug mode
build:
    cargo build

# Build the project with BRP features
build-brp:
    cargo build --features brp

# Build the project in release mode
build-release:
    cargo build --release --features brp

# Run the main game (no BRP)
run:
    cargo run

# Run the main game with BRP enabled
run-brp:
    cargo run --features brp

# Run the main game with BRP in release mode
run-release:
    cargo run --release --features brp

# Run the basic scene example
example-basic:
    cargo run --example basic_scene

# Run the BRP demo example
demo:
    cargo run --example brp_demo --features brp

# Run the BRP demo in release mode (faster)
demo-release:
    cargo run --example brp_demo --features brp --release

# Run all tests
test:
    cargo test

# Run tests with all features
test-all:
    cargo test --all-features

# Check code without building
check:
    cargo check --all-features

# Format code with rustfmt
fmt:
    cargo fmt

# Check formatting without changing files
fmt-check:
    cargo fmt --check

# Run clippy linter
lint:
    cargo clippy --all-features

# Run clippy with strict warnings
lint-strict:
    cargo clippy --all-features -- -D warnings

# Clean build artifacts
clean:
    cargo clean

# Check BRP status (assumes brp_demo is running)
brp-status:
    @echo "Checking BRP status for brp_demo..."
    @# This would need the MCP tool running, shown as example

# Full check: format, lint, test, build
check-all: fmt lint test build-brp
    @echo "✅ All checks passed!"

# Development workflow: clean, build with BRP, run demo
dev: clean build-brp demo

# Production build: clean, format, lint, test, build release
prod: clean fmt lint test build-release
    @echo "✅ Production build complete!"

# Watch and rebuild on file changes (requires cargo-watch)
watch:
    cargo watch -x 'run --features brp'

# Watch and run demo on file changes (requires cargo-watch)
watch-demo:
    cargo watch -x 'run --example brp_demo --features brp'

# Install development dependencies
install-deps:
    cargo install cargo-watch
    @echo "✅ Development dependencies installed!"

# Show project information
info:
    @echo "Bevy MCP Reference Implementation"
    @echo "=================================="
    @echo "Bevy Version: 0.16"
    @echo "Features: BRP (Bevy Remote Protocol)"
    @echo "MCP Server: bevy_brp_extras"
    @echo "Port: 15702"
    @echo ""
    @cargo --version
    @rustc --version

# Generate documentation
docs:
    cargo doc --no-deps --open

# Generate documentation with all features
docs-all:
    cargo doc --all-features --no-deps --open
