#!/usr/bin/env bash

#############################################################################
# init-repo.sh - Bootstrap a new repository with raibid-labs org config
#############################################################################
#
# This script initializes a repository with the appropriate Claude Code
# configuration based on repo type, extending the org-wide standards.
#
# Usage:
#   ./init-repo.sh [repo-path] [repo-type]
#
# Arguments:
#   repo-path   - Path to the repository (default: current directory)
#   repo-type   - Type of repository: web, mobile, ml, devops, lib (auto-detected if omitted)
#
# Examples:
#   ./init-repo.sh                          # Auto-detect in current dir
#   ./init-repo.sh /path/to/repo web        # Initialize web project
#   ./init-repo.sh ../my-api devops         # Initialize devops project
#
#############################################################################

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ORG_CONFIG_REPO="raibid-labs/claude-org-config"
ORG_CONFIG_PATH=".claude/org-config"

#############################################################################
# Helper Functions
#############################################################################

print_usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS] [repo-path] [repo-type]

Bootstrap a repository with raibid-labs org config.

Arguments:
    repo-path       Path to repository (default: current directory)
    repo-type       Type: web, mobile, ml, devops, lib (auto-detected if omitted)

Options:
    -h, --help      Show this help message
    -v, --verbose   Enable verbose output
    -f, --force     Overwrite existing configuration
    --dry-run       Show what would be done without making changes

Examples:
    $(basename "$0")                          # Auto-detect in current dir
    $(basename "$0") /path/to/repo web        # Initialize web project
    $(basename "$0") --force ../my-api        # Force reinitialize

Supported repo types:
    web      - Web applications (React, Vue, Angular, etc.)
    mobile   - Mobile applications (React Native, Flutter, etc.)
    ml       - Machine Learning projects
    devops   - DevOps, infrastructure, and tooling
    lib      - Libraries and packages

EOF
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

#############################################################################
# Repository Type Detection
#############################################################################

detect_repo_type() {
    local repo_path="$1"

    # Check for package.json indicators
    if [[ -f "$repo_path/package.json" ]]; then
        local package_content
        package_content=$(cat "$repo_path/package.json")

        # Check for React/Vue/Angular
        if echo "$package_content" | grep -q '"react":\|"vue":\|"@angular/core":'; then
            echo "web"
            return
        fi

        # Check for React Native/Flutter
        if echo "$package_content" | grep -q '"react-native":\|"expo":'; then
            echo "mobile"
            return
        fi
    fi

    # Check for Python ML indicators
    if [[ -f "$repo_path/requirements.txt" ]] || [[ -f "$repo_path/pyproject.toml" ]]; then
        if grep -q "tensorflow\|pytorch\|scikit-learn\|keras" "$repo_path/requirements.txt" 2>/dev/null; then
            echo "ml"
            return
        fi
    fi

    # Check for Flutter
    if [[ -f "$repo_path/pubspec.yaml" ]]; then
        if grep -q "flutter:" "$repo_path/pubspec.yaml"; then
            echo "mobile"
            return
        fi
    fi

    # Check for Terraform/Ansible/Docker
    if [[ -f "$repo_path/terraform.tf" ]] || [[ -f "$repo_path/ansible.cfg" ]] || [[ -f "$repo_path/docker-compose.yml" ]]; then
        echo "devops"
        return
    fi

    # Check for setup.py (Python library)
    if [[ -f "$repo_path/setup.py" ]]; then
        echo "lib"
        return
    fi

    # Default to lib for generic projects
    echo "lib"
}

#############################################################################
# Template Generation
#############################################################################

generate_project_config() {
    local repo_type="$1"
    local repo_name="$2"

    cat << EOF
{
  "extends": "claude-org-config/.claude/project.json",
  "name": "$repo_name",
  "description": "Repository initialized with raibid-labs org config",
  "type": "$repo_type",
  "customInstructions": [
    "Follow raibid-labs coding standards and best practices",
    "Use SPARC methodology for development",
    "Coordinate work using Claude Flow swarm architecture"
  ],
  "mcpServers": {
    "claude-flow": {
      "command": "npx",
      "args": ["claude-flow@alpha", "mcp", "start"],
      "env": {
        "CLAUDE_FLOW_SESSION": "$repo_name-session"
      }
    }
  }
}
EOF
}

generate_claude_md() {
    local repo_type="$1"
    local repo_name="$2"

    cat << 'EOF'
# Claude Code Configuration

This repository extends the raibid-labs organization configuration.

## Project Configuration

This project is configured to use:
- SPARC methodology for systematic development
- Claude Flow for agent coordination
- Organization-wide coding standards

## Key Commands

### SPARC Workflow
```bash
# List available SPARC modes
npx claude-flow sparc modes

# Run specific mode
npx claude-flow sparc run <mode> "<task>"

# Full TDD workflow
npx claude-flow sparc tdd "<feature>"
```

### Agent Coordination
```bash
# Initialize swarm
npx claude-flow@alpha hooks pre-task --description "task description"

# Session management
npx claude-flow@alpha hooks session-restore --session-id "session-id"
npx claude-flow@alpha hooks session-end --export-metrics true
```

## Development Guidelines

1. Always use concurrent operations in single messages
2. Never save working files to root folder
3. Organize files in appropriate subdirectories
4. Use Claude Code's Task tool for agent spawning
5. Follow SPARC methodology phases

## File Organization

- `/src` - Source code files
- `/tests` - Test files
- `/docs` - Documentation
- `/config` - Configuration files
- `/scripts` - Utility scripts
- `/examples` - Example code

## Support

For org-wide configuration issues, see:
https://github.com/raibid-labs/claude-org-config
EOF
}

