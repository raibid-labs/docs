#!/usr/bin/env nu

# clean.nu - Cleanup build artifacts and temporary files
#
# Usage:
#   nu scripts/clean.nu [OPTIONS]
#
# Options:
#   --all             Clean everything (cargo, wasm, node_modules, coverage)
#   --cargo           Clean cargo build artifacts (target/)
#   --wasm            Clean WASM artifacts (pkg/, wasm target)
#   --node            Clean node_modules
#   --coverage        Clean coverage reports
#   --temp            Clean temporary files
#   --dry-run         Show what would be deleted without deleting
#   --help            Show this help message

def main [
    --all             # Clean everything
    --cargo           # Clean cargo artifacts
    --wasm            # Clean WASM artifacts
    --node            # Clean node_modules
    --coverage        # Clean coverage reports
    --temp            # Clean temporary files
    --dry-run         # Show what would be deleted
    --help            # Show help
] {
    if $help {
        show_help
        return
    }

    print_header "🧹 Cleanup Automation"

    # If no specific option is provided, show what's available to clean
    let clean_all = $all or (not $cargo and not $wasm and not $node and not $coverage and not $temp)

    if $dry_run {
        print_warning "DRY RUN MODE - No files will be deleted"
    }

    if $clean_all {
        clean_cargo $dry_run
        clean_wasm $dry_run
        clean_node $dry_run
        clean_coverage $dry_run
        clean_temp $dry_run
    } else {
        if $cargo {
            clean_cargo $dry_run
        }
        if $wasm {
            clean_wasm $dry_run
        }
        if $node {
            clean_node $dry_run
        }
        if $coverage {
            clean_coverage $dry_run
        }
        if $temp {
            clean_temp $dry_run
        }
    }

    # Show disk space summary
    show_summary

    print_success "Cleanup completed!"
}

# Clean cargo build artifacts
def clean_cargo [dry_run: bool] {
    print_step "Cleaning cargo build artifacts"

    if ("target" | path exists) {
        let size = (du -sh target | parse "{size} {path}" | get size | first)
        print_info $"Target directory size: ($size)"

        if $dry_run {
            print_warning "Would delete: target/"
        } else {
            print_info "Removing target/ directory..."
            rm -rf target
            print_success "Removed target/ directory"
        }
    } else {
        print_info "No cargo artifacts to clean (target/ doesn't exist)"
    }
}

# Clean WASM artifacts
def clean_wasm [dry_run: bool] {
    print_step "Cleaning WASM artifacts"

    let mut cleaned = false

    if ("pkg" | path exists) {
        let size = (du -sh pkg | parse "{size} {path}" | get size | first)
        print_info $"pkg/ directory size: ($size)"

        if $dry_run {
            print_warning "Would delete: pkg/"
        } else {
            print_info "Removing pkg/ directory..."
            rm -rf pkg
            print_success "Removed pkg/ directory"
            $cleaned = true
        }
    }

    if ("target/wasm32-unknown-unknown" | path exists) {
        let size = (du -sh target/wasm32-unknown-unknown | parse "{size} {path}" | get size | first)
        print_info $"WASM target size: ($size)"

        if $dry_run {
            print_warning "Would delete: target/wasm32-unknown-unknown/"
        } else {
            print_info "Removing target/wasm32-unknown-unknown/..."
            rm -rf target/wasm32-unknown-unknown
            print_success "Removed WASM target"
            $cleaned = true
        }
    }

    if not $cleaned {
        print_info "No WASM artifacts to clean"
    }
}

# Clean node_modules
def clean_node [dry_run: bool] {
    print_step "Cleaning node_modules"

    if ("node_modules" | path exists) {
        let size = (du -sh node_modules | parse "{size} {path}" | get size | first)
        print_info $"node_modules size: ($size)"

        if $dry_run {
            print_warning "Would delete: node_modules/"
        } else {
            print_info "Removing node_modules/..."
            rm -rf node_modules
            print_success "Removed node_modules/"
        }
    } else {
        print_info "No node_modules to clean"
    }
}

# Clean coverage reports
def clean_coverage [dry_run: bool] {
    print_step "Cleaning coverage reports"

    let mut cleaned = false

    if ("coverage" | path exists) {
        let size = (du -sh coverage | parse "{size} {path}" | get size | first)
        print_info $"coverage/ size: ($size)"

        if $dry_run {
            print_warning "Would delete: coverage/"
        } else {
            print_info "Removing coverage/..."
            rm -rf coverage
            print_success "Removed coverage/"
            $cleaned = true
        }
    }

    # Also check for cobertura.xml and other coverage files
    let coverage_files = (ls **/*.profraw **/*.profdata cobertura.xml tarpaulin-report.html 2>/dev/null | length)

    if $coverage_files > 0 {
        if $dry_run {
            print_warning $"Would delete ($coverage_files) coverage files"
        } else {
            print_info $"Removing ($coverage_files) coverage files..."
            rm -f **/*.profraw **/*.profdata cobertura.xml tarpaulin-report.html 2>/dev/null
            print_success "Removed coverage files"
            $cleaned = true
        }
    }

    if not $cleaned {
        print_info "No coverage reports to clean"
    }
}

