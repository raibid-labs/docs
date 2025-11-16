# raibid-ci justfile
# Development automation for building, running, and testing

# Default recipe - show available commands
default:
    @just --list

# === Build Commands ===

# Build all workspace crates in dev mode
build:
    cargo build --workspace

# Build all workspace crates in release mode
build-release:
    cargo build --workspace --release

# Build only the CLI in dev mode
build-cli:
    cargo build --package raibid-cli

# Build only the CLI in release mode
build-cli-release:
    cargo build --package raibid-cli --release

# Build only the server in dev mode
build-server:
    cargo build --package raibid-server

# Build only the server in release mode
build-server-release:
    cargo build --package raibid-server --release

# Build only the agent in dev mode
build-agent:
    cargo build --package raibid-agent

# Build only the agent in release mode
build-agent-release:
    cargo build --package raibid-agent --release

# Clean build artifacts
clean:
    cargo clean

# === Run Commands ===

# Run the CLI with help
cli *ARGS:
    cargo run --package raibid-cli -- {{ARGS}}

# Run the CLI in release mode
cli-release *ARGS:
    cargo run --package raibid-cli --release -- {{ARGS}}

# Launch the TUI dashboard
tui:
    cargo run --package raibid-cli -- tui

# Launch the TUI dashboard in release mode
tui-release:
    cargo run --package raibid-cli --release -- tui

# Run the server (default config)
server PORT="8080":
    RUST_LOG=raibid_server=debug,tower_http=debug cargo run --package raibid-server -- --port {{PORT}}

# Run the server in release mode
server-release PORT="8080":
    RUST_LOG=raibid_server=info cargo run --package raibid-server --release -- --port {{PORT}}

# Run the agent
agent:
    RUST_LOG=raibid_agent=debug cargo run --package raibid-agent

# Run the agent in release mode
agent-release:
    RUST_LOG=raibid_agent=info cargo run --package raibid-agent --release

# === Test Commands ===

# Run all tests
test:
    cargo test --workspace

# Run all tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Run tests for a specific package
test-package PACKAGE:
    cargo test --package {{PACKAGE}}

# Run only unit tests (lib tests)
test-unit:
    cargo test --workspace --lib

# Run only integration tests
test-integration:
    cargo test --workspace --test '*'

# Run only doc tests
test-doc:
    cargo test --workspace --doc

# Run tests for the CLI
test-cli:
    cargo test --package raibid-cli

# Run tests for the server
test-server:
    cargo test --package raibid-server

# Run tests for the agent
test-agent:
    cargo test --package raibid-agent

# Run tests for common lib
test-common:
    cargo test --package raibid-common

# Run tests for TUI
test-tui:
    cargo test --package raibid-tui

# Run specific test by name
test-name NAME:
    cargo test --workspace {{NAME}}

# Run tests and show coverage (requires tarpaulin)
test-coverage:
    cargo tarpaulin --workspace --out Html --output-dir coverage

# === Check Commands ===

# Run cargo check on workspace
check:
    cargo check --workspace

# Run clippy (linter)
lint:
    cargo clippy --workspace -- -D warnings

# Run clippy with auto-fixes
lint-fix:
    cargo clippy --workspace --fix --allow-dirty --allow-staged

# Check code formatting
fmt-check:
    cargo fmt --all -- --check

# Format all code
fmt:
    cargo fmt --all

# Run all quality checks (check, lint, fmt, test)
ci: check lint fmt-check test
    @echo "All CI checks passed!"

# === Development Commands ===

# Watch for changes and rebuild
watch:
    cargo watch -x "build --workspace"

# Watch for changes and run tests
watch-test:
    cargo watch -x "test --workspace"

# Watch for changes and run the CLI
watch-cli *ARGS:
    cargo watch -x "run --package raibid-cli -- {{ARGS}}"

# Watch for changes and run the server
watch-server PORT="8080":
    cargo watch -x "run --package raibid-server -- --port {{PORT}}"

# === Install Commands ===

# Install the CLI binary globally
install-cli:
    cargo install --path crates/cli

# Install all development tools
install-tools:
    cargo install cargo-watch cargo-tarpaulin

# === Utility Commands ===

# Show dependency tree
deps:
    cargo tree --workspace

# Show outdated dependencies
outdated:
    cargo outdated --workspace

# Update dependencies
update:
    cargo update

# Generate documentation
docs:
    cargo doc --workspace --no-deps --open

# Show workspace info
info:
    @echo "=== Workspace Info ==="
    @echo "Crates:"
    @cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | "  - \(.name) v\(.version)"'
    @echo ""
    @echo "Binaries:"
    @ls -1 target/debug/raibid* 2>/dev/null | grep -v '\.d$$' | sed 's/^/  - /' || echo "  (none built yet)"

# Run benchmarks (if any)
bench:
    cargo bench --workspace

# === Docker Commands (future) ===

# Build Docker image for server
docker-build-server:
    docker build -t raibid-server:latest -f infra/docker/Dockerfile.server .

# Build Docker image for agent
docker-build-agent:
    docker build -t raibid-agent:latest -f infra/docker/Dockerfile.agent .

# === Git Commands ===

# Show current branch and status
status:
    @git branch --show-current
    @git status --short

# Run pre-commit checks
pre-commit: fmt lint test
    @echo "Pre-commit checks passed! Ready to commit."