#############################################################################
# Validation
#############################################################################

validate_setup() {
    local repo_path="$1"
    local errors=0

    log_info "Validating setup..."

    # Check .claude directory
    if [[ ! -d "$repo_path/.claude" ]]; then
        log_error ".claude directory not found"
        ((errors++))
    fi

    # Check project.json
    if [[ ! -f "$repo_path/.claude/project.json" ]]; then
        log_error ".claude/project.json not found"
        ((errors++))
    else
        # Validate JSON syntax
        if ! jq empty "$repo_path/.claude/project.json" 2>/dev/null; then
            log_error "Invalid JSON in .claude/project.json"
            ((errors++))
        fi

        # Check for extends field
        if ! jq -e '.extends' "$repo_path/.claude/project.json" >/dev/null 2>&1; then
            log_warn "project.json does not extend org config"
        fi
    fi

    # Check CLAUDE.md
    if [[ ! -f "$repo_path/CLAUDE.md" ]]; then
        log_warn "CLAUDE.md not found (recommended but optional)"
    fi

    # Check git repository
    if [[ ! -d "$repo_path/.git" ]]; then
        log_warn "Not a git repository"
    fi

    if [[ $errors -eq 0 ]]; then
        log_success "Validation passed"
        return 0
    else
        log_error "Validation failed with $errors error(s)"
        return 1
    fi
}

#############################################################################
# Main Script
#############################################################################

main() {
    local repo_path="${1:-.}"
    local repo_type="${2:-}"
    local force=false
    local dry_run=false
    local verbose=false

    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                print_usage
                exit 0
                ;;
            -f|--force)
                force=true
                shift
                ;;
            --dry-run)
                dry_run=true
                shift
                ;;
            -v|--verbose)
                verbose=true
                set -x
                shift
                ;;
            -*)
                log_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
            *)
                if [[ -z "$repo_path" ]] || [[ "$repo_path" == "." ]]; then
                    repo_path="$1"
                elif [[ -z "$repo_type" ]]; then
                    repo_type="$1"
                fi
                shift
                ;;
        esac
    done

    # Resolve absolute path
    repo_path=$(cd "$repo_path" 2>/dev/null && pwd || echo "$repo_path")

    # Validate repo path exists
    if [[ ! -d "$repo_path" ]]; then
        log_error "Repository path does not exist: $repo_path"
        exit 1
    fi

    log_info "Initializing repository at: $repo_path"

    # Get repository name
    local repo_name
    repo_name=$(basename "$repo_path")

    # Auto-detect repo type if not provided
    if [[ -z "$repo_type" ]]; then
        log_info "Auto-detecting repository type..."
        repo_type=$(detect_repo_type "$repo_path")
        log_info "Detected repository type: $repo_type"
    fi

    # Validate repo type
    case "$repo_type" in
        web|mobile|ml|devops|lib)
            ;;
        *)
            log_error "Invalid repo type: $repo_type"
            log_error "Must be one of: web, mobile, ml, devops, lib"
            exit 1
            ;;
    esac

    # Check if already initialized
    if [[ -f "$repo_path/.claude/project.json" ]] && [[ "$force" != true ]]; then
        log_warn "Repository already initialized. Use --force to overwrite."
        exit 0
    fi

    if [[ "$dry_run" == true ]]; then
        log_info "[DRY RUN] Would create .claude directory structure"
        log_info "[DRY RUN] Would generate project.json for type: $repo_type"
        log_info "[DRY RUN] Would create CLAUDE.md"
        exit 0
    fi

    # Create .claude directory
    log_info "Creating .claude directory..."
    mkdir -p "$repo_path/.claude"

    # Generate project.json
    log_info "Generating project.json..."
    generate_project_config "$repo_type" "$repo_name" > "$repo_path/.claude/project.json"

    # Generate CLAUDE.md if it doesn't exist
    if [[ ! -f "$repo_path/CLAUDE.md" ]] || [[ "$force" == true ]]; then
        log_info "Generating CLAUDE.md..."
        generate_claude_md "$repo_type" "$repo_name" > "$repo_path/CLAUDE.md"
    fi

    # Create .gitignore entry for Claude Code
    if [[ -f "$repo_path/.gitignore" ]]; then
        if ! grep -q ".claude/\*" "$repo_path/.gitignore"; then
            log_info "Adding .claude/* to .gitignore..."
            echo "" >> "$repo_path/.gitignore"
            echo "# Claude Code local state" >> "$repo_path/.gitignore"
            echo ".claude/*" >> "$repo_path/.gitignore"
            echo "!.claude/project.json" >> "$repo_path/.gitignore"
        fi
    fi

    # Validate setup
    if validate_setup "$repo_path"; then
        log_success "Repository initialized successfully!"
        echo ""
        log_info "Next steps:"
        echo "  1. Review .claude/project.json and customize as needed"
        echo "  2. Review CLAUDE.md for usage instructions"
        echo "  3. Run: npx claude-flow sparc modes"
        echo ""
        log_info "Repository type: $repo_type"
        log_info "Configuration extends: $ORG_CONFIG_REPO"
    else
        log_error "Initialization completed with warnings"
        exit 1
    fi
}

# Run main function
main "$@"
