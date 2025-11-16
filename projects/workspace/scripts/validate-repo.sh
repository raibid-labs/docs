#!/usr/bin/env bash

#############################################################################
# validate-repo.sh - Validate repository compliance with org standards
#############################################################################
#
# This script checks if a repository follows raibid-labs organization
# standards and generates a compliance report.
#
# Usage:
#   ./validate-repo.sh [OPTIONS] [repo-path]
#
# Arguments:
#   repo-path   - Path to the repository (default: current directory)
#
# Examples:
#   ./validate-repo.sh                     # Validate current directory
#   ./validate-repo.sh /path/to/repo       # Validate specific repo
#   ./validate-repo.sh --strict            # Strict validation mode
#
#############################################################################

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Validation counters
ERRORS=0
WARNINGS=0
INFO=0

# Configuration
ORG_CONFIG_REPO="raibid-labs/claude-org-config"
STRICT_MODE=false
VERBOSE=false
OUTPUT_FORMAT="text" # text, json, markdown

#############################################################################
# Helper Functions
#############################################################################

print_usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS] [repo-path]

Validate repository compliance with raibid-labs org standards.

Arguments:
    repo-path           Path to repository (default: current directory)

Options:
    -h, --help          Show this help message
    -v, --verbose       Enable verbose output
    -s, --strict        Enable strict validation mode
    -f, --format        Output format: text, json, markdown (default: text)
    --fix               Automatically fix issues where possible
    --report FILE       Save report to file

Examples:
    $(basename "$0")                           # Validate current dir
    $(basename "$0") /path/to/repo             # Validate specific repo
    $(basename "$0") --strict --format json    # JSON output with strict checks
    $(basename "$0") --fix                     # Auto-fix issues

Validation Checks:
    - .claude/project.json exists and is valid
    - Extends org configuration
    - Required files present
    - MCP server configurations
    - Directory structure
    - Git repository setup
    - SPARC compliance

Exit Codes:
    0 - All validations passed
    1 - Warnings found (non-strict mode)
    2 - Errors found
    3 - Critical errors (missing required files)

EOF
}

log_error() {
    ((ERRORS++))
    if [[ "$OUTPUT_FORMAT" == "text" ]]; then
        echo -e "${RED}[ERROR]${NC} $*" >&2
    fi
}

log_warn() {
    ((WARNINGS++))
    if [[ "$OUTPUT_FORMAT" == "text" ]]; then
        echo -e "${YELLOW}[WARN]${NC} $*" >&2
    fi
}

log_info() {
    ((INFO++))
    if [[ "$OUTPUT_FORMAT" == "text" ]]; then
        echo -e "${BLUE}[INFO]${NC} $*"
    fi
}

log_success() {
    if [[ "$OUTPUT_FORMAT" == "text" ]]; then
        echo -e "${GREEN}[PASS]${NC} $*"
    fi
}

log_check() {
    if [[ "$OUTPUT_FORMAT" == "text" ]]; then
        echo -e "${MAGENTA}[CHECK]${NC} $*"
    fi
}

#############################################################################
# Validation Functions
#############################################################################

validate_directory_structure() {
    local repo_path="$1"

    log_check "Validating directory structure..."

    # Check for recommended directories
    local recommended_dirs=("src" "tests" "docs" "config" "scripts")
    local found_dirs=0

    for dir in "${recommended_dirs[@]}"; do
        if [[ -d "$repo_path/$dir" ]]; then
            ((found_dirs++))
        fi
    done

    if [[ $found_dirs -eq 0 ]]; then
        log_warn "No standard directories found (src, tests, docs, config, scripts)"
    elif [[ $found_dirs -lt 3 ]]; then
        log_info "Found $found_dirs standard directories (recommended: at least 3)"
    else
        log_success "Directory structure follows conventions"
    fi
}

validate_claude_config() {
    local repo_path="$1"

    log_check "Validating Claude configuration..."

    # Check .claude directory
    if [[ ! -d "$repo_path/.claude" ]]; then
        log_error ".claude directory not found"
        return 1
    fi

    # Check project.json
    if [[ ! -f "$repo_path/.claude/project.json" ]]; then
        log_error ".claude/project.json not found"
        return 1
    fi

    # Validate JSON syntax
    if ! jq empty "$repo_path/.claude/project.json" 2>/dev/null; then
        log_error "Invalid JSON syntax in .claude/project.json"
        return 1
    fi

    log_success "Claude configuration files present"

    # Check for extends field
    local extends_field
    extends_field=$(jq -r '.extends // empty' "$repo_path/.claude/project.json")

    if [[ -z "$extends_field" ]]; then
        log_error "project.json does not extend org config"
        return 1
    elif [[ "$extends_field" != *"claude-org-config"* ]]; then
        log_warn "project.json extends unknown config: $extends_field"
    else
        log_success "Extends org config correctly"
    fi

    # Check for required fields
    local required_fields=("name" "description" "type")
    for field in "${required_fields[@]}"; do
        if ! jq -e ".$field" "$repo_path/.claude/project.json" >/dev/null 2>&1; then
            log_warn "Missing recommended field: $field"
        fi
    done

    return 0
}

