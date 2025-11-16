# DGX Spark MCP Server - Development Commands
# https://just.systems/

# Default recipe to display help information
default:
    @just --list

# ============================================================================
# Build Commands
# ============================================================================

# Build the project (compile TypeScript)
build:
    @echo "Building TypeScript..."
    npm run build
    @echo "Build complete!"

# Clean build artifacts
clean:
    @echo "Cleaning build artifacts..."
    npm run clean
    rm -rf coverage/
    rm -rf .nyc_output/
    rm -rf *.log
    @echo "Clean complete!"

# Clean and rebuild
rebuild: clean build

# Build documentation index
docs-build: build
    @echo "Building documentation index..."
    npm run docs:build
    @echo "Documentation index built!"

# ============================================================================
# Test Commands
# ============================================================================

# Run all tests
test:
    @echo "Running tests..."
    npm test

# Run tests in watch mode
test-watch:
    @echo "Running tests in watch mode..."
    npm run test:watch

# Run tests with coverage
test-coverage:
    @echo "Running tests with coverage..."
    npm run test:coverage
    @echo "Coverage report generated in coverage/"

# Run integration tests
test-integration:
    @echo "Running integration tests..."
    npm run test:integration

# Run tests with mocked hardware
test-mock:
    @echo "Running tests with mocked hardware..."
    MOCK_HARDWARE=true npm test

# Run performance benchmarks
test-benchmark:
    @echo "Running performance benchmarks..."
    npm run test:benchmark

# ============================================================================
# Development Server
# ============================================================================

# Run development server with hot reload
dev:
    @echo "Starting development server..."
    npm run dev

# Run production server
start: build
    @echo "Starting production server..."
    npm start

# ============================================================================
# Code Quality
# ============================================================================

# Run linter
lint:
    @echo "Running linter..."
    npm run lint

# Fix linting issues
lint-fix:
    @echo "Fixing linting issues..."
    npm run lint:fix

# Format code
format:
    @echo "Formatting code..."
    npm run format

# Check code formatting
format-check:
    @echo "Checking code formatting..."
    npm run format:check

# Run type checking
typecheck:
    @echo "Running type checker..."
    npm run typecheck

# Run all code quality checks
check: lint format-check typecheck
    @echo "All checks passed!"

# ============================================================================
# Docker Commands
# ============================================================================

# Build Docker image
docker-build:
    @echo "Building Docker image..."
    docker build -t dgx-spark-mcp:latest .
    @echo "Docker image built!"

# Run Docker container
docker-run:
    @echo "Running Docker container..."
    docker run --rm -it \
        --name dgx-spark-mcp \
        -v $(pwd)/config:/app/config:ro \
        -v $(pwd)/logs:/app/logs \
        dgx-spark-mcp:latest

# Run Docker container with GPU support
docker-run-gpu:
    @echo "Running Docker container with GPU support..."
    docker run --rm -it \
        --name dgx-spark-mcp \
        --gpus all \
        -v $(pwd)/config:/app/config:ro \
        -v $(pwd)/logs:/app/logs \
        dgx-spark-mcp:latest

# Stop Docker container
docker-stop:
    @echo "Stopping Docker container..."
    docker stop dgx-spark-mcp

# Remove Docker image
docker-clean:
    @echo "Removing Docker image..."
    docker rmi dgx-spark-mcp:latest

# Run Docker shell
docker-shell:
    @echo "Starting Docker shell..."
    docker run --rm -it \
        --entrypoint /bin/bash \
        dgx-spark-mcp:latest

# ============================================================================
# Deployment Commands
# ============================================================================

# Install systemd service
install: build
    @echo "Installing systemd service..."
    sudo ./scripts/install.sh
    @echo "Installation complete!"

# Update to latest version
update:
    @echo "Updating to latest version..."
    ./scripts/update.sh
    @echo "Update complete!"

