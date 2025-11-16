# Raibid-CI Nushell Environment Configuration
# Environment variables and PATH setup for raibid-ci development
#
# This file sets up the environment for raibid-ci development.
# Source this file before running development commands.
#
# Usage:
#   source scripts/nu/env.nu

# Determine project root (two directories up from this file)
let project_root = ($env.FILE_PWD | path dirname | path dirname)

# Core project paths
$env.RAIBID_ROOT = $project_root
$env.RAIBID_SCRIPTS = ($project_root | path join "scripts")
$env.RAIBID_NU_SCRIPTS = ($project_root | path join "scripts" "nu")
$env.RAIBID_MODULES = ($project_root | path join "scripts" "nu" "modules")
$env.RAIBID_DOCS = ($project_root | path join "docs")
$env.RAIBID_EXAMPLES = ($project_root | path join "examples")
$env.RAIBID_SRC = ($project_root | path join "src")
$env.RAIBID_TESTS = ($project_root | path join "tests")

# Build paths
$env.RAIBID_TARGET = ($project_root | path join "target")
$env.RAIBID_RELEASE = ($project_root | path join "target" "release")
$env.RAIBID_DEBUG = ($project_root | path join "target" "debug")

# Development configuration
$env.RAIBID_ENV = "development"
$env.RAIBID_LOG_LEVEL = "info"

# CI/CD configuration (when running in CI)
if "CI" in $env {
    $env.RAIBID_ENV = "ci"
    $env.RAIBID_LOG_LEVEL = "debug"
}

# k3s configuration (for local development)
$env.RAIBID_K3S_CONFIG = ($env.HOME | path join ".kube" "config")
$env.RAIBID_KUBECONFIG = $env.RAIBID_K3S_CONFIG

# Gitea configuration
$env.RAIBID_GITEA_URL = "http://localhost:3000"
$env.RAIBID_GITEA_API = "http://localhost:3000/api/v1"

# Redis configuration
$env.RAIBID_REDIS_URL = "redis://localhost:6379"
$env.RAIBID_REDIS_STREAM = "raibid:jobs"

# Add project bin to PATH (for local builds)
let raibid_bin = ($env.RAIBID_RELEASE)
if $raibid_bin not-in $env.PATH {
    $env.PATH = ($env.PATH | prepend $raibid_bin)
}

# Add user local bin to PATH (for installed tools)
let user_local_bin = ($env.HOME | path join ".local" "bin")
if ($user_local_bin | path exists) and ($user_local_bin not-in $env.PATH) {
    $env.PATH = ($env.PATH | prepend $user_local_bin)
}

# Add Nushell modules directory to NU_LIB_DIRS
if $env.RAIBID_MODULES not-in $env.NU_LIB_DIRS {
    $env.NU_LIB_DIRS = ($env.NU_LIB_DIRS | append $env.RAIBID_MODULES)
}

# Rust development settings
$env.CARGO_TERM_COLOR = "always"
$env.RUST_BACKTRACE = "1"

# Export info message
def show-env-info [] {
    print $"(ansi green)✓(ansi reset) Raibid-CI environment configured"
    print $"  Root: ($env.RAIBID_ROOT)"
    print $"  Environment: ($env.RAIBID_ENV)"
    print $"  Log level: ($env.RAIBID_LOG_LEVEL)"
}

show-env-info
