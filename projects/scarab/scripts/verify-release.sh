#!/usr/bin/env bash
#
# Release Build Verification Script
#
# This script automates the verification of release builds for Scarab.
# It builds the workspace in release mode, checks binary existence,
# verifies basic functionality, and generates a verification report.
#
# Usage:
#   ./scripts/verify-release.sh [--clean] [--verbose]
#
# Options:
#   --clean     Clean build artifacts before building
#   --verbose   Enable verbose output
#   --skip-build Skip the build step (verify existing binaries)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERBOSE=0
CLEAN=0
SKIP_BUILD=0
REPORT_FILE="$PROJECT_ROOT/release-verification-report.txt"

# Expected binaries
declare -a BINARIES=(
    "scarab-daemon"
    "scarab-client"
    "scarab-plugin-compiler"
)

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --clean)
            CLEAN=1
            shift
            ;;
        --verbose)
            VERBOSE=1
            shift
            ;;
        --skip-build)
            SKIP_BUILD=1
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--clean] [--verbose] [--skip-build]"
            echo ""
            echo "Options:"
            echo "  --clean       Clean build artifacts before building"
            echo "  --verbose     Enable verbose output"
            echo "  --skip-build  Skip the build step (verify existing binaries)"
            echo "  -h, --help    Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_verbose() {
    if [[ $VERBOSE -eq 1 ]]; then
        echo -e "${BLUE}[VERBOSE]${NC} $1"
    fi
}

# Initialize report
init_report() {
    cat > "$REPORT_FILE" <<EOF
========================================
Scarab Release Build Verification Report
========================================
Date: $(date)
Platform: $(uname -s) $(uname -m)
Rust Version: $(rustc --version)
Cargo Version: $(cargo --version)

EOF
}

# Find release directory (either project target or cargo home)
find_release_dir() {
    local project_target="$PROJECT_ROOT/target/release"
    local cargo_home="${CARGO_HOME:-$HOME/.cargo}"
    local cargo_target="$cargo_home/target/release"

    if [[ -d "$project_target" ]]; then
        echo "$project_target"
    elif [[ -d "$cargo_target" ]]; then
        echo "$cargo_target"
    else
        log_error "No release directory found"
        exit 1
    fi
}

# Clean build artifacts
clean_build() {
    log_info "Cleaning build artifacts..."
    cd "$PROJECT_ROOT"
    cargo clean
    log_success "Build artifacts cleaned"
}

# Build workspace in release mode
build_release() {
    log_info "Building workspace in release mode..."
    cd "$PROJECT_ROOT"

    local build_cmd="cargo build --release --workspace"

    if [[ $VERBOSE -eq 1 ]]; then
        $build_cmd
    else
        $build_cmd 2>&1 | grep -E "(Compiling|Finished|error|warning:)" || true
    fi

    if [[ ${PIPESTATUS[0]} -eq 0 ]]; then
        log_success "Release build completed successfully"
        echo "" >> "$REPORT_FILE"
        echo "Build Status: SUCCESS" >> "$REPORT_FILE"
        return 0
    else
        log_error "Release build failed"
        echo "" >> "$REPORT_FILE"
        echo "Build Status: FAILED" >> "$REPORT_FILE"
        return 1
    fi
}

# Verify binary existence and permissions
verify_binaries() {
    log_info "Verifying binaries..."

    local release_dir=$(find_release_dir)
    log_verbose "Release directory: $release_dir"

    echo "" >> "$REPORT_FILE"
    echo "Binary Verification:" >> "$REPORT_FILE"
    echo "-------------------" >> "$REPORT_FILE"

    local all_found=1

    for binary in "${BINARIES[@]}"; do
        local binary_path="$release_dir/$binary"

        if [[ -f "$binary_path" ]]; then
            local size=$(stat -f%z "$binary_path" 2>/dev/null || stat -c%s "$binary_path" 2>/dev/null)
            local size_mb=$(echo "scale=2; $size / 1024 / 1024" | bc)

            log_success "Found: $binary ($size_mb MB)"
            echo "  ✓ $binary - $size_mb MB" >> "$REPORT_FILE"

            # Check if executable
            if [[ -x "$binary_path" ]]; then
                log_verbose "$binary is executable"
            else
                log_warning "$binary exists but is not executable"
                echo "    WARNING: Not executable" >> "$REPORT_FILE"
            fi
        else
            log_error "Missing: $binary"
            echo "  ✗ $binary - NOT FOUND" >> "$REPORT_FILE"
            all_found=0
        fi
    done

    if [[ $all_found -eq 1 ]]; then
        log_success "All expected binaries found"
        return 0
    else
        log_error "Some binaries are missing"
        return 1
    fi
}

# Test binary execution
test_binaries() {
    log_info "Testing binary execution..."

    local release_dir=$(find_release_dir)

    echo "" >> "$REPORT_FILE"
    echo "Binary Execution Tests:" >> "$REPORT_FILE"
    echo "----------------------" >> "$REPORT_FILE"

    local all_passed=1

    # Test compiler --help
    log_verbose "Testing scarab-plugin-compiler --help"
    local compiler_path="$release_dir/scarab-plugin-compiler"
    if [[ -f "$compiler_path" ]]; then
        if timeout 5s "$compiler_path" --help &>/dev/null; then
            log_success "scarab-plugin-compiler --help works"
            echo "  ✓ scarab-plugin-compiler --help" >> "$REPORT_FILE"
        else
            log_error "scarab-plugin-compiler --help failed"
            echo "  ✗ scarab-plugin-compiler --help FAILED" >> "$REPORT_FILE"
            all_passed=0
        fi

        # Test compiler --version
        log_verbose "Testing scarab-plugin-compiler --version"
        if timeout 5s "$compiler_path" --version &>/dev/null; then
            local version=$("$compiler_path" --version 2>&1 | head -n1)
            log_success "scarab-plugin-compiler --version: $version"
            echo "  ✓ scarab-plugin-compiler --version: $version" >> "$REPORT_FILE"
        else
            log_error "scarab-plugin-compiler --version failed"
            echo "  ✗ scarab-plugin-compiler --version FAILED" >> "$REPORT_FILE"
            all_passed=0
        fi
    fi

    # Note: daemon and client are interactive and need special handling
    log_verbose "Skipping daemon/client execution tests (require runtime environment)"
    echo "  - daemon/client: Skipped (require runtime environment)" >> "$REPORT_FILE"

    if [[ $all_passed -eq 1 ]]; then
        log_success "All execution tests passed"
        return 0
    else
        log_error "Some execution tests failed"
        return 1
    fi
}

