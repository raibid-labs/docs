#!/usr/bin/env bash

#############################################################################
# analyze-org.sh - Analyze raibid-labs organization repositories
#############################################################################
#
# This script analyzes all repositories in the raibid-labs organization
# and generates a comprehensive report similar to the initial analysis.
#
# Usage:
#   ./analyze-org.sh [OPTIONS] > analysis.md
#
# Options:
#   --format FORMAT     Output format: markdown, json, text (default: markdown)
#   --include-archived  Include archived repositories
#   --detailed          Include detailed analysis per repo
#
# Examples:
#   ./analyze-org.sh > docs/org-analysis.md
#   ./analyze-org.sh --format json > analysis.json
#   ./analyze-org.sh --detailed --include-archived > detailed-analysis.md
#
#############################################################################

set -euo pipefail

# Configuration
ORG_NAME="raibid-labs"
OUTPUT_FORMAT="markdown"
INCLUDE_ARCHIVED=false
DETAILED_ANALYSIS=false

# Counters
TOTAL_REPOS=0
COMPLIANT_REPOS=0
NON_COMPLIANT_REPOS=0
ARCHIVED_REPOS=0
PRIVATE_REPOS=0
PUBLIC_REPOS=0

#############################################################################
# Helper Functions
#############################################################################

print_usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Analyze raibid-labs organization repositories and generate report.

Options:
    -h, --help              Show this help message
    -f, --format FORMAT     Output format: markdown, json, text (default: markdown)
    -a, --include-archived  Include archived repositories
    -d, --detailed          Include detailed analysis per repo
    -o, --output FILE       Write output to file

Examples:
    $(basename "$0") > docs/org-analysis.md
    $(basename "$0") --format json > analysis.json
    $(basename "$0") --detailed --output detailed-analysis.md

Prerequisites:
    - gh CLI installed and authenticated
    - jq installed for JSON processing

EOF
}

#############################################################################
# Data Collection Functions
#############################################################################

get_all_repositories() {
    local include_archived="$1"

    if [[ "$include_archived" == true ]]; then
        gh repo list "$ORG_NAME" --limit 1000 --json name,isArchived,isPrivate,description,updatedAt,primaryLanguage,languages,stargazerCount,forkCount
    else
        gh repo list "$ORG_NAME" --limit 1000 --json name,isArchived,isPrivate,description,updatedAt,primaryLanguage,languages,stargazerCount,forkCount | \
            jq '[.[] | select(.isArchived == false)]'
    fi
}

analyze_repository() {
    local repo_name="$1"

    # Clone repository to temp location for analysis
    local temp_dir
    temp_dir=$(mktemp -d)

    if gh repo clone "$ORG_NAME/$repo_name" "$temp_dir/$repo_name" 2>/dev/null; then
        local repo_path="$temp_dir/$repo_name"
        local has_claude=false
        local extends_org=false
        local has_tests=false
        local has_docs=false

        # Check for Claude configuration
        if [[ -f "$repo_path/.claude/project.json" ]]; then
            has_claude=true

            # Check if extends org config
            if jq -e '.extends' "$repo_path/.claude/project.json" | grep -q "claude-org-config"; then
                extends_org=true
            fi
        fi

        # Check for tests
        if [[ -d "$repo_path/tests" ]] || [[ -d "$repo_path/test" ]] || [[ -d "$repo_path/__tests__" ]]; then
            has_tests=true
        fi

        # Check for documentation
        if [[ -d "$repo_path/docs" ]] || [[ -f "$repo_path/README.md" ]]; then
            has_docs=true
        fi

        # Cleanup
        rm -rf "$temp_dir"

        # Return JSON
        jq -n \
            --arg name "$repo_name" \
            --argjson has_claude "$has_claude" \
            --argjson extends_org "$extends_org" \
            --argjson has_tests "$has_tests" \
            --argjson has_docs "$has_docs" \
            '{
                name: $name,
                has_claude_config: $has_claude,
                extends_org_config: $extends_org,
                has_tests: $has_tests,
                has_docs: $has_docs
            }'
    else
        echo "{\"name\": \"$repo_name\", \"error\": \"Failed to clone\"}"
    fi
}

