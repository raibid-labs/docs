#!/usr/bin/env bash

#############################################################################
# sync-to-repos.sh - Sync org config updates to all repositories
#############################################################################
#
# This script updates all raibid-labs repositories when org config changes,
# creating PRs where necessary.
#
# Usage:
#   ./sync-to-repos.sh [OPTIONS]
#
# Options:
#   --dry-run       Show what would be done without making changes
#   --repos LIST    Comma-separated list of repos to sync (default: all)
#   --skip LIST     Comma-separated list of repos to skip
#   --auto-merge    Auto-merge PRs that pass validation
#
# Examples:
#   ./sync-to-repos.sh --dry-run           # Preview changes
#   ./sync-to-repos.sh --repos repo1,repo2 # Sync specific repos
#   ./sync-to-repos.sh --skip archived-*   # Skip archived repos
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
ORG_NAME="raibid-labs"
ORG_CONFIG_REPO="claude-org-config"
WORK_DIR="/tmp/raibid-labs-sync"
BRANCH_NAME="sync/org-config-update-$(date +%Y%m%d-%H%M%S)"

# Counters
REPOS_PROCESSED=0
REPOS_UPDATED=0
REPOS_FAILED=0
REPOS_SKIPPED=0

#############################################################################
# Helper Functions
#############################################################################

print_usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Sync org config updates to all raibid-labs repositories.

Options:
    -h, --help          Show this help message
    --dry-run           Preview changes without making them
    --repos LIST        Comma-separated list of repos to sync
    --skip LIST         Comma-separated list of repos to skip
    --auto-merge        Auto-merge PRs that pass validation
    --force             Force update even if validation fails
    -v, --verbose       Enable verbose output

Examples:
    $(basename "$0") --dry-run                    # Preview changes
    $(basename "$0") --repos repo1,repo2          # Sync specific repos
    $(basename "$0") --skip "archived-*,test-*"   # Skip repos matching patterns
    $(basename "$0") --auto-merge                 # Auto-merge passing PRs

Prerequisites:
    - gh CLI installed and authenticated
    - git installed
    - jq installed
    - Write access to raibid-labs repositories

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
# Repository Discovery
#############################################################################

get_org_repositories() {
    local skip_patterns="$1"

    log_info "Fetching repositories from $ORG_NAME..."

    # Get all repositories using gh CLI
    local repos
    repos=$(gh repo list "$ORG_NAME" --limit 1000 --json name,isArchived,isPrivate --jq '.[] | select(.isArchived == false) | .name')

    # Filter out org config repo itself
    repos=$(echo "$repos" | grep -v "^${ORG_CONFIG_REPO}$")

    # Apply skip patterns
    if [[ -n "$skip_patterns" ]]; then
        local IFS=','
        for pattern in $skip_patterns; do
            repos=$(echo "$repos" | grep -v "$pattern" || true)
        done
    fi

    echo "$repos"
}

filter_repositories() {
    local all_repos="$1"
    local include_list="$2"

    if [[ -z "$include_list" ]]; then
        echo "$all_repos"
        return
    fi

    local filtered=""
    local IFS=','
    for repo in $include_list; do
        if echo "$all_repos" | grep -q "^${repo}$"; then
            filtered="${filtered}${repo}\n"
        else
            log_warn "Repository not found: $repo"
        fi
    done

    echo -e "$filtered" | grep -v '^$'
}

#############################################################################
# Repository Sync Functions
#############################################################################

clone_or_update_repo() {
    local repo_name="$1"
    local repo_path="$WORK_DIR/$repo_name"

    if [[ -d "$repo_path" ]]; then
        log_info "Updating existing clone: $repo_name"
        git -C "$repo_path" fetch origin
        git -C "$repo_path" checkout main 2>/dev/null || git -C "$repo_path" checkout master
        git -C "$repo_path" pull
    else
        log_info "Cloning repository: $repo_name"
        gh repo clone "$ORG_NAME/$repo_name" "$repo_path"
    fi
}

check_if_repo_uses_claude() {
    local repo_path="$1"

    # Check if .claude directory exists or CLAUDE.md exists
    if [[ -d "$repo_path/.claude" ]] || [[ -f "$repo_path/CLAUDE.md" ]]; then
        return 0
    fi

    return 1
}