# Clean temporary files
def clean_temp [dry_run: bool] {
    print_step "Cleaning temporary files"

    let temp_patterns = [
        "**/*.swp",      # Vim swap files
        "**/*.swo",      # Vim swap files
        "**/*~",         # Backup files
        "**/.DS_Store",  # macOS metadata
        "**/Thumbs.db",  # Windows thumbnails
        "**/*.tmp",      # Temporary files
        "**/*.bak",      # Backup files
    ]

    let mut cleaned = false

    for pattern in $temp_patterns {
        let files = (do { ls $pattern } | complete | get stdout | lines | length)

        if $files > 0 {
            if $dry_run {
                print_warning $"Would delete ($files) files matching ($pattern)"
            } else {
                print_info $"Removing files matching ($pattern)..."
                rm -f $pattern
                $cleaned = true
            }
        }
    }

    if $cleaned and not $dry_run {
        print_success "Removed temporary files"
    } else if not $cleaned {
        print_info "No temporary files to clean"
    }
}

# Show summary of cleaned space
def show_summary [] {
    print_step "Disk space summary"

    # Check current directory size
    let current_size = (du -sh . | parse "{size} {path}" | get size | first)
    print_info $"Current directory size: ($current_size)"

    # Show what exists
    print_info "Existing artifacts:"

    if ("target" | path exists) {
        let size = (du -sh target | parse "{size} {path}" | get size | first)
        print $"  - target/: ($size)"
    }

    if ("pkg" | path exists) {
        let size = (du -sh pkg | parse "{size} {path}" | get size | first)
        print $"  - pkg/: ($size)"
    }

    if ("node_modules" | path exists) {
        let size = (du -sh node_modules | parse "{size} {path}" | get size | first)
        print $"  - node_modules/: ($size)"
    }

    if ("coverage" | path exists) {
        let size = (du -sh coverage | parse "{size} {path}" | get size | first)
        print $"  - coverage/: ($size)"
    }
}

# Show help message
def show_help [] {
    print "
🧹 clean.nu - Cleanup build artifacts and temporary files

Usage:
    nu scripts/clean.nu [OPTIONS]

Options:
    --all             Clean everything (recommended for fresh start)
    --cargo           Clean cargo build artifacts (target/)
    --wasm            Clean WASM artifacts (pkg/, wasm target)
    --node            Clean node_modules
    --coverage        Clean coverage reports
    --temp            Clean temporary files (*.swp, *~, .DS_Store, etc.)
    --dry-run         Show what would be deleted without deleting
    --help            Show this help message

Examples:
    # Clean everything
    nu scripts/clean.nu --all

    # Clean only cargo artifacts
    nu scripts/clean.nu --cargo

    # Clean WASM and cargo
    nu scripts/clean.nu --wasm --cargo

    # Dry run to see what would be deleted
    nu scripts/clean.nu --all --dry-run

    # Clean temporary files only
    nu scripts/clean.nu --temp

Directories cleaned:
    target/                    Cargo build artifacts
    pkg/                       WASM package output
    target/wasm32-unknown-unknown/  WASM build artifacts
    node_modules/              Node.js dependencies
    coverage/                  Coverage reports

Files cleaned:
    **/*.swp, **/*.swo        Vim swap files
    **/*~                      Backup files
    **/.DS_Store              macOS metadata
    **/Thumbs.db              Windows thumbnails
    **/*.tmp, **/*.bak        Temporary/backup files
    **/*.profraw, **/*.profdata  Coverage data
    cobertura.xml             Coverage report
    tarpaulin-report.html     Tarpaulin coverage report
    "
}

# === Utility Functions ===

def print_header [msg: string] {
    print $"(ansi green_bold)($msg)(ansi reset)"
    print $"(ansi green){'=' * 50}(ansi reset)"
}

def print_step [msg: string] {
    print $"(ansi cyan_bold)▶ ($msg)(ansi reset)"
}

def print_info [msg: string] {
    print $"(ansi blue)ℹ ($msg)(ansi reset)"
}

def print_success [msg: string] {
    print $"(ansi green)✓ ($msg)(ansi reset)"
}

def print_error [msg: string] {
    print $"(ansi red)✗ ($msg)(ansi reset)"
}

def print_warning [msg: string] {
    print $"(ansi yellow)⚠ ($msg)(ansi reset)"
}

def print_command [cmd: list] {
    print $"(ansi purple)$ ($cmd | str join ' ')(ansi reset)"
}
