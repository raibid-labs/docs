#!/usr/bin/env bash
# Scarab Plugin Validator
# Validates plugin structure, metadata, and API compatibility

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# API version from scarab-plugin-api
CURRENT_API_VERSION="0.1.0"

# Validation statistics
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0
WARNINGS=0

# Print usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS] <plugin.fsx|plugin.fzb>

Validate Fusabi plugin structure and metadata

OPTIONS:
    -h, --help              Show this help message
    -v, --verbose           Enable verbose output
    -s, --strict            Enable strict validation (warnings become errors)
    -a, --all               Validate all plugins in examples/fusabi/
    -j, --json              Output results in JSON format
    --api-version VERSION   Override API version check (default: $CURRENT_API_VERSION)

EXAMPLES:
    # Validate single plugin
    $0 examples/fusabi/hello.fsx

    # Validate all plugins with strict mode
    $0 --strict --all

    # Validate with JSON output
    $0 --json examples/fusabi/theme.fsx
EOF
}

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $*"
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $*"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
    WARNINGS=$((WARNINGS + 1))
}

log_check() {
    echo -e "${CYAN}[CHECK]${NC} $*"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
}

# Extract metadata from plugin file
extract_metadata() {
    local plugin_file="$1"
    local key="$2"

    # Look for metadata comments: // @key value
    grep -i "^[[:space:]]*//.*@$key" "$plugin_file" | head -1 | sed -E "s/^[[:space:]]*\/\/[[:space:]]*@$key[[:space:]]*:?[[:space:]]*//" || echo ""
}

# Validate plugin metadata
validate_metadata() {
    local plugin_file="$1"

    log_info "Validating metadata for $(basename "$plugin_file")"

    # Required metadata fields
    local name version description author

    name=$(extract_metadata "$plugin_file" "name")
    version=$(extract_metadata "$plugin_file" "version")
    description=$(extract_metadata "$plugin_file" "description")
    author=$(extract_metadata "$plugin_file" "author")

    # Check required fields
    log_check "Checking required metadata fields"

    local has_errors=0

    if [ -n "$name" ]; then
        log_success "Plugin name: $name"
    else
        log_fail "Missing required @name metadata"
        has_errors=1
    fi

    if [ -n "$version" ]; then
        # Validate semver format
        if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?$ ]]; then
            log_success "Plugin version: $version (valid semver)"
        else
            log_warn "Version '$version' is not valid semver format"
        fi
    else
        log_fail "Missing required @version metadata"
        has_errors=1
    fi

    if [ -n "$description" ]; then
        log_success "Plugin description: $description"
    else
        log_warn "Missing recommended @description metadata"
    fi

    if [ -n "$author" ]; then
        log_success "Plugin author: $author"
    else
        log_warn "Missing recommended @author metadata"
    fi

    # Optional fields
    local homepage license api_version min_scarab_version

    homepage=$(extract_metadata "$plugin_file" "homepage")
    license=$(extract_metadata "$plugin_file" "license")
    api_version=$(extract_metadata "$plugin_file" "api-version")
    min_scarab_version=$(extract_metadata "$plugin_file" "min-scarab-version")

    if [ -n "$homepage" ]; then
        log_info "Homepage: $homepage"
    fi

    if [ -n "$license" ]; then
        log_info "License: $license"
    fi

    if [ -n "$api_version" ]; then
        # Check API compatibility
        log_check "Checking API version compatibility"
        if [ "$api_version" == "$CURRENT_API_VERSION" ]; then
            log_success "API version $api_version matches current version"
        else
            log_warn "API version $api_version differs from current $CURRENT_API_VERSION"
        fi
    fi

    if [ -n "$min_scarab_version" ]; then
        log_info "Minimum Scarab version: $min_scarab_version"
    fi

    return $has_errors
}

# Validate plugin structure
validate_structure() {
    local plugin_file="$1"

    log_info "Validating plugin structure"

    # Check file exists and is readable
    log_check "Checking file accessibility"
    if [ -f "$plugin_file" ] && [ -r "$plugin_file" ]; then
        log_success "File is accessible"
    else
        log_fail "File is not accessible or does not exist"
        return 1
    fi

    # Check file size
    log_check "Checking file size"
    local file_size
    file_size=$(stat -f%z "$plugin_file" 2>/dev/null || stat -c%s "$plugin_file" 2>/dev/null)
    if [ "$file_size" -gt 0 ]; then
        log_success "File size: $file_size bytes"
    else
        log_fail "File is empty"
        return 1
    fi

    # Check for syntax errors (basic check)
    log_check "Checking basic syntax"

    # Look for common F# syntax patterns
    local has_module=0
    local has_function=0

    if grep -q "^[[:space:]]*module " "$plugin_file"; then
        has_module=1
    fi

    if grep -q "^[[:space:]]*let " "$plugin_file"; then
        has_function=1
        log_success "Found function definitions"
    else
        log_warn "No function definitions found"
    fi

    # Check for plugin lifecycle hooks
    log_check "Checking for plugin hooks"
    local hooks_found=0

    for hook in "on_load" "on_unload" "on_output" "on_input" "on_resize"; do
        if grep -q "$hook" "$plugin_file"; then
            log_info "Found hook: $hook"
            hooks_found=$((hooks_found + 1))
        fi
    done

    if [ $hooks_found -gt 0 ]; then
        log_success "Found $hooks_found plugin hook(s)"
    else
        log_warn "No plugin hooks found - plugin may not interact with Scarab"
    fi

    return 0
}

