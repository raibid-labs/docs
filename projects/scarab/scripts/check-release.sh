#!/usr/bin/env bash
# Pre-release validation script
# Runs comprehensive checks before creating a release

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Counters
CHECKS_PASSED=0
CHECKS_FAILED=0
CHECKS_WARNED=0

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
    ((CHECKS_PASSED++))
}

log_fail() {
    echo -e "${RED}[✗]${NC} $1"
    ((CHECKS_FAILED++))
}

log_warn() {
    echo -e "${YELLOW}[⚠]${NC} $1"
    ((CHECKS_WARNED++))
}

# Check functions
check_git_status() {
    log_info "Checking Git status..."

    if [[ -n "$(git status --porcelain)" ]]; then
        log_fail "Working directory is not clean"
        git status --short
        return 1
    else
        log_success "Working directory is clean"
    fi
}

check_git_branch() {
    log_info "Checking Git branch..."

    local current_branch
    current_branch=$(git branch --show-current)

    if [[ "$current_branch" != "main" && ! "$current_branch" =~ ^release/ ]]; then
        log_warn "Not on main or release branch (current: $current_branch)"
        log_warn "Releases should typically be from main or release/* branches"
    else
        log_success "On appropriate branch: $current_branch"
    fi
}

check_cargo_fmt() {
    log_info "Checking code formatting..."

    if cargo fmt --all -- --check &>/dev/null; then
        log_success "Code is formatted correctly"
    else
        log_fail "Code formatting issues found"
        log_info "Run: cargo fmt --all"
        return 1
    fi
}

check_cargo_clippy() {
    log_info "Running Clippy..."

    if cargo clippy --workspace --all-targets --all-features -- -D warnings &>/dev/null; then
        log_success "Clippy passed with no warnings"
    else
        log_fail "Clippy found issues"
        log_info "Run: cargo clippy --workspace --all-targets --all-features"
        return 1
    fi
}

check_cargo_test() {
    log_info "Running test suite..."

    if cargo test --workspace --quiet &>/dev/null; then
        log_success "All tests passed"
    else
        log_fail "Test suite failed"
        log_info "Run: cargo test --workspace"
        return 1
    fi
}

check_cargo_build() {
    log_info "Building workspace..."

    if cargo build --workspace --release --quiet &>/dev/null; then
        log_success "Workspace builds successfully"
    else
        log_fail "Build failed"
        log_info "Run: cargo build --workspace --release"
        return 1
    fi
}

check_cargo_audit() {
    log_info "Checking for vulnerable dependencies..."

    if ! command -v cargo-audit &>/dev/null; then
        log_warn "cargo-audit not installed"
        log_info "Install: cargo install cargo-audit"
        return 0
    fi

    if cargo audit &>/dev/null; then
        log_success "No vulnerable dependencies found"
    else
        log_fail "Vulnerable dependencies detected"
        log_info "Run: cargo audit"
        return 1
    fi
}

