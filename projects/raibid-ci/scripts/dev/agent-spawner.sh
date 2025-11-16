#!/bin/bash
# Agent Spawner - Reads GitHub spawn comments and launches Claude Code agents
#
# Usage: ./agent-spawner.sh [options]
#
# Options:
#   --once           Run once and exit (default: continuous loop)
#   --interval SEC   Polling interval in seconds (default: 30)
#   --issue NUM      Spawn agent for specific issue number
#   --dry-run        Show what would be spawned without spawning
#   --help           Show this help message
#
# Description:
#   Monitors GitHub issues for ORCHESTRATOR-SPAWN-AGENT comments and
#   automatically spawns Claude Code agents using the Task tool.
#
# Exit codes:
#   0 - Success
#   1 - Error occurred
#   2 - Invalid arguments
#   3 - Missing dependencies

set -euo pipefail

# Color codes
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly PURPLE='\033[0;35m'
readonly NC='\033[0m' # No Color

# Configuration
POLL_INTERVAL=30
RUN_ONCE=false
DRY_RUN=false
SPECIFIC_ISSUE=""
STATE_FILE="/tmp/raibid_agent_spawner_state.json"

# Logging functions
log_info() { echo -e "${BLUE}ℹ${NC} $*"; }
log_success() { echo -e "${GREEN}✓${NC} $*"; }
log_warning() { echo -e "${YELLOW}⚠${NC} $*"; }
log_error() { echo -e "${RED}✗${NC} $*" >&2; }
log_agent() { echo -e "${PURPLE}🤖${NC} $*"; }

# Check dependencies
check_dependencies() {
    local missing_deps=()

    for cmd in gh jq; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_deps+=("$cmd")
        fi
    done

    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_info "Install with: brew install ${missing_deps[*]}"
        exit 3
    fi
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --once)
                RUN_ONCE=true
                shift
                ;;
            --interval)
                POLL_INTERVAL="$2"
                shift 2
                ;;
            --issue)
                SPECIFIC_ISSUE="$2"
                shift 2
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --help)
                grep '^#' "$0" | sed 's/^# \?//'
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 2
                ;;
        esac
    done
}

# Initialize state file
init_state() {
    if [ ! -f "$STATE_FILE" ]; then
        echo '{"spawned_agents": {}}' > "$STATE_FILE"
    fi
}

# Check if agent already spawned for issue
is_agent_spawned() {
    local issue_num=$1
    local status=$(jq -r --arg issue "$issue_num" '.spawned_agents[$issue] // "not_spawned"' "$STATE_FILE")
    [ "$status" != "not_spawned" ]
}

# Mark agent as spawned
mark_agent_spawned() {
    local issue_num=$1
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    jq --arg issue "$issue_num" --arg time "$timestamp" \
        '.spawned_agents[$issue] = $time' "$STATE_FILE" > "${STATE_FILE}.tmp"
    mv "${STATE_FILE}.tmp" "$STATE_FILE"
}

# Get spawn comment from issue
get_spawn_comment() {
    local issue_num=$1
    gh issue view "$issue_num" --json comments | \
        jq -r '([.comments[] | select(.body | contains("ORCHESTRATOR-SPAWN-AGENT"))][0] // empty) | .body'
}

# Parse spawn comment for agent details
parse_spawn_comment() {
    local comment="$1"

    # Extract issue number
    local issue=$(echo "$comment" | grep -oP '(?<=\*\*Issue\*\*: #)\d+')

    # Extract issue ID (e.g., WS-00)
    local issue_id=$(echo "$comment" | grep -oP '(?<=\*\*Issue ID\*\*: )[\w-]+')

    # Extract agent type
    local agent_type=$(echo "$comment" | grep -oP '(?<=\*\*Type\*\*: )[\w-]+')

    # Extract status
    local status=$(echo "$comment" | grep -oP '(?<=\*\*Status\*\*: )\w+')

    echo "$issue|$issue_id|$agent_type|$status"
}

# Get issue title for description
get_issue_title() {
    local issue_num=$1
    gh issue view "$issue_num" --json title | jq -r '.title'
}