# Validate plugin API compatibility
validate_api_compatibility() {
    local plugin_file="$1"

    log_info "Validating API compatibility"

    # Check for deprecated API usage
    log_check "Checking for deprecated API usage"

    local deprecated_found=0
    local deprecated_patterns=(
        "PluginV1"
        "old_api"
        "deprecated"
    )

    for pattern in "${deprecated_patterns[@]}"; do
        if grep -qi "$pattern" "$plugin_file"; then
            log_warn "Found potentially deprecated API usage: $pattern"
            deprecated_found=$((deprecated_found + 1))
        fi
    done

    if [ $deprecated_found -eq 0 ]; then
        log_success "No deprecated API usage detected"
    fi

    # Check for required imports
    log_check "Checking for required imports"

    # In F#, we might see 'open' statements
    if grep -q "^[[:space:]]*open " "$plugin_file"; then
        log_info "Found module imports"
    else
        log_info "No explicit imports found (may use qualified names)"
    fi

    return 0
}

# Run all validations on a plugin
validate_plugin() {
    local plugin_file="$1"

    echo ""
    echo "========================================"
    log_info "Validating plugin: $(basename "$plugin_file")"
    echo "========================================"

    local validation_failed=0

    # Reset counters for this plugin
    local plugin_checks=$TOTAL_CHECKS
    local plugin_passed=$PASSED_CHECKS
    local plugin_failed=$FAILED_CHECKS
    local plugin_warnings=$WARNINGS

    # Run validation steps
    if ! validate_structure "$plugin_file"; then
        validation_failed=1
    fi

    echo ""

    if ! validate_metadata "$plugin_file"; then
        validation_failed=1
    fi

    echo ""

    validate_api_compatibility "$plugin_file"

    # Calculate results for this plugin
    local checks_run=$((TOTAL_CHECKS - plugin_checks))
    local checks_passed=$((PASSED_CHECKS - plugin_passed))
    local checks_failed=$((FAILED_CHECKS - plugin_failed))
    local checks_warned=$((WARNINGS - plugin_warnings))

    echo ""
    echo "========================================"
    log_info "Validation Summary for $(basename "$plugin_file"):"
    log_info "  Total checks:   $checks_run"
    log_info "  Passed:         $checks_passed"
    if [ $checks_failed -gt 0 ]; then
        log_fail "  Failed:         $checks_failed"
    fi
    if [ $checks_warned -gt 0 ]; then
        log_warn "  Warnings:       $checks_warned"
    fi
    echo "========================================"

    return $validation_failed
}

# Validate all plugins in directory
validate_all_plugins() {
    local plugin_dir="${1:-$PROJECT_ROOT/examples/fusabi}"

    log_info "Validating all plugins in $plugin_dir"

    if [ ! -d "$plugin_dir" ]; then
        log_fail "Plugin directory not found: $plugin_dir"
        return 1
    fi

    local count=0
    local success=0
    local failed=0

    while IFS= read -r -d '' plugin_file; do
        count=$((count + 1))

        if validate_plugin "$plugin_file"; then
            success=$((success + 1))
        else
            failed=$((failed + 1))
        fi

        echo ""
    done < <(find "$plugin_dir" -maxdepth 1 \( -name "*.fsx" -o -name "*.fzb" \) -type f -print0 | sort -z)

    echo ""
    echo "========================================"
    log_info "Overall Validation Summary:"
    log_info "  Total plugins:  $count"
    log_success "  Passed:         $success"
    if [ $failed -gt 0 ]; then
        log_fail "  Failed:         $failed"
    fi
    if [ $WARNINGS -gt 0 ]; then
        log_warn "  Total warnings: $WARNINGS"
    fi
    echo "========================================"

    return $((failed > 0 ? 1 : 0))
}

# Parse command line arguments
VERBOSE=0
STRICT=0
VALIDATE_ALL=0
JSON_OUTPUT=0
PLUGIN_FILE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=1
            shift
            ;;
        -s|--strict)
            STRICT=1
            shift
            ;;
        -a|--all)
            VALIDATE_ALL=1
            shift
            ;;
        -j|--json)
            JSON_OUTPUT=1
            shift
            ;;
        --api-version)
            CURRENT_API_VERSION="$2"
            shift 2
            ;;
        -*)
            log_fail "Unknown option: $1"
            usage
            exit 1
            ;;
        *)
            PLUGIN_FILE="$1"
            shift
            ;;
    esac
done

# Main execution
main() {
    log_info "Scarab Fusabi Plugin Validator"
    log_info "API Version: $CURRENT_API_VERSION"

    local exit_code=0

    if [ "$VALIDATE_ALL" -eq 1 ]; then
        if ! validate_all_plugins; then
            exit_code=1
        fi
    else
        if [ -z "$PLUGIN_FILE" ]; then
            log_fail "No plugin file specified"
            usage
            exit 1
        fi

        if ! validate_plugin "$PLUGIN_FILE"; then
            exit_code=1
        fi
    fi

    # In strict mode, warnings count as failures
    if [ "$STRICT" -eq 1 ] && [ $WARNINGS -gt 0 ]; then
        log_fail "Strict mode: $WARNINGS warning(s) treated as errors"
        exit_code=1
    fi

    exit $exit_code
}

main