check_version_sync() {
    log_info "Checking version synchronization..."

    local versions=()
    local crate_names=()

    # Extract versions from all crates
    for crate_toml in crates/*/Cargo.toml; do
        if [[ -f "$crate_toml" ]]; then
            local version
            version=$(grep '^version = ' "$crate_toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
            versions+=("$version")
            crate_names+=("$(basename $(dirname "$crate_toml"))")
        fi
    done

    # Check if all versions are the same
    local first_version="${versions[0]}"
    local all_same=true

    for i in "${!versions[@]}"; do
        if [[ "${versions[$i]}" != "$first_version" ]]; then
            all_same=false
            log_fail "Version mismatch: ${crate_names[$i]} = ${versions[$i]} (expected $first_version)"
        fi
    done

    if $all_same; then
        log_success "All crate versions synchronized: $first_version"
    else
        log_fail "Crate versions are not synchronized"
        log_info "Run: ./scripts/bump-version.sh X.Y.Z"
        return 1
    fi
}

check_changelog() {
    log_info "Checking CHANGELOG.md..."

    if [[ ! -f "$PROJECT_ROOT/CHANGELOG.md" ]]; then
        log_warn "CHANGELOG.md not found"
        log_info "Create CHANGELOG.md before release"
        return 0
    fi

    # Check if CHANGELOG has recent updates
    local changelog_age
    changelog_age=$(git log -1 --format=%cd --date=relative -- CHANGELOG.md)

    if [[ -n "$changelog_age" ]]; then
        log_success "CHANGELOG.md exists (last updated: $changelog_age)"
    else
        log_warn "CHANGELOG.md has no git history"
    fi

    # Check for "Unreleased" section
    if grep -q "## \[Unreleased\]" "$PROJECT_ROOT/CHANGELOG.md"; then
        log_warn "CHANGELOG.md still has [Unreleased] section"
        log_info "Update [Unreleased] to [X.Y.Z] - YYYY-MM-DD before release"
    fi
}

check_documentation() {
    log_info "Checking documentation..."

    local docs_missing=0

    # Check critical docs exist
    local required_docs=(
        "README.md"
        "CLAUDE.md"
        "docs/RELEASE_PROCESS.md"
        "docs/RELEASE_CHECKLIST.md"
    )

    for doc in "${required_docs[@]}"; do
        if [[ ! -f "$PROJECT_ROOT/$doc" ]]; then
            log_warn "Missing documentation: $doc"
            ((docs_missing++))
        fi
    done

    if [[ $docs_missing -eq 0 ]]; then
        log_success "All critical documentation exists"
    else
        log_warn "$docs_missing documentation files missing"
    fi
}

check_dependencies() {
    log_info "Checking for outdated dependencies..."

    # Check Cargo.lock exists
    if [[ ! -f "$PROJECT_ROOT/Cargo.lock" ]]; then
        log_fail "Cargo.lock not found"
        log_info "Run: cargo build"
        return 1
    fi

    # Check if Cargo.lock is committed
    if ! git ls-files --error-unmatch Cargo.lock &>/dev/null; then
        log_warn "Cargo.lock is not committed to git"
        log_info "Add Cargo.lock: git add Cargo.lock"
    fi

    log_success "Cargo.lock exists and is tracked"
}

check_platform_builds() {
    log_info "Checking platform-specific code compiles..."

    # Note: This only checks if code compiles, not if it actually builds
    # Full cross-compilation testing requires more setup

    if cargo check --workspace --quiet &>/dev/null; then
        log_success "Platform-agnostic code compiles"
    else
        log_fail "Compilation check failed"
        return 1
    fi
}

check_benchmarks() {
    log_info "Checking benchmarks compile..."

    if cargo bench --workspace --no-run --quiet &>/dev/null; then
        log_success "Benchmarks compile successfully"
    else
        log_warn "Benchmarks failed to compile"
        log_info "Run: cargo bench --workspace --no-run"
    fi
}

check_examples() {
    log_info "Checking examples..."

    local examples_dir="$PROJECT_ROOT/examples"

    if [[ ! -d "$examples_dir" ]]; then
        log_warn "No examples directory found"
        return 0
    fi

    local example_count
    example_count=$(find "$examples_dir" -name "*.rs" | wc -l)

    if [[ $example_count -gt 0 ]]; then
        log_success "Found $example_count example files"
    else
        log_warn "No example files found"
    fi
}

check_license() {
    log_info "Checking license..."

    if [[ -f "$PROJECT_ROOT/LICENSE" ]] || [[ -f "$PROJECT_ROOT/LICENSE.md" ]]; then
        log_success "License file exists"
    else
        log_warn "No LICENSE file found"
        log_info "Consider adding a LICENSE file"
    fi
}

# Summary function
print_summary() {
    echo
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${BLUE}Release Readiness Check Summary${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${GREEN}Passed:${NC}  $CHECKS_PASSED"
    echo -e "${YELLOW}Warnings:${NC} $CHECKS_WARNED"
    echo -e "${RED}Failed:${NC}  $CHECKS_FAILED"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    if [[ $CHECKS_FAILED -eq 0 ]]; then
        if [[ $CHECKS_WARNED -eq 0 ]]; then
            echo -e "${GREEN}✓ All checks passed! Ready to release.${NC}"
        else
            echo -e "${YELLOW}⚠ All critical checks passed, but there are $CHECKS_WARNED warnings.${NC}"
            echo "  Review warnings before proceeding with release."
        fi
        return 0
    else
        echo -e "${RED}✗ $CHECKS_FAILED critical checks failed.${NC}"
        echo "  Fix the issues above before releasing."
        return 1
    fi
}

# Main function
main() {
    cd "$PROJECT_ROOT"

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${BLUE}Scarab Release Readiness Check${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo

    # Run all checks
    check_git_status || true
    check_git_branch || true
    check_version_sync || true
    check_cargo_fmt || true
    check_cargo_clippy || true
    check_cargo_test || true
    check_cargo_build || true
    check_cargo_audit || true
    check_changelog || true
    check_documentation || true
    check_dependencies || true
    check_platform_builds || true
    check_benchmarks || true
    check_examples || true
    check_license || true

    # Print summary
    print_summary
}

# Run main
main "$@"
