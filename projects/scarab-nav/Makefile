.PHONY: help test build check fmt clippy clean release dry-run

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

test: ## Run all tests
	cargo test --all-features --workspace

build: ## Build all crates
	cargo build --all-features --workspace

check: test fmt clippy ## Run all checks (tests, formatting, clippy)
	@echo "All checks passed!"

fmt: ## Check code formatting
	cargo fmt --all -- --check

fmt-fix: ## Fix code formatting
	cargo fmt --all

clippy: ## Run clippy lints
	cargo clippy --all-features --workspace -- -D warnings

clean: ## Clean build artifacts
	cargo clean

dry-run: ## Test package publishing (dry run)
	@echo "Testing scarab-nav-protocol package..."
	cd crates/scarab-nav-protocol && cargo publish --dry-run --allow-dirty
	@echo ""
	@echo "Note: scarab-nav-client dry-run will fail until protocol is published to crates.io"

doc: ## Build and open documentation
	cargo doc --all-features --workspace --no-deps --open

release: ## Interactive release helper (requires version argument: make release VERSION=0.2.0)
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION is required. Usage: make release VERSION=0.2.0"; \
		exit 1; \
	fi
	./scripts/release.sh $(VERSION) both

release-protocol: ## Release protocol crate only (requires version argument: make release-protocol VERSION=0.2.0)
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION is required. Usage: make release-protocol VERSION=0.2.0"; \
		exit 1; \
	fi
	./scripts/release.sh $(VERSION) protocol

release-client: ## Release client crate only (requires version argument: make release-client VERSION=0.2.0)
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION is required. Usage: make release-client VERSION=0.2.0"; \
		exit 1; \
	fi
	./scripts/release.sh $(VERSION) client