# Check for debug symbols (should be stripped)
check_stripped() {
    log_info "Checking if binaries are stripped..."

    local release_dir=$(find_release_dir)

    echo "" >> "$REPORT_FILE"
    echo "Debug Symbol Check:" >> "$REPORT_FILE"
    echo "------------------" >> "$REPORT_FILE"

    for binary in "${BINARIES[@]}"; do
        local binary_path="$release_dir/$binary"

        if [[ ! -f "$binary_path" ]]; then
            continue
        fi

        # Platform-specific stripping check
        if command -v file &> /dev/null; then
            local file_output=$(file "$binary_path")

            if echo "$file_output" | grep -q "not stripped"; then
                log_warning "$binary is not stripped (contains debug symbols)"
                echo "  ⚠ $binary - NOT STRIPPED" >> "$REPORT_FILE"
            else
                log_success "$binary is properly stripped"
                echo "  ✓ $binary - Stripped" >> "$REPORT_FILE"
            fi
        else
            log_verbose "Cannot check strip status (file command not available)"
            echo "  ? $binary - Unknown (file command not available)" >> "$REPORT_FILE"
        fi
    done
}

# Verify workspace configuration
check_workspace_config() {
    log_info "Verifying workspace configuration..."

    echo "" >> "$REPORT_FILE"
    echo "Workspace Configuration:" >> "$REPORT_FILE"
    echo "-----------------------" >> "$REPORT_FILE"

    cd "$PROJECT_ROOT"

    # Check release profile in Cargo.toml
    if grep -q '\[profile.release\]' Cargo.toml; then
        log_verbose "Found release profile configuration"

        # Extract profile settings
        local lto=$(grep -A5 '\[profile.release\]' Cargo.toml | grep 'lto' | head -n1 || echo "lto not set")
        local opt_level=$(grep -A5 '\[profile.release\]' Cargo.toml | grep 'opt-level' | head -n1 || echo "opt-level not set")
        local strip=$(grep -A5 '\[profile.release\]' Cargo.toml | grep 'strip' | head -n1 || echo "strip not set")

        echo "  LTO: $lto" >> "$REPORT_FILE"
        echo "  Optimization: $opt_level" >> "$REPORT_FILE"
        echo "  Strip: $strip" >> "$REPORT_FILE"

        log_success "Release profile configured"
    else
        log_warning "No release profile configuration found"
        echo "  WARNING: No release profile found" >> "$REPORT_FILE"
    fi
}

# Run release verification tests
run_verification_tests() {
    log_info "Running release verification tests..."

    echo "" >> "$REPORT_FILE"
    echo "Verification Tests:" >> "$REPORT_FILE"
    echo "------------------" >> "$REPORT_FILE"

    cd "$PROJECT_ROOT"

    if cargo test --test release_verification 2>&1 | tee -a "$REPORT_FILE"; then
        log_success "Verification tests passed"
        return 0
    else
        log_error "Verification tests failed"
        return 1
    fi
}

# Generate summary
generate_summary() {
    echo "" >> "$REPORT_FILE"
    echo "========================================" >> "$REPORT_FILE"
    echo "Verification Summary" >> "$REPORT_FILE"
    echo "========================================" >> "$REPORT_FILE"

    local release_dir=$(find_release_dir)
    local total_size=0

    for binary in "${BINARIES[@]}"; do
        local binary_path="$release_dir/$binary"
        if [[ -f "$binary_path" ]]; then
            local size=$(stat -f%z "$binary_path" 2>/dev/null || stat -c%s "$binary_path" 2>/dev/null)
            total_size=$((total_size + size))
        fi
    done

    local total_mb=$(echo "scale=2; $total_size / 1024 / 1024" | bc)

    echo "Total binary size: $total_mb MB" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "Report generated: $(date)" >> "$REPORT_FILE"

    log_info "Report saved to: $REPORT_FILE"
}

# Main execution
main() {
    log_info "Starting release verification..."
    log_info "Project root: $PROJECT_ROOT"

    # Initialize report
    init_report

    # Clean if requested
    if [[ $CLEAN -eq 1 ]]; then
        clean_build
    fi

    # Build unless skipped
    if [[ $SKIP_BUILD -eq 0 ]]; then
        if ! build_release; then
            log_error "Build failed, aborting verification"
            exit 1
        fi
    else
        log_info "Skipping build step"
    fi

    # Verify binaries exist
    if ! verify_binaries; then
        log_error "Binary verification failed"
        exit 1
    fi

    # Test binary execution
    test_binaries

    # Check if stripped
    check_stripped

    # Verify workspace config
    check_workspace_config

    # Run verification tests
    run_verification_tests

    # Generate summary
    generate_summary

    log_success "Release verification complete!"
    log_info "Full report: $REPORT_FILE"
}

# Run main function
main
