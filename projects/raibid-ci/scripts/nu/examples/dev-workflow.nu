#!/usr/bin/env nu
# Raibid-CI Development Workflow Example
# Demonstrates a common development workflow using raibid-ci modules

# Source environment and config
source ../env.nu
source ../config.nu

def main [
    --check-all  # Run all health checks
    --build      # Build the project
    --test       # Run tests
    --status     # Show project status
] {
    print "=== Raibid-CI Development Workflow ==="
    print ""

    if $status or not ($check_all or $build or $test) {
        # Show project status
        project-status
        return
    }

    if $check_all {
        run-health-checks
    }

    if $build {
        build-project
    }

    if $test {
        run-tests
    }
}

def run-health-checks [] {
    log-info "Running health checks..."
    print ""

    # Check prerequisites
    if not (check-dev-prerequisites) {
        log-error "Some prerequisites are missing"
        exit 1
    }

    print ""

    # Check infrastructure (if available)
    check-infrastructure
}

def check-infrastructure [] {
    log-info "Checking infrastructure components..."
    print ""

    # Check k3s
    if (command-exists "kubectl") {
        use ../modules/kubectl.nu *
        if (kubectl-check-cluster) {
            log-success "k3s cluster is healthy"
        } else {
            log-warning "k3s cluster not reachable (optional for development)"
        }
    } else {
        log-info "kubectl not installed (optional for development)"
    }

    # Check Redis
    if (command-exists "redis-cli") {
        use ../modules/redis.nu *
        if (redis-check-connection) {
            log-success "Redis is healthy"
        } else {
            log-warning "Redis not reachable (optional for development)"
        }
    } else {
        log-info "redis-cli not installed (optional for development)"
    }

    # Check Gitea
    if (command-exists "curl") {
        use ../modules/gitea.nu *
        if (gitea-check-connection) {
            log-success "Gitea is healthy"
        } else {
            log-warning "Gitea not reachable (optional for development)"
        }
    } else {
        log-info "curl not installed (needed for Gitea checks)"
    }

    print ""
}

def build-project [] {
    log-info "Building raibid-ci..."

    cd $env.RAIBID_ROOT

    try {
        cargo build --release
        log-success "Build completed successfully!"

        # Show binary location
        let binary = ($env.RAIBID_RELEASE | path join "raibid-cli")
        if ($binary | path exists) {
            print $"Binary: ($binary)"
        }
    } catch {
        log-error "Build failed!"
        exit 1
    }
}

def run-tests [] {
    log-info "Running tests..."

    cd $env.RAIBID_ROOT

    try {
        cargo test
        log-success "All tests passed!"
    } catch {
        log-error "Tests failed!"
        exit 1
    }
}

main