# Rollback to previous version
rollback:
    @echo "Rolling back to previous version..."
    ./scripts/rollback.sh
    @echo "Rollback complete!"

# Start systemd service
service-start:
    @echo "Starting systemd service..."
    sudo systemctl start dgx-spark-mcp
    sudo systemctl status dgx-spark-mcp

# Stop systemd service
service-stop:
    @echo "Stopping systemd service..."
    sudo systemctl stop dgx-spark-mcp

# Restart systemd service
service-restart:
    @echo "Restarting systemd service..."
    sudo systemctl restart dgx-spark-mcp
    sudo systemctl status dgx-spark-mcp

# View service status
service-status:
    @echo "Service status:"
    sudo systemctl status dgx-spark-mcp

# View service logs
service-logs:
    @echo "Service logs:"
    sudo journalctl -u dgx-spark-mcp -f

# Enable service on boot
service-enable:
    @echo "Enabling service on boot..."
    sudo systemctl enable dgx-spark-mcp

# Disable service on boot
service-disable:
    @echo "Disabling service on boot..."
    sudo systemctl disable dgx-spark-mcp

# ============================================================================
# Monitoring Commands
# ============================================================================

# Check health endpoint
health:
    @echo "Checking health endpoint..."
    curl -s http://localhost:3000/health | jq .

# Check metrics endpoint
metrics:
    @echo "Fetching Prometheus metrics..."
    curl -s http://localhost:3000/metrics

# View logs
logs:
    @echo "Tailing logs..."
    tail -f logs/dgx-mcp-combined.log

# View error logs
logs-error:
    @echo "Tailing error logs..."
    tail -f logs/dgx-mcp-error.log

# ============================================================================
# Utility Commands
# ============================================================================

# Validate configuration
validate-config: build
    @echo "Validating configuration..."
    npm run validate-config
    @echo "Configuration is valid!"

# Search documentation
docs-search query:
    @echo "Searching documentation for: {{query}}"
    npm run docs:search -- "{{query}}"

# Generate hardware report
hardware-report:
    @echo "Generating hardware report..."
    node test-hardware.mjs

# Test Spark intelligence
test-spark:
    @echo "Testing Spark intelligence..."
    node test-intelligence.js

# Install dependencies
deps:
    @echo "Installing dependencies..."
    npm install

# Update dependencies
deps-update:
    @echo "Updating dependencies..."
    npm update

# Check for outdated dependencies
deps-outdated:
    @echo "Checking for outdated dependencies..."
    npm outdated

# Audit dependencies for vulnerabilities
deps-audit:
    @echo "Auditing dependencies..."
    npm audit

# Fix dependency vulnerabilities
deps-audit-fix:
    @echo "Fixing dependency vulnerabilities..."
    npm audit fix

# ============================================================================
# Release Commands
# ============================================================================

# Create a new release (version bump)
release-patch:
    @echo "Creating patch release..."
    npm version patch
    git push --follow-tags

# Create a new minor release
release-minor:
    @echo "Creating minor release..."
    npm version minor
    git push --follow-tags

# Create a new major release
release-major:
    @echo "Creating major release..."
    npm version major
    git push --follow-tags

# ============================================================================
# CI/CD Commands
# ============================================================================

# Run CI pipeline locally (requires act)
ci-test:
    @echo "Running CI tests locally..."
    act -j test

# Run build workflow locally
ci-build:
    @echo "Running CI build locally..."
    act -j build

# Verify all workflows
ci-verify:
    @echo "Verifying GitHub Actions workflows..."
    act --list

# ============================================================================
# Complete Workflow Commands
# ============================================================================

# Full pre-commit check
pre-commit: check test build
    @echo "Pre-commit checks complete!"

# Full pre-push check
pre-push: check test-coverage build docker-build
    @echo "Pre-push checks complete!"

# Full release preparation
pre-release: clean check test-coverage build docker-build validate-config
    @echo "Pre-release checks complete!"
