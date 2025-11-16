# Makefile for raibid-ci agent container
# Convenience wrapper around build.sh

.PHONY: help build push test clean verify health-check shell

# Configuration
IMAGE_NAME ?= raibid-ci-agent
IMAGE_TAG ?= latest
REGISTRY ?= localhost:5000
PLATFORM ?= linux/arm64

# Colors for output
COLOR_RESET = \033[0m
COLOR_BOLD = \033[1m
COLOR_GREEN = \033[32m
COLOR_YELLOW = \033[33m
COLOR_BLUE = \033[34m

help: ## Show this help message
	@echo "$(COLOR_BOLD)raibid-ci Agent Container Build$(COLOR_RESET)"
	@echo ""
	@echo "$(COLOR_BLUE)Available targets:$(COLOR_RESET)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(COLOR_GREEN)%-15s$(COLOR_RESET) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(COLOR_BLUE)Configuration:$(COLOR_RESET)"
	@echo "  IMAGE_NAME = $(IMAGE_NAME)"
	@echo "  IMAGE_TAG  = $(IMAGE_TAG)"
	@echo "  REGISTRY   = $(REGISTRY)"
	@echo "  PLATFORM   = $(PLATFORM)"
	@echo ""
	@echo "$(COLOR_BLUE)Examples:$(COLOR_RESET)"
	@echo "  make build                    # Build locally"
	@echo "  make build push               # Build and push"
	@echo "  make REGISTRY=gitea.local:3000/raibid push"
	@echo "  make test                     # Run tests"

build: ## Build the container image
	@echo "$(COLOR_BLUE)Building $(IMAGE_NAME):$(IMAGE_TAG)...$(COLOR_RESET)"
	./build.sh \
		--name $(IMAGE_NAME) \
		--tag $(IMAGE_TAG) \
		--registry $(REGISTRY) \
		--platform $(PLATFORM)

push: build ## Build and push to registry
	@echo "$(COLOR_BLUE)Pushing $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)...$(COLOR_RESET)"
	docker push $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)

build-push: ## Build and push in one step (using BuildKit)
	@echo "$(COLOR_BLUE)Building and pushing $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)...$(COLOR_RESET)"
	./build.sh \
		--name $(IMAGE_NAME) \
		--tag $(IMAGE_TAG) \
		--registry $(REGISTRY) \
		--platform $(PLATFORM) \
		--push

test: build health-check ## Build and run tests
	@echo "$(COLOR_GREEN)All tests passed!$(COLOR_RESET)"

health-check: ## Run health check on built image
	@echo "$(COLOR_BLUE)Running health check...$(COLOR_RESET)"
	docker run --rm --platform $(PLATFORM) $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) /usr/local/bin/healthcheck.sh

verify: ## Verify image details
	@echo "$(COLOR_BLUE)Image Details:$(COLOR_RESET)"
	@echo ""
	@docker image inspect $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) --format='  Created:       {{.Created}}'
	@docker image inspect $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) --format='  Size:          {{.Size}}' | awk '{printf "  Size:          %.2f MB\n", $$2/1024/1024}'
	@docker image inspect $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) --format='  Architecture:  {{.Architecture}}'
	@docker image inspect $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) --format='  OS:            {{.Os}}'
	@echo ""
	@echo "$(COLOR_BLUE)Installed Tools:$(COLOR_RESET)"
	@docker run --rm --platform $(PLATFORM) $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) rustc --version
	@docker run --rm --platform $(PLATFORM) $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) cargo --version
	@docker run --rm --platform $(PLATFORM) $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) docker --version
	@docker run --rm --platform $(PLATFORM) $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) git --version
	@docker run --rm --platform $(PLATFORM) $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) cargo-nextest --version
	@docker run --rm --platform $(PLATFORM) $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) cargo-audit --version
	@docker run --rm --platform $(PLATFORM) $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) cargo-deny --version

shell: ## Open interactive shell in container
	@echo "$(COLOR_BLUE)Starting interactive shell...$(COLOR_RESET)"
	docker run -it --rm --platform $(PLATFORM) $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) /bin/bash

shell-with-docker: ## Open shell with Docker socket mounted
	@echo "$(COLOR_BLUE)Starting interactive shell with Docker...$(COLOR_RESET)"
	docker run -it --rm \
		--platform $(PLATFORM) \
		-v /var/run/docker.sock:/var/run/docker.sock \
		$(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) \
		/bin/bash

clean: ## Remove built images
	@echo "$(COLOR_YELLOW)Removing images...$(COLOR_RESET)"
	-docker rmi $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)
	@echo "$(COLOR_GREEN)Cleaned!$(COLOR_RESET)"

clean-cache: ## Remove BuildKit cache
	@echo "$(COLOR_YELLOW)Removing BuildKit cache...$(COLOR_RESET)"
	rm -rf /tmp/buildkit-cache
	@echo "$(COLOR_GREEN)Cache cleaned!$(COLOR_RESET)"

size-check: ## Check if image size meets constraint
	@SIZE=$$(docker image inspect $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) --format='{{.Size}}' | awk '{print $$1/1024/1024}'); \
	if [ "$$(echo "$$SIZE > 1536" | bc -l)" -eq 1 ]; then \
		echo "$(COLOR_YELLOW)WARNING: Image size ($$SIZE MB) exceeds 1536 MB target$(COLOR_RESET)"; \
		exit 1; \
	else \
		echo "$(COLOR_GREEN)Image size ($$SIZE MB) is within 1536 MB target$(COLOR_RESET)"; \
	fi

.DEFAULT_GOAL := help