#############################################################################
# Report Generation - Markdown
#############################################################################

generate_markdown_report() {
    local repos_data="$1"

    cat << EOF
# Raibid Labs Organization Analysis

**Generated:** $(date)
**Organization:** $ORG_NAME

## Executive Summary

| Metric | Count |
|--------|-------|
| Total Repositories | $TOTAL_REPOS |
| Active Repositories | $((TOTAL_REPOS - ARCHIVED_REPOS)) |
| Archived Repositories | $ARCHIVED_REPOS |
| Public Repositories | $PUBLIC_REPOS |
| Private Repositories | $PRIVATE_REPOS |
| Compliant with Org Config | $COMPLIANT_REPOS |
| Non-Compliant | $NON_COMPLIANT_REPOS |

## Compliance Overview

\`\`\`
Compliance Rate: $(awk "BEGIN {printf \"%.1f\", ($COMPLIANT_REPOS / $TOTAL_REPOS) * 100}")%
\`\`\`

### Compliance Status

- ✅ **Compliant**: Repositories that extend the org config
- ⚠️ **Partial**: Have Claude config but don't extend org config
- ❌ **Non-Compliant**: Missing Claude configuration

## Repository Breakdown

### By Language

EOF

    # Language distribution
    echo "$repos_data" | jq -r '
        group_by(.primaryLanguage.name // "Unknown") |
        map({language: .[0].primaryLanguage.name // "Unknown", count: length}) |
        sort_by(.count) | reverse |
        .[] |
        "| \(.language) | \(.count) |"
    ' | {
        echo "| Language | Count |"
        echo "|----------|-------|"
        cat
    }

    echo ""
    echo "### By Compliance Status"
    echo ""
    echo "| Repository | Status | Last Updated |"
    echo "|------------|--------|--------------|"

    # List repositories by compliance
    echo "$repos_data" | jq -r '
        sort_by(.name) |
        .[] |
        "\(.name)|\(.updatedAt)"
    ' | while IFS='|' read -r name updated; do
        # Here you would check actual compliance status
        echo "| $name | ⚠️ Needs Check | $updated |"
    done

    if [[ "$DETAILED_ANALYSIS" == true ]]; then
        echo ""
        echo "## Detailed Repository Analysis"
        echo ""

        echo "$repos_data" | jq -r '.[] | .name' | while read -r repo; do
            echo "### $repo"
            echo ""

            # Get repository details
            local repo_info
            repo_info=$(echo "$repos_data" | jq -r ".[] | select(.name == \"$repo\")")

            echo "**Description:** $(echo "$repo_info" | jq -r '.description // "N/A"')"
            echo ""
            echo "**Primary Language:** $(echo "$repo_info" | jq -r '.primaryLanguage.name // "N/A"')"
            echo ""
            echo "**Stars:** $(echo "$repo_info" | jq -r '.stargazerCount')"
            echo ""
            echo "**Forks:** $(echo "$repo_info" | jq -r '.forkCount')"
            echo ""
            echo "**Last Updated:** $(echo "$repo_info" | jq -r '.updatedAt')"
            echo ""
            echo "---"
            echo ""
        done
    fi

    cat << EOF

## Recommendations

### Immediate Actions

1. **Initialize Non-Compliant Repositories**
   - Run \`./scripts/init-repo.sh\` on repositories without Claude config
   - Update existing configs to extend org config

2. **Standardize MCP Configurations**
   - Ensure all repositories have claude-flow MCP server configured
   - Add optional MCP servers as needed per project type

3. **Update Documentation**
   - Add CLAUDE.md to all repositories
   - Ensure README.md references org standards

### Long-term Improvements

1. **Automate Compliance Checks**
   - Set up GitHub Actions for validation
   - Run weekly compliance reports

2. **Repository Templates**
   - Create repository templates for each project type
   - Pre-configure with org standards

3. **Training and Documentation**
   - Create onboarding guide for new developers
   - Document best practices and examples

## Tools Available

- \`init-repo.sh\` - Initialize repository with org config
- \`validate-repo.sh\` - Check compliance with standards
- \`sync-to-repos.sh\` - Sync updates across all repos
- \`analyze-org.sh\` - Generate this analysis report

## Next Steps

1. Review compliance status for each repository
2. Run sync script to update non-compliant repositories
3. Set up automated validation in CI/CD
4. Schedule regular compliance audits

---

Generated by raibid-labs/claude-org-config analysis tools
EOF
}

#############################################################################
# Report Generation - JSON
#############################################################################

generate_json_report() {
    local repos_data="$1"

    jq -n \
        --arg timestamp "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
        --arg org "$ORG_NAME" \
        --argjson total "$TOTAL_REPOS" \
        --argjson archived "$ARCHIVED_REPOS" \
        --argjson public "$PUBLIC_REPOS" \
        --argjson private "$PRIVATE_REPOS" \
        --argjson compliant "$COMPLIANT_REPOS" \
        --argjson non_compliant "$NON_COMPLIANT_REPOS" \
        --argjson repos "$repos_data" \
        '{
            timestamp: $timestamp,
            organization: $org,
            summary: {
                total_repositories: $total,
                archived_repositories: $archived,
                public_repositories: $public,
                private_repositories: $private,
                compliant_repositories: $compliant,
                non_compliant_repositories: $non_compliant,
                compliance_rate: (($compliant / $total) * 100)
            },
            repositories: $repos
        }'
}

#############################################################################
# Main Script
#############################################################################

main() {
    local output_file=""

    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                print_usage
                exit 0
                ;;
            -f|--format)
                OUTPUT_FORMAT="$2"
                shift 2
                ;;
            -a|--include-archived)
                INCLUDE_ARCHIVED=true
                shift
                ;;
            -d|--detailed)
                DETAILED_ANALYSIS=true
                shift
                ;;
            -o|--output)
                output_file="$2"
                shift 2
                ;;
            -*)
                echo "Unknown option: $1" >&2
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
        echo "Error: gh CLI not found. Install from: https://cli.github.com/" >&2
        exit 1
    fi

    if ! command -v jq &> /dev/null; then
        echo "Error: jq not found. Install with: brew install jq" >&2
        exit 1
    fi

    # Verify gh authentication
    if ! gh auth status &> /dev/null; then
        echo "Error: Not authenticated with GitHub. Run: gh auth login" >&2
        exit 1
    fi

    # Fetch repository data
    echo "Fetching repository data..." >&2
    local repos_data
    repos_data=$(get_all_repositories "$INCLUDE_ARCHIVED")

    # Calculate statistics
    TOTAL_REPOS=$(echo "$repos_data" | jq 'length')
    ARCHIVED_REPOS=$(echo "$repos_data" | jq '[.[] | select(.isArchived == true)] | length')
    PUBLIC_REPOS=$(echo "$repos_data" | jq '[.[] | select(.isPrivate == false)] | length')
    PRIVATE_REPOS=$(echo "$repos_data" | jq '[.[] | select(.isPrivate == true)] | length')

    echo "Analyzing $TOTAL_REPOS repositories..." >&2

    # Generate report
    local report
    case "$OUTPUT_FORMAT" in
        json)
            report=$(generate_json_report "$repos_data")
            ;;
        markdown|md)
            report=$(generate_markdown_report "$repos_data")
            ;;
        text|*)
            report=$(generate_markdown_report "$repos_data")
            ;;
    esac

    # Output report
    if [[ -n "$output_file" ]]; then
        echo "$report" > "$output_file"
        echo "Report saved to: $output_file" >&2
    else
        echo "$report"
    fi
}

# Run main function
main "$@"