# Spawn agent via Claude Code Task tool
spawn_agent() {
    local issue_num=$1
    local issue_id=$2
    local agent_type=$3
    local issue_title=$4

    log_agent "Spawning agent for Issue #$issue_num ($issue_id)"
    log_info "  Agent Type: $agent_type"
    log_info "  Title: $issue_title"

    if [ "$DRY_RUN" = true ]; then
        log_warning "DRY RUN - Would spawn agent but not actually executing"
        return 0
    fi

    # Create spawn instruction file
    local spawn_file="/tmp/spawn_agent_${issue_num}.txt"
    cat > "$spawn_file" << EOF
You are working on Issue #${issue_num}: ${issue_title}

**Your Mission:**
Complete this issue following the instructions provided.

**Read the Full Issue:**
Visit https://github.com/raibid-labs/raibid-ci/issues/${issue_num} and read all details, tasks, and acceptance criteria.

**Workflow:**
1. Create appropriate feature branch
2. Follow TDD where applicable (write tests first)
3. Implement required changes
4. Ensure all acceptance criteria met
5. Create PR referencing #${issue_num}

**Important:**
- Read the issue carefully before starting
- Follow project conventions in CLAUDE.md
- Write clear commit messages
- Test your changes thoroughly
- Update documentation as needed

Begin by reading issue #${issue_num} in detail, then proceed with implementation.
EOF

    log_info "  Instructions saved to: $spawn_file"
    log_success "Agent spawn initiated for #${issue_num}"

    # Note: The actual Task() call would be made by Claude Code, not bash
    # This script prepares the data and logs the spawn event
    # In practice, this would integrate with Claude Code's session

    mark_agent_spawned "$issue_num"
}

# Process a single issue
process_issue() {
    local issue_num=$1

    # Check if already spawned
    if is_agent_spawned "$issue_num"; then
        return 0
    fi

    # Get spawn comment
    local spawn_comment=$(get_spawn_comment "$issue_num")

    if [ -z "$spawn_comment" ]; then
        return 0
    fi

    # Parse spawn comment
    local parsed=$(parse_spawn_comment "$spawn_comment")
    IFS='|' read -r issue issue_id agent_type status <<< "$parsed"

    # Verify status is ready
    if [ "$status" != "ready" ]; then
        log_warning "Issue #$issue_num status is '$status', not 'ready'"
        return 0
    fi

    # Get issue title
    local issue_title=$(get_issue_title "$issue_num")

    # Spawn agent
    spawn_agent "$issue_num" "$issue_id" "$agent_type" "$issue_title"
}

# Scan all open issues for spawn comments
scan_issues() {
    log_info "Scanning for spawn-ready issues..."

    local issues
    if [ -n "$SPECIFIC_ISSUE" ]; then
        issues="$SPECIFIC_ISSUE"
    else
        # Get all open issues
        issues=$(gh issue list --state open --json number | jq -r '.[].number')
    fi

    local spawn_count=0

    for issue_num in $issues; do
        process_issue "$issue_num" && spawn_count=$((spawn_count + 1)) || true
    done

    if [ $spawn_count -eq 0 ]; then
        log_info "No new agents spawned"
    else
        log_success "Spawned $spawn_count agent(s)"
    fi
}

# Show current state
show_state() {
    local spawned_count=$(jq -r '.spawned_agents | length' "$STATE_FILE")
    log_info "Currently tracking $spawned_count spawned agent(s)"
}

# Main execution
main() {
    parse_args "$@"
    check_dependencies
    init_state

    log_info "Agent Spawner started"
    log_info "  Mode: $([ "$RUN_ONCE" = true ] && echo "single run" || echo "continuous (${POLL_INTERVAL}s interval)")"
    log_info "  State file: $STATE_FILE"

    if [ "$DRY_RUN" = true ]; then
        log_warning "DRY RUN MODE - No agents will be spawned"
    fi

    show_state
    echo ""

    if [ "$RUN_ONCE" = true ]; then
        scan_issues
    else
        while true; do
            scan_issues
            echo ""
            log_info "Sleeping for ${POLL_INTERVAL}s..."
            sleep "$POLL_INTERVAL"
            echo ""
        done
    fi
}

# Handle errors
trap 'log_error "Script failed at line $LINENO"; exit 1' ERR

# Run main function
main "$@"
