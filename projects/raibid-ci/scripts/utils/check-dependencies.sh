#!/bin/bash
################################################################################
# Script Name: check-dependencies.sh
# Description: Check system dependencies for all raibid-ci scripts
# Author: raibid-ci project
# Created: 2025-11-01
# Modified: 2025-11-01
#
# Usage: ./check-dependencies.sh [OPTIONS]
#
# Options:
#   -h, --help       Show this help message
#   -v, --verbose    Enable verbose output
#   -f, --fix        Show installation commands for missing deps
#
# Examples:
#   ./check-dependencies.sh
#   ./check-dependencies.sh --verbose --fix
#
# Dependencies:
#   - bash >= 4.0
#
# Exit Codes:
#   0 - All dependencies satisfied
#   1 - Some dependencies missing
################################################################################

set -euo pipefail

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Color codes
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m'

# Flags
VERBOSE=false
SHOW_FIX=false

################################################################################
# Helper Functions
################################################################################

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

show_usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Description:
    Check system dependencies for all raibid-ci scripts.

Options:
    -h, --help       Show this help message and exit
    -v, --verbose    Enable verbose output
    -f, --fix        Show installation commands for missing deps

Examples:
    $(basename "$0")
    $(basename "$0") --verbose --fix

Exit Codes:
    0 - All dependencies satisfied
    1 - Some dependencies missing

EOF
}

command_exists() {
    command -v "$1" &> /dev/null
}

get_install_cmd() {
    local dep=$1
    case "$dep" in
        jq)
            echo "sudo apt install jq  # or: brew install jq"
            ;;
        gh)
            echo "Visit: https://cli.github.com/ or: brew install gh"
            ;;
        nu|nushell)
            echo "Visit: https://www.nushell.sh/ or: brew install nushell"
            ;;
        shellcheck)
            echo "sudo apt install shellcheck  # or: brew install shellcheck"
            ;;
        cargo|rustc)
            echo "Visit: https://rustup.rs/ or: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
            ;;
        *)
            echo "sudo apt install $dep  # or: brew install $dep"
            ;;
    esac
}

################################################################################
# Dependency Checking
################################################################################

check_dependencies() {
    log_info "Checking dependencies for raibid-ci scripts..."
    echo ""

    local missing=()
    local optional_missing=()

    # Core dependencies
    log_info "Core dependencies:"

    if command_exists "bash"; then
        local bash_version=$(bash --version | head -n1 | grep -oP '\d+\.\d+' | head -1)
        log_success "bash $bash_version"
    else
        log_error "bash (required)"
        missing+=("bash")
    fi

    if command_exists "jq"; then
        local jq_version=$(jq --version | grep -oP '\d+\.\d+' || echo "unknown")
        log_success "jq $jq_version"
    else
        log_error "jq (for JSON processing)"
        missing+=("jq")
    fi

    if command_exists "gh"; then
        local gh_version=$(gh --version | head -n1 | grep -oP '\d+\.\d+\.\d+')
        log_success "gh $gh_version"

        # Check gh authentication
        if gh auth status &> /dev/null; then
            log_success "gh authenticated"
        else
            log_warning "gh not authenticated (run: gh auth login)"
        fi
    else
        log_error "gh (GitHub CLI)"
        missing+=("gh")
    fi

    echo ""
    log_info "Optional dependencies:"

    if command_exists "nu"; then
        local nu_version=$(nu --version | grep -oP '\d+\.\d+\.\d+' || echo "unknown")
        log_success "nushell $nu_version"
    else
        log_warning "nushell (for .nu scripts)"
        optional_missing+=("nu")
    fi

    if command_exists "shellcheck"; then
        local sc_version=$(shellcheck --version | grep -oP 'version: \K[\d.]+')
        log_success "shellcheck $sc_version"
    else
        log_warning "shellcheck (for linting)"
        optional_missing+=("shellcheck")
    fi

    echo ""
    log_info "Build dependencies (for Rust project):"

    if command_exists "cargo"; then
        local cargo_version=$(cargo --version | grep -oP '\d+\.\d+\.\d+')
        log_success "cargo $cargo_version"
    else
        log_warning "cargo (Rust build tool)"
        optional_missing+=("cargo")
    fi

    if command_exists "rustc"; then
        local rustc_version=$(rustc --version | grep -oP '\d+\.\d+\.\d+')
        log_success "rustc $rustc_version"
    else
        log_warning "rustc (Rust compiler)"
        optional_missing+=("rustc")
    fi

    # Summary
    echo ""
    echo "========================================"

    if [[ ${#missing[@]} -eq 0 ]]; then
        log_success "All required dependencies satisfied!"

        if [[ ${#optional_missing[@]} -gt 0 ]]; then
            echo ""
            log_warning "${#optional_missing[@]} optional dependencies missing (non-critical)"

            if [[ "$SHOW_FIX" == "true" ]]; then
                echo ""
                log_info "To install optional dependencies:"
                for dep in "${optional_missing[@]}"; do
                    echo "  $dep: $(get_install_cmd "$dep")"
                done
            fi
        fi

        echo ""
        return 0
    else
        echo ""
        log_error "${#missing[@]} required dependencies missing!"

        if [[ "$SHOW_FIX" == "true" ]]; then
            echo ""
            log_info "To install missing dependencies:"
            for dep in "${missing[@]}"; do
                echo "  $dep: $(get_install_cmd "$dep")"
            done
        else
            echo "Run with --fix to see installation commands"
        fi

        echo ""
        return 1
    fi
}

################################################################################
# Argument Parsing
################################################################################

parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h|--help)
                show_usage
                exit 0
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -f|--fix)
                SHOW_FIX=true
                shift
                ;;
            -*)
                log_error "Unknown option: $1"
                show_usage
                exit 2
                ;;
            *)
                log_error "Unexpected argument: $1"
                show_usage
                exit 2
                ;;
        esac
    done
}

################################################################################
# Main
################################################################################

parse_arguments "$@"
check_dependencies