update_repo_config() {
    local repo_path="$1"
    local repo_name="$2"
    local changes_made=false

    log_info "Updating configuration for: $repo_name"

    # Check if already extends org config
    if [[ -f "$repo_path/.claude/project.json" ]]; then
        local extends_field
        extends_field=$(jq -r '.extends // empty' "$repo_path/.claude/project.json" 2>/dev/null)

        if [[ -n "$extends_field" ]] && [[ "$extends_field" == *"claude-org-config"* ]]; then
            log_info "Already extends org config, checking for updates..."

            # Update MCP server configs if needed
            if ! jq -e '.mcpServers."claude-flow"' "$repo_path/.claude/project.json" >/dev/null 2>&1; then
                log_info "Adding claude-flow MCP server..."

                local temp_file
                temp_file=$(mktemp)
                jq '.mcpServers."claude-flow" = {
                    "command": "npx",
                    "args": ["claude-flow@alpha", "mcp", "start"],
                    "env": {
                        "CLAUDE_FLOW_SESSION": "'"$repo_name"'-session"
                    }
                }' "$repo_path/.claude/project.json" > "$temp_file"
                mv "$temp_file" "$repo_path/.claude/project.json"
                changes_made=true
            fi
        else
            log_info "Updating to extend org config..."

            # Backup existing config
            cp "$repo_path/.claude/project.json" "$repo_path/.claude/project.json.backup"

            # Update extends field
            local temp_file
            temp_file=$(mktemp)
            jq '.extends = "claude-org-config/.claude/project.json"' "$repo_path/.claude/project.json" > "$temp_file"
            mv "$temp_file" "$repo_path/.claude/project.json"
            changes_made=true
        fi
    else
        log_info "No .claude/project.json found, initializing..."
        # Run init-repo.sh if available
        if [[ -x "$(dirname "$0")/init-repo.sh" ]]; then
            "$(dirname "$0")/init-repo.sh" "$repo_path"
            changes_made=true
        else
            log_warn "init-repo.sh not found, skipping initialization"
        fi
    fi

    # Update CLAUDE.md if it exists
    if [[ -f "$repo_path/CLAUDE.md" ]]; then
        if ! grep -q "raibid-labs organization configuration" "$repo_path/CLAUDE.md"; then
            log_info "Updating CLAUDE.md header..."
            {
                echo "# Claude Code Configuration"
                echo ""
                echo "This repository extends the raibid-labs organization configuration."
                echo ""
                cat "$repo_path/CLAUDE.md"
            } > "$repo_path/CLAUDE.md.new"
            mv "$repo_path/CLAUDE.md.new" "$repo_path/CLAUDE.md"
            changes_made=true
        fi
    fi

    if [[ "$changes_made" == true ]]; then
        return 0
    else
        return 1
    fi
}

