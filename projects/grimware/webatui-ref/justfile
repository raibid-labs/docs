# webatui justfile
# Comprehensive task automation for Rust + WASM project

# Default: Show available commands
default:
    @just --list --unsorted

# === Build Commands ===

# Build project in debug mode
build:
    @echo "🔨 Building project (debug)..."
    cargo build

# Build project in release mode
build-release:
    @echo "🚀 Building project (release)..."
    cargo build --release

# Build WASM target
build-wasm:
    @echo "🌐 Building WASM..."
    nu scripts/build.nu --wasm

# Build WASM with wasm-pack
build-wasm-pack:
    @echo "📦 Building with wasm-pack..."
    wasm-pack build --target web --out-dir pkg

# Build WASM release optimized
build-wasm-release:
    @echo "🚀 Building WASM (release)..."
    wasm-pack build --target web --release --out-dir pkg

# === Run Commands ===


# Run (this is a library - use examples instead)
run *ARGS:
    @echo "⚠️  This is a library crate, not a binary."
    @echo "💡 Use 'just example <name>' to run an example:"
    @echo ""
    @just list-examples


# Run a specific example (this is a library crate)
run-example NAME *ARGS:
    @echo "📝 Running example: {{NAME}}..."
    cargo run --example {{NAME}} --features terminal {{ARGS}}

# Alias for run-example (shorter)
example NAME:
    @just run-example {{NAME}}

# List all available examples
list-examples:
    @echo "📋 Available examples:"
    @echo "  - basic         : Simple terminal UI example"
    @echo "  - dashboard     : Dashboard with system metrics"
    @echo "  - interactive   : Interactive UI with user input"
    @echo ""
    @echo "💡 Run with: just example <name>"
    @echo "   or:       cargo run --example <name> --features terminal"

# Serve WASM build with HTTP server
serve PORT="8080":
    @echo "🌐 Serving WASM on port {{PORT}}..."
    nu scripts/serve.nu --port {{PORT}}

# === Test Commands ===

# Run all tests
test:
    @echo "🧪 Running tests..."
    cargo test

# Run all tests with verbose output
test-all:
    @echo "🧪 Running all tests (verbose)..."
    cargo test -- --nocapture --test-threads=1

# Run tests for WASM target
test-wasm:
    @echo "🌐 Running WASM tests..."
    wasm-pack test --headless --firefox

# Run tests with coverage
test-coverage:
    @echo "📊 Running tests with coverage..."
    cargo tarpaulin --out Html --output-dir coverage

# Run specific test
test-one TEST:
    @echo "🧪 Running test: {{TEST}}..."
    cargo test {{TEST}} -- --nocapture

# === Check Commands ===

# Run cargo check
check:
    @echo "✅ Checking project..."
    cargo check

# Check WASM target
check-wasm:
    @echo "🌐 Checking WASM target..."
    cargo check --target wasm32-unknown-unknown

# Check all targets and features
check-all:
    @echo "✅ Checking all targets..."
    cargo check --all-targets --all-features

# === Format & Lint Commands ===

# Format code
fmt:
    @echo "🎨 Formatting code..."
    cargo fmt

# Check formatting
fmt-check:
    @echo "🎨 Checking formatting..."
    cargo fmt -- --check

# Run clippy lints
lint:
    @echo "🔍 Running clippy..."
    cargo clippy -- -D warnings

# Run clippy with strict lints
lint-strict:
    @echo "🔍 Running clippy (strict)..."
    cargo clippy --all-targets --all-features -- -D warnings -D clippy::all -D clippy::pedantic

# Fix clippy warnings automatically
lint-fix:
    @echo "🔧 Fixing clippy warnings..."
    cargo clippy --fix --allow-dirty --allow-staged

# === Clean Commands ===

# Clean build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    cargo clean

# Clean WASM artifacts
clean-wasm:
    @echo "🧹 Cleaning WASM artifacts..."
    rm -rf pkg target/wasm32-unknown-unknown

# Clean all artifacts including node_modules
clean-all:
    @echo "🧹 Deep cleaning..."
    cargo clean
    rm -rf pkg target node_modules

# === Watch Commands ===

# Watch and rebuild on changes
watch:
    @echo "👀 Watching for changes..."
    bacon --features terminal

# Watch and run tests on changes
watch-test:
    @echo "👀 Watching and testing..."
    bacon test --features terminal

# Watch and run specific example
# Uses bacon with custom job configuration
watch-example NAME:
    @echo "👀 Watching example: {{NAME}}..."
    @echo "💡 Running bacon with example-{{NAME}} job"
    bacon example-{{NAME}}

# Watch WASM build
watch-wasm:
    @echo "👀 Watching WASM build..."
    bacon check --target wasm32-unknown-unknown

# === Install Commands ===

# Install development dependencies
install-deps:
    @echo "📦 Installing dependencies..."
    @echo "⚠️  Using bacon instead of cargo-watch (Apple Silicon compatibility)..."
    cargo install bacon
    cargo install wasm-pack
    cargo install wasm-bindgen-cli
    cargo install basic-http-server

# Install WASM tools
install-wasm-tools:
    @echo "🌐 Installing WASM tools..."
    rustup target add wasm32-unknown-unknown
    cargo install wasm-pack
    cargo install wasm-bindgen-cli
    cargo install wasm-opt

# Install all tools
install-all: install-deps install-wasm-tools
    @echo "✅ All tools installed!"

# === Workflow Commands ===

# Development workflow: format, lint, test
dev: fmt lint test
    @echo "✅ Development checks passed!"

# Production workflow: format check, lint strict, test, build release
prod: fmt-check lint-strict test build-release
    @echo "✅ Production build ready!"