validate_mcp_servers() {
    local repo_path="$1"

    log_check "Validating MCP server configuration..."

    if [[ ! -f "$repo_path/.claude/project.json" ]]; then
        return 1
    fi

    # Check for mcpServers section
    if ! jq -e '.mcpServers' "$repo_path/.claude/project.json" >/dev/null 2>&1; then
        log_warn "No MCP servers configured"
        return 0
    fi

    # Check for claude-flow (required)
    if ! jq -e '.mcpServers."claude-flow"' "$repo_path/.claude/project.json" >/dev/null 2>&1; then
        log_error "claude-flow MCP server not configured (required)"
    else
        log_success "claude-flow MCP server configured"
    fi

    # List all configured servers
    local servers
    servers=$(jq -r '.mcpServers | keys[]' "$repo_path/.claude/project.json" 2>/dev/null || echo "")

    if [[ -n "$servers" ]]; then
        log_info "Configured MCP servers: $(echo "$servers" | tr '\n' ',' | sed 's/,$//')"
    fi
}

validate_git_setup() {
    local repo_path="$1"

    log_check "Validating git setup..."

    if [[ ! -d "$repo_path/.git" ]]; then
        log_error "Not a git repository"
        return 1
    fi

    log_success "Git repository initialized"

    # Check .gitignore
    if [[ ! -f "$repo_path/.gitignore" ]]; then
        log_warn ".gitignore file not found"
    else
        # Check if .claude/* is ignored
        if ! grep -q ".claude/\*" "$repo_path/.gitignore" 2>/dev/null; then
            log_warn ".claude/* not in .gitignore (local state should be ignored)"
        else
            log_success ".gitignore configured correctly"
        fi
    fi

    # Check for README
    if [[ ! -f "$repo_path/README.md" ]]; then
        log_warn "README.md not found"
    fi
}

validate_claude_md() {
    local repo_path="$1"

    log_check "Validating CLAUDE.md..."

    if [[ ! -f "$repo_path/CLAUDE.md" ]]; then
        log_warn "CLAUDE.md not found (recommended for project instructions)"
        return 0
    fi

    log_success "CLAUDE.md present"

    # Check for key sections
    local key_sections=("SPARC" "Agent" "File Organization")
    for section in "${key_sections[@]}"; do
        if ! grep -q "$section" "$repo_path/CLAUDE.md"; then
            log_info "CLAUDE.md missing section: $section"
        fi
    done
}

validate_sparc_compliance() {
    local repo_path="$1"

    log_check "Checking SPARC methodology compliance..."

    # Check for package.json with claude-flow
    if [[ -f "$repo_path/package.json" ]]; then
        if grep -q "claude-flow" "$repo_path/package.json" 2>/dev/null; then
            log_success "claude-flow dependency found"
        else
            log_info "claude-flow not in package.json (optional)"
        fi
    fi

    # Check for test directory
    if [[ ! -d "$repo_path/tests" ]] && [[ ! -d "$repo_path/test" ]] && [[ ! -d "$repo_path/__tests__" ]]; then
        log_warn "No test directory found (recommended for TDD)"
    else
        log_success "Test directory present"
    fi
}

validate_security() {
    local repo_path="$1"

    log_check "Running security checks..."

    # Check for common secret files that shouldn't be committed
    local sensitive_files=(".env" "credentials.json" "secrets.yaml" "config/secrets.yml")

    for file in "${sensitive_files[@]}"; do
        if [[ -f "$repo_path/$file" ]]; then
            if git -C "$repo_path" ls-files --error-unmatch "$file" 2>/dev/null; then
                log_error "Sensitive file tracked in git: $file"
            fi
        fi
    done

    # Check .gitignore for common patterns
    if [[ -f "$repo_path/.gitignore" ]]; then
        local patterns=(".env" "*.key" "*.pem" "credentials")
        for pattern in "${patterns[@]}"; do
            if ! grep -q "$pattern" "$repo_path/.gitignore" 2>/dev/null; then
                log_info "Consider adding '$pattern' to .gitignore"
            fi
        done
    fi
}

#############################################################################
# Report Generation
#############################################################################

generate_text_report() {
    echo ""
    echo "============================================"
    echo "  Repository Validation Report"
    echo "============================================"
    echo ""
    echo "Summary:"
    echo "  Errors:   $ERRORS"
    echo "  Warnings: $WARNINGS"
    echo "  Info:     $INFO"
    echo ""

    if [[ $ERRORS -eq 0 ]] && [[ $WARNINGS -eq 0 ]]; then
        echo -e "${GREEN}✓ Repository is fully compliant with org standards${NC}"
        return 0
    elif [[ $ERRORS -eq 0 ]]; then
        echo -e "${YELLOW}⚠ Repository is compliant with minor warnings${NC}"
        return 1
    else
        echo -e "${RED}✗ Repository has compliance issues${NC}"
        return 2
    fi
}