create_pr() {
    local repo_path="$1"
    local repo_name="$2"

    log_info "Creating PR for: $repo_name"

    # Create branch
    git -C "$repo_path" checkout -b "$BRANCH_NAME"

    # Stage changes
    git -C "$repo_path" add .claude/ CLAUDE.md 2>/dev/null || true

    # Check if there are changes
    if ! git -C "$repo_path" diff --cached --quiet; then
        # Commit changes
        git -C "$repo_path" commit -m "chore: sync with org config

- Update .claude/project.json to extend org config
- Update MCP server configurations
- Sync CLAUDE.md with latest standards

This is an automated sync from raibid-labs/claude-org-config."

        # Push branch
        git -C "$repo_path" push origin "$BRANCH_NAME"

        # Create PR using gh CLI
        local pr_url
        pr_url=$(gh pr create \
            --repo "$ORG_NAME/$repo_name" \
            --title "Sync with org config updates" \
            --body "This PR syncs the repository with the latest raibid-labs organization configuration.

## Changes
- Updated \`.claude/project.json\` to extend org config
- Updated MCP server configurations
- Synced \`CLAUDE.md\` with latest standards

## Validation
Run \`./scripts/validate-repo.sh\` to verify compliance.

---
*This PR was automatically generated by the org config sync script.*" \
            --base main 2>/dev/null || gh pr create \
            --repo "$ORG_NAME/$repo_name" \
            --title "Sync with org config updates" \
            --body "This PR syncs the repository with the latest raibid-labs organization configuration.

## Changes
- Updated \`.claude/project.json\` to extend org config
- Updated MCP server configurations
- Synced \`CLAUDE.md\` with latest standards

## Validation
Run \`./scripts/validate-repo.sh\` to verify compliance.

---
*This PR was automatically generated by the org config sync script.*" \
            --base master)

        log_success "Created PR: $pr_url"
        echo "$pr_url"
    else
        log_info "No changes to commit"
        git -C "$repo_path" checkout main 2>/dev/null || git -C "$repo_path" checkout master
        git -C "$repo_path" branch -D "$BRANCH_NAME"
        return 1
    fi
}

#############################################################################
# Main Script
#############################################################################

main() {
    local dry_run=false
    local auto_merge=false
    local force=false
    local repos_filter=""
    local skip_patterns=""

    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                print_usage
                exit 0
                ;;
            --dry-run)
                dry_run=true
                shift
                ;;
            --repos)
                repos_filter="$2"
                shift 2
                ;;
            --skip)
                skip_patterns="$2"
                shift 2
                ;;
            --auto-merge)
                auto_merge=true
                shift
                ;;
            --force)
                force=true
                shift
                ;;
            -v|--verbose)
                set -x
                shift
                ;;
            -*)
                log_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
            *)
                shift
                ;;
        esac
    done

    # Check prerequisites
    if ! command -v gh &> /dev/null; then
        log_error "gh CLI not found. Install from: https://cli.github.com/"
        exit 1
    fi

    if ! command -v jq &> /dev/null; then
        log_error "jq not found. Install with: brew install jq"
        exit 1
    fi

    # Verify gh authentication
    if ! gh auth status &> /dev/null; then
        log_error "Not authenticated with GitHub. Run: gh auth login"
        exit 1
    fi

    log_info "Starting org config sync..."
    echo ""

    # Create work directory
    mkdir -p "$WORK_DIR"

    # Get repositories
    local all_repos
    all_repos=$(get_org_repositories "$skip_patterns")

    if [[ -n "$repos_filter" ]]; then
        all_repos=$(filter_repositories "$all_repos" "$repos_filter")
    fi

    local repo_count
    repo_count=$(echo "$all_repos" | wc -l | tr -d ' ')
    log_info "Found $repo_count repositories to process"
    echo ""

    if [[ "$dry_run" == true ]]; then
        log_warn "DRY RUN MODE - No changes will be made"
        echo "$all_repos"
        exit 0
    fi

    # Process each repository
    while IFS= read -r repo; do
        [[ -z "$repo" ]] && continue

        ((REPOS_PROCESSED++))

        echo ""
        echo "============================================"
        log_info "Processing: $repo ($REPOS_PROCESSED/$repo_count)"
        echo "============================================"

        # Clone or update
        if ! clone_or_update_repo "$repo"; then
            log_error "Failed to clone/update: $repo"
            ((REPOS_FAILED++))
            continue
        fi

        local repo_path="$WORK_DIR/$repo"

        # Check if repo uses Claude Code
        if ! check_if_repo_uses_claude "$repo_path"; then
            log_info "Repository does not use Claude Code, skipping"
            ((REPOS_SKIPPED++))
            continue
        fi

        # Update configuration
        if update_repo_config "$repo_path" "$repo"; then
            # Validate changes unless forced
            if [[ "$force" == false ]]; then
                if [[ -x "$(dirname "$0")/validate-repo.sh" ]]; then
                    if ! "$(dirname "$0")/validate-repo.sh" "$repo_path"; then
                        log_warn "Validation failed, but continuing..."
                    fi
                fi
            fi

            # Create PR
            if pr_url=$(create_pr "$repo_path" "$repo"); then
                ((REPOS_UPDATED++))

                # Auto-merge if requested
                if [[ "$auto_merge" == true ]]; then
                    log_info "Auto-merging PR..."
                    if gh pr merge "$pr_url" --auto --squash; then
                        log_success "PR auto-merge enabled"
                    else
                        log_warn "Failed to enable auto-merge"
                    fi
                fi
            else
                log_info "No changes needed"
                ((REPOS_SKIPPED++))
            fi
        else
            log_info "No updates required"
            ((REPOS_SKIPPED++))
        fi

    done <<< "$all_repos"

    # Print summary
    echo ""
    echo "============================================"
    echo "  Sync Summary"
    echo "============================================"
    echo "Processed: $REPOS_PROCESSED"
    echo "Updated:   $REPOS_UPDATED"
    echo "Skipped:   $REPOS_SKIPPED"
    echo "Failed:    $REPOS_FAILED"
    echo ""

    if [[ $REPOS_UPDATED -gt 0 ]]; then
        log_success "Sync completed! $REPOS_UPDATED PRs created"
    else
        log_info "Sync completed, no updates needed"
    fi
}

# Run main function
main "$@"