# CI workflow: all checks
ci: fmt-check lint-strict test-all check-all
    @echo "✅ CI checks passed!"

# WASM development workflow
dev-wasm: fmt lint check-wasm build-wasm
    @echo "✅ WASM development checks passed!"

# === Documentation Commands ===

# Build documentation
docs:
    @echo "📚 Building documentation..."
    cargo doc --no-deps

# Build and open documentation
docs-open:
    @echo "📚 Building and opening documentation..."
    cargo doc --no-deps --open

# Build documentation with private items
docs-private:
    @echo "📚 Building documentation (with private)..."
    cargo doc --no-deps --document-private-items

# === Stats & Info Commands ===

# Show project statistics
stats:
    @echo "📊 Project statistics:"
    @echo "Lines of code:"
    @tokei .
    @echo "\nDependencies:"
    @cargo tree --depth 1

# Show project status
status:
    @echo "📊 Project status:"
    @echo "\n🔧 Git status:"
    @git status -s
    @echo "\n📦 Outdated dependencies:"
    @cargo outdated || echo "Run 'cargo install cargo-outdated' to check dependencies"

# Show project info
info:
    @echo "ℹ️  Project information:"
    @echo "Cargo version: $(cargo --version)"
    @echo "Rust version: $(rustc --version)"
    @echo "Node version: $(node --version 2>/dev/null || echo 'Not installed')"
    @echo "Just version: $(just --version)"
    @echo "wasm-pack: $(wasm-pack --version 2>/dev/null || echo 'Not installed')"

# Show dependencies tree
deps:
    @echo "🌳 Dependency tree:"
    cargo tree

# Check for outdated dependencies
outdated:
    @echo "📦 Checking for outdated dependencies..."
    cargo outdated || echo "Install with: cargo install cargo-outdated"

# === WASM Specific Commands ===

# Build WASM for web target
wasm-web:
    @echo "🌐 Building WASM for web..."
    wasm-pack build --target web

# Build WASM for bundler
wasm-bundler:
    @echo "📦 Building WASM for bundler..."
    wasm-pack build --target bundler

# Build WASM for Node.js
wasm-node:
    @echo "🟢 Building WASM for Node.js..."
    wasm-pack build --target nodejs

# Test WASM in browser
wasm-test-browser:
    @echo "🌐 Testing WASM in browser..."
    wasm-pack test --headless --chrome -- --lib --no-default-features

# Optimize WASM bundle
wasm-optimize:
    @echo "⚡ Optimizing WASM..."
    wasm-opt -Oz -o pkg/optimized_bg.wasm pkg/*_bg.wasm

# === Server Commands ===

# Start simple HTTP server
server PORT="8080":
    @echo "🌐 Starting HTTP server on port {{PORT}}..."
    basic-http-server . -a 0.0.0.0:{{PORT}}

# Start server in pkg directory
server-pkg PORT="8080":
    @echo "🌐 Starting HTTP server for pkg/ on port {{PORT}}..."
    cd pkg && basic-http-server . -a 0.0.0.0:{{PORT}}

# === Deploy Commands ===

# Deploy to GitHub Pages
deploy:
    @echo "🚀 Deploying to GitHub Pages..."
    nu scripts/deploy.nu

# Build for production deployment
deploy-build:
    @echo "🚀 Building for deployment..."
    nu scripts/build.nu --release --wasm

# === Benchmark Commands ===

# Run benchmarks
bench:
    @echo "⏱️  Running benchmarks..."
    cargo bench

# Run benchmarks for specific test
bench-one NAME:
    @echo "⏱️  Running benchmark: {{NAME}}..."
    cargo bench {{NAME}}

# === Audit & Security Commands ===

# Audit dependencies for security vulnerabilities
audit:
    @echo "🔒 Auditing dependencies..."
    cargo audit || echo "Install with: cargo install cargo-audit"

# Update dependencies
update:
    @echo "📦 Updating dependencies..."
    cargo update

# === Utility Commands ===

# Initialize a new WASM project structure
init-wasm:
    @echo "🌐 Initializing WASM project structure..."
    mkdir -p src www pkg
    @echo "✅ Created directories: src, www, pkg"

# Create a new example
new-example NAME:
    @echo "📝 Creating new example: {{NAME}}..."
    mkdir -p examples
    touch examples/{{NAME}}.rs
    @echo "✅ Created examples/{{NAME}}.rs"

# Generate ctags
tags:
    @echo "🏷️  Generating ctags..."
    ctags -R .

# Show help with descriptions
help:
    @echo "🛠️  webatui - Project Commands"
    @echo ""
    @echo "📋 Use 'just' to see all commands"
    @echo "📋 Use 'just <command>' to run a specific command"
    @echo ""
    @echo "Quick commands:"
    @echo "  just dev          - Run development workflow"
    @echo "  just build-wasm   - Build WASM bundle"
    @echo "  just serve        - Serve WASM locally"
    @echo "  just test         - Run tests"
    @echo ""
    @echo "For more information: just --list"

# === Advanced Commands ===

# Profile build time
profile-build:
    @echo "⏱️  Profiling build time..."
    cargo build --timings

# Show compilation times
timings:
    @echo "⏱️  Analyzing compilation times..."
    cargo build --timings
    @echo "Open target/cargo-timings/cargo-timing.html to view report"

# Expand macros
expand:
    @echo "🔍 Expanding macros..."
    cargo expand

# Check binary size
size:
    @echo "📦 Checking binary size..."
    cargo size --release -- -A

# Strip binary
strip:
    @echo "🔪 Stripping binary..."
    cargo build --release
    strip target/release/*

# Show assembly output
asm:
    @echo "🔍 Showing assembly..."
    cargo asm
