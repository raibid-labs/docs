# Sparky Task Automation
# Based on dgx-pixels patterns

# Show available commands
default:
    @just --list

# ============================================================================
# Building
# ============================================================================

# Build all crates in release mode
build:
    cargo build --release --workspace

# Build all crates in debug mode
build-dev:
    cargo build --workspace

# Build specific crate
build-crate CRATE:
    cargo build --release -p {{CRATE}}

# Clean build artifacts
clean:
    cargo clean
    rm -rf data/raw/* data/processed/* output/* target/

# ============================================================================
# Testing
# ============================================================================

# Run all tests
test:
    cargo test --workspace

# Run tests for specific crate
test-crate CRATE:
    cargo test -p {{CRATE}}

# Run tests with coverage
test-coverage:
    cargo tarpaulin --workspace --out Html --output-dir coverage/

# Run integration tests
test-integration:
    cargo test --test '*' --workspace

# Run end-to-end test
test-e2e:
    nu scripts/test/test-e2e.nu

# Run load tests
test-load:
    nu scripts/test/test-load.nu

# ============================================================================
# Development
# ============================================================================

# Run orchestrator in dev mode
dev-orchestrator:
    cargo run --bin sparky-orchestrator

# Run collector in dev mode
dev-collector:
    cargo run --bin sparky-collector

# Run analyzer in dev mode
dev-analyzer:
    cargo run --bin sparky-analyzer

# Run generator in dev mode
dev-generator:
    cargo run --bin sparky-generator

# Run publisher in dev mode
dev-publisher:
    cargo run --bin sparky-publisher

# Watch and rebuild on changes
watch:
    cargo watch -x build

# ============================================================================
# Docker
# ============================================================================

# Build Docker images for all services
docker-build:
    nu scripts/docker/build.nu

# Build Docker image for specific service
docker-build-service SERVICE:
    docker build -t sparky-{{SERVICE}}:latest \
        --target {{SERVICE}} \
        -f Dockerfile .

# Push Docker images to registry
docker-push:
    nu scripts/docker/push.nu

# Run Docker Compose for local development
docker-up:
    docker-compose up -d

# Stop Docker Compose
docker-down:
    docker-compose down

# View Docker logs
docker-logs:
    docker-compose logs -f

# ============================================================================
# k3s/k3d (Local Development)
# ============================================================================

# Create k3d cluster for local development
k3d-create:
    nu scripts/k3s/create-cluster.nu

# Destroy k3d cluster
k3d-destroy:
    nu scripts/k3s/destroy-cluster.nu

# Deploy Ollama to k3d cluster
deploy-ollama:
    kubectl apply -f k8s/ollama/

# Deploy all services to k3d cluster
deploy-local:
    nu scripts/k3s/deploy-local.nu

# Undeploy from k3d cluster
undeploy-local:
    kubectl delete namespace sparky

# Get status of k3d deployment
k3d-status:
    kubectl get pods -n sparky
    kubectl get services -n sparky

# ============================================================================
# k3s (Production on DGX)
# ============================================================================

# Install k3s on DGX using k3sup
k3s-install-dgx:
    nu scripts/k3s/install-dgx.nu

# Deploy to production k3s cluster
deploy-production:
    nu scripts/k3s/deploy-production.nu

# Undeploy from production
undeploy-production:
    nu scripts/k3s/undeploy-production.nu

# Get production status
production-status:
    nu scripts/monitor/status.nu --env production

# ============================================================================
# Data Operations
# ============================================================================

# Collect today's data
collect-today:
    nu scripts/collect.nu --date (date now | format date "%Y-%m-%d")

# Collect data for specific date
collect-date DATE:
    nu scripts/collect.nu --date {{DATE}}

# Collect weekly data
collect-weekly:
    nu scripts/collect.nu --mode weekly

# Collect monthly data
collect-monthly:
    nu scripts/collect.nu --mode monthly

# Run full pipeline (collect → analyze → generate → publish)
pipeline-daily:
    nu scripts/pipeline.nu --mode daily

pipeline-weekly:
    nu scripts/pipeline.nu --mode weekly

pipeline-monthly:
    nu scripts/pipeline.nu --mode monthly

# ============================================================================
# Analysis
# ============================================================================

# Analyze collected data
analyze DATE:
    nu scripts/analyze.nu --date {{DATE}}

# Generate content from analysis
generate DATE:
    nu scripts/generate.nu --date {{DATE}}

# Publish generated content
publish DATE:
    nu scripts/publish.nu --date {{DATE}}

# ============================================================================
# Monitoring
# ============================================================================

# Show system status
status:
    nu scripts/monitor/status.nu

# Follow logs from all services
logs-follow:
    nu scripts/monitor/logs.nu --follow

# Show logs for specific service
logs-service SERVICE:
    kubectl logs -f -n sparky -l app={{SERVICE}}

# Show metrics
metrics:
    nu scripts/monitor/metrics.nu

# Health check all services
health-check:
    nu scripts/monitor/health-check.nu

# ============================================================================
# Demo
# ============================================================================

# Run demo of full pipeline
demo:
    nu scripts/demo/run-demo.nu

# Run demo of daily digest generation
demo-daily:
    nu scripts/demo/daily-digest.nu

# Run demo of weekly report
demo-weekly:
    nu scripts/demo/weekly-report.nu

# ============================================================================
# Configuration
# ============================================================================

# Validate configuration files
config-validate:
    nu scripts/config/validate.nu

# Show current configuration
config-show:
    nu scripts/config/show.nu

# Edit configuration for environment
config-edit ENV:
    $EDITOR config/{{ENV}}.toml

# ============================================================================
# Git Operations
# ============================================================================

# Commit generated content
git-commit:
    nu scripts/git/commit.nu

# Push to remote
git-push:
    nu scripts/git/push.nu

# Create PR for generated content
git-pr:
    nu scripts/git/create-pr.nu

# ============================================================================
# CI/CD
# ============================================================================

# Run CI checks locally
ci-check:
    just test
    just build
    cargo clippy --workspace -- -D warnings
    cargo fmt --check

# Format code
fmt:
    cargo fmt --all

# Run clippy linter
lint:
    cargo clippy --workspace -- -D warnings

# Fix linting issues automatically
lint-fix:
    cargo clippy --workspace --fix --allow-dirty

# ============================================================================
# Database Operations (if added later)
# ============================================================================

# Run database migrations (future)
db-migrate:
    # sqlx migrate run
    echo "Database migrations not yet implemented"

# Seed database with test data
db-seed:
    # nu scripts/db/seed.nu
    echo "Database seeding not yet implemented"

# ============================================================================
# Utilities
# ============================================================================

# Install all dependencies
install-deps:
    nu scripts/utils/install-deps.nu

# Check system requirements
check-requirements:
    nu scripts/utils/check-requirements.nu

# Generate API documentation
docs:
    cargo doc --workspace --no-deps --open

# Generate changelog
changelog:
    git log --pretty=format:"%h - %s (%an)" > CHANGELOG.txt

# Count lines of code
count-loc:
    tokei

# ============================================================================
# Help
# ============================================================================

# Show help for a specific command
help COMMAND:
    just --show {{COMMAND}}

# Show version information
version:
    @echo "Sparky Version: 1.0.0"
    @cargo --version
    @rustc --version
    @nu --version
    @kubectl version --client --short 2>/dev/null || echo "kubectl not installed"