generate_json_report() {
    cat << EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "repository": "$1",
  "compliance": {
    "errors": $ERRORS,
    "warnings": $WARNINGS,
    "info": $INFO
  },
  "status": "$(if [[ $ERRORS -eq 0 ]]; then echo "compliant"; else echo "non-compliant"; fi)"
}
EOF
}

generate_markdown_report() {
    cat << EOF
# Repository Validation Report

**Generated:** $(date)
**Repository:** $1

## Summary

| Metric   | Count |
|----------|-------|
| Errors   | $ERRORS |
| Warnings | $WARNINGS |
| Info     | $INFO |

## Status

$(if [[ $ERRORS -eq 0 ]] && [[ $WARNINGS -eq 0 ]]; then
    echo "✅ **COMPLIANT** - Repository meets all org standards"
elif [[ $ERRORS -eq 0 ]]; then
    echo "⚠️ **COMPLIANT WITH WARNINGS** - Minor issues found"
else
    echo "❌ **NON-COMPLIANT** - Critical issues need attention"
fi)

## Validation Details

See console output for detailed check results.

---
Generated by raibid-labs/claude-org-config validation tools
EOF
}

#############################################################################
# Auto-fix Functions
#############################################################################

auto_fix_issues() {
    local repo_path="$1"

    log_info "Attempting to auto-fix issues..."

    # Fix .gitignore
    if [[ -f "$repo_path/.gitignore" ]]; then
        if ! grep -q ".claude/\*" "$repo_path/.gitignore"; then
            log_info "Adding .claude/* to .gitignore"
            echo "" >> "$repo_path/.gitignore"
            echo "# Claude Code local state" >> "$repo_path/.gitignore"
            echo ".claude/*" >> "$repo_path/.gitignore"
            echo "!.claude/project.json" >> "$repo_path/.gitignore"
        fi
    fi

    # Create missing directories
    local recommended_dirs=("tests" "docs")
    for dir in "${recommended_dirs[@]}"; do
        if [[ ! -d "$repo_path/$dir" ]]; then
            log_info "Creating $dir directory"
            mkdir -p "$repo_path/$dir"
        fi
    done

    log_success "Auto-fix completed"
}

#############################################################################
# Main Script
#############################################################################

main() {
    local repo_path="${1:-.}"
    local fix_mode=false
    local report_file=""

    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                print_usage
                exit 0
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -s|--strict)
                STRICT_MODE=true
                shift
                ;;
            -f|--format)
                OUTPUT_FORMAT="$2"
                shift 2
                ;;
            --fix)
                fix_mode=true
                shift
                ;;
            --report)
                report_file="$2"
                shift 2
                ;;
            -*)
                log_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
            *)
                repo_path="$1"
                shift
                ;;
        esac
    done

    # Resolve absolute path
    repo_path=$(cd "$repo_path" 2>/dev/null && pwd || echo "$repo_path")

    # Validate repo path exists
    if [[ ! -d "$repo_path" ]]; then
        echo -e "${RED}[ERROR]${NC} Repository path does not exist: $repo_path" >&2
        exit 3
    fi

    if [[ "$OUTPUT_FORMAT" == "text" ]]; then
        echo -e "${BLUE}=== Repository Validation ===${NC}"
        echo "Repository: $repo_path"
        echo ""
    fi

    # Run validations
    validate_claude_config "$repo_path"
    validate_mcp_servers "$repo_path"
    validate_directory_structure "$repo_path"
    validate_git_setup "$repo_path"
    validate_claude_md "$repo_path"
    validate_sparc_compliance "$repo_path"
    validate_security "$repo_path"

    # Auto-fix if requested
    if [[ "$fix_mode" == true ]]; then
        auto_fix_issues "$repo_path"
    fi

    # Generate report
    local exit_code=0

    case "$OUTPUT_FORMAT" in
        json)
            generate_json_report "$repo_path"
            ;;
        markdown)
            generate_markdown_report "$repo_path"
            ;;
        text|*)
            generate_text_report
            exit_code=$?
            ;;
    esac

    # Save report to file if requested
    if [[ -n "$report_file" ]]; then
        case "$OUTPUT_FORMAT" in
            json)
                generate_json_report "$repo_path" > "$report_file"
                ;;
            markdown)
                generate_markdown_report "$repo_path" > "$report_file"
                ;;
            text)
                generate_text_report > "$report_file"
                ;;
        esac
        log_info "Report saved to: $report_file"
    fi

    # Determine exit code
    if [[ $ERRORS -gt 0 ]]; then
        exit_code=2
    elif [[ $STRICT_MODE == true ]] && [[ $WARNINGS -gt 0 ]]; then
        exit_code=1
    fi

    exit $exit_code
}

# Run main function
main "$@"
