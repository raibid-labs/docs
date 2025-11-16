# Raibid-CI Nushell Configuration
# Project-specific Nushell settings for development automation
#
# This file provides customized settings for the raibid-ci development environment.
# Source this file in your Nushell session to enable project-specific functionality.
#
# Usage:
#   source scripts/nu/config.nu

# Version check - ensure we're using Nushell 0.96 or later
let nu_version = (version | get version)
if ($nu_version | split row '.' | first | into int) < 0 {
    error make {msg: "Nushell version too old. Please upgrade to 0.96 or later."}
}

# Project configuration
def project-info [] {
    {
        name: "raibid-ci"
        description: "DGX Spark Personal CI Agent Pool"
        repo: "raibid-labs/raibid-ci"
        project_root: ($env.PWD | path dirname | path dirname)
    }
}

# Color scheme for output
const COLORS = {
    success: (ansi green)
    error: (ansi red)
    warning: (ansi yellow)
    info: (ansi blue)
    reset: (ansi reset)
}

# Common paths
def --env setup-paths [] {
    let project_root = (project-info | get project_root)

    $env.RAIBID_ROOT = $project_root
    $env.RAIBID_SCRIPTS = ($project_root | path join "scripts" "nu")
    $env.RAIBID_MODULES = ($project_root | path join "scripts" "nu" "modules")
    $env.RAIBID_DOCS = ($project_root | path join "docs")
    $env.RAIBID_EXAMPLES = ($project_root | path join "examples")
}

# Setup module path for easy imports
def --env setup-module-path [] {
    setup-paths

    # Add our modules directory to NU_LIB_DIRS if not already present
    if ($env.RAIBID_MODULES not-in $env.NU_LIB_DIRS) {
        $env.NU_LIB_DIRS = ($env.NU_LIB_DIRS | append $env.RAIBID_MODULES)
    }
}

# Logging utilities
def log-success [message: string] {
    print $"($COLORS.success)✓($COLORS.reset) ($message)"
}

def log-error [message: string] {
    print $"($COLORS.error)✗($COLORS.reset) ($message)"
}

def log-warning [message: string] {
    print $"($COLORS.warning)⚠($COLORS.reset) ($message)"
}

def log-info [message: string] {
    print $"($COLORS.info)ℹ($COLORS.reset) ($message)"
}

# Check if a command exists
def command-exists [cmd: string] {
    (which $cmd | length) > 0
}

# Validate prerequisites for development
def check-dev-prerequisites [] {
    log-info "Checking development prerequisites..."

    let required = [
        {name: "cargo", desc: "Rust toolchain"}
        {name: "git", desc: "Git version control"}
        {name: "gh", desc: "GitHub CLI"}
    ]

    mut all_present = true

    for tool in $required {
        if (command-exists $tool.name) {
            log-success $"($tool.name) - ($tool.desc)"
        } else {
            log-error $"($tool.name) not found - ($tool.desc)"
            $all_present = false
        }
    }

    $all_present
}

# Quick project status
def project-status [] {
    let info = (project-info)

    print $"\n($COLORS.info)Project: ($info.name)($COLORS.reset)"
    print $"Description: ($info.description)"
    print $"Repository: ($info.repo)\n"

    # Git status
    if (command-exists "git") {
        let branch = (git branch --show-current)
        let status = (git status --porcelain | lines | length)

        print $"Git branch: ($branch)"
        if $status > 0 {
            log-warning $"Uncommitted changes: ($status) files"
        } else {
            log-success "Working tree clean"
        }
    }

    # Cargo status
    if (command-exists "cargo") {
        try {
            cargo --version | print
        }
    }
}

# Auto-initialize when sourced
setup-module-path
log-success "Raibid-CI Nushell configuration loaded"
