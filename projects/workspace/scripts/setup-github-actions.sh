#!/usr/bin/env bash

#############################################################################
# setup-github-actions.sh - Set up GitHub Actions for org config management
#############################################################################
#
# This script creates GitHub Actions workflows for:
# - Validating repository compliance
# - Auto-syncing org config updates
# - Running compliance audits
#
# Usage:
#   ./setup-github-actions.sh [OPTIONS] [repo-path]
#
# Options:
#   --validation-only   Only create validation workflow
#   --sync-only        Only create sync workflow
#   --all              Create all workflows (default)
#
# Examples:
#   ./setup-github-actions.sh                    # Setup in current repo
#   ./setup-github-actions.sh /path/to/repo      # Setup in specific repo
#   ./setup-github-actions.sh --validation-only  # Only validation workflow
#
#############################################################################

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

#############################################################################
# Helper Functions
#############################################################################

print_usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS] [repo-path]

Set up GitHub Actions workflows for org config management.

Arguments:
    repo-path           Path to repository (default: current directory)

Options:
    -h, --help          Show this help message
    --validation-only   Only create validation workflow
    --sync-only         Only create sync workflow
    --audit-only        Only create audit workflow
    --all               Create all workflows (default)
    -f, --force         Overwrite existing workflows

Examples:
    $(basename "$0")                        # Setup in current repo
    $(basename "$0") /path/to/repo          # Setup in specific repo
    $(basename "$0") --validation-only      # Only validation workflow

Created Workflows:
    - validate-compliance.yml - PR validation
    - sync-org-config.yml    - Auto-sync on org config changes
    - audit-compliance.yml   - Weekly compliance audits

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
# Workflow Templates
#############################################################################

create_validation_workflow() {
    cat << 'EOF'
name: Validate Org Config Compliance

on:
  pull_request:
    branches: [main, master]
    paths:
      - '.claude/**'
      - 'CLAUDE.md'
  push:
    branches: [main, master]
    paths:
      - '.claude/**'
      - 'CLAUDE.md'
  workflow_dispatch:

jobs:
  validate:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install jq
        run: sudo apt-get update && sudo apt-get install -y jq

      - name: Download validation script
        run: |
          mkdir -p scripts
          curl -o scripts/validate-repo.sh https://raw.githubusercontent.com/raibid-labs/claude-org-config/main/scripts/validate-repo.sh
          chmod +x scripts/validate-repo.sh

      - name: Run validation
        id: validate
        run: |
          ./scripts/validate-repo.sh --format markdown --report validation-report.md
        continue-on-error: true

      - name: Upload validation report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: validation-report
          path: validation-report.md

      - name: Comment PR with results
        if: github.event_name == 'pull_request' && always()
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('validation-report.md', 'utf8');

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## Org Config Compliance Report\n\n${report}`
            });

      - name: Check validation status
        run: |
          if [ "${{ steps.validate.outcome }}" != "success" ]; then
            echo "Validation failed. Please review the report."
            exit 1
          fi
EOF
}

create_sync_workflow() {
    cat << 'EOF'
name: Sync Org Config Updates

on:
  repository_dispatch:
    types: [org-config-updated]
  workflow_dispatch:
    inputs:
      force:
        description: 'Force update even if validation fails'
        required: false
        default: 'false'

jobs:
  sync:
    runs-on: ubuntu-latest

    permissions:
      contents: write
      pull-requests: write

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y jq
          npm install -g @ruvnet/claude-flow@alpha

      - name: Download org config scripts
        run: |
          mkdir -p scripts
          curl -o scripts/init-repo.sh https://raw.githubusercontent.com/raibid-labs/claude-org-config/main/scripts/init-repo.sh
          curl -o scripts/validate-repo.sh https://raw.githubusercontent.com/raibid-labs/claude-org-config/main/scripts/validate-repo.sh
          chmod +x scripts/*.sh

      - name: Update configuration
        id: update
        run: |
          if [ -f .claude/project.json ]; then
            # Backup current config
            cp .claude/project.json .claude/project.json.backup

            # Update extends field if needed
            jq '.extends = "claude-org-config/.claude/project.json"' .claude/project.json > .claude/project.json.tmp
            mv .claude/project.json.tmp .claude/project.json

            # Update MCP servers
            if ! jq -e '.mcpServers."claude-flow"' .claude/project.json > /dev/null; then
              jq '.mcpServers."claude-flow" = {
                "command": "npx",
                "args": ["claude-flow@alpha", "mcp", "start"],
                "env": {
                  "CLAUDE_FLOW_SESSION": "${{ github.event.repository.name }}-session"
                }
              }' .claude/project.json > .claude/project.json.tmp
              mv .claude/project.json.tmp .claude/project.json
            fi

            echo "updated=true" >> $GITHUB_OUTPUT
          else
            echo "No .claude/project.json found, initializing..."
            ./scripts/init-repo.sh .
            echo "updated=true" >> $GITHUB_OUTPUT
          fi

      - name: Validate changes
        if: steps.update.outputs.updated == 'true' && github.event.inputs.force != 'true'
        run: |
          ./scripts/validate-repo.sh --strict

      - name: Create Pull Request
        if: steps.update.outputs.updated == 'true'
        uses: peter-evans/create-pull-request@v6
        with:
          commit-message: |
            chore: sync with org config

            - Update .claude/project.json to extend org config
            - Update MCP server configurations
            - Sync CLAUDE.md with latest standards

            This is an automated sync from raibid-labs/claude-org-config.
          branch: sync/org-config-${{ github.run_number }}
          title: 'Sync with org config updates'
          body: |
            ## Org Config Sync

            This PR syncs the repository with the latest raibid-labs organization configuration.

            ### Changes
            - Updated `.claude/project.json` to extend org config
            - Updated MCP server configurations
            - Synced `CLAUDE.md` with latest standards

            ### Validation
            Run `./scripts/validate-repo.sh` to verify compliance.

            ---
            *This PR was automatically generated by the org config sync workflow.*
          labels: |
            org-config
            automated
          draft: false
EOF
}

create_audit_workflow() {
    cat << 'EOF'
name: Compliance Audit

on:
  schedule:
    - cron: '0 0 * * 0' # Weekly on Sunday at midnight
  workflow_dispatch:

jobs:
  audit:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install jq
        run: sudo apt-get update && sudo apt-get install -y jq

      - name: Download validation script
        run: |
          mkdir -p scripts
          curl -o scripts/validate-repo.sh https://raw.githubusercontent.com/raibid-labs/claude-org-config/main/scripts/validate-repo.sh
          chmod +x scripts/validate-repo.sh

      - name: Run compliance audit
        id: audit
        run: |
          ./scripts/validate-repo.sh --format markdown --report audit-report.md
        continue-on-error: true

      - name: Upload audit report
        uses: actions/upload-artifact@v4
        with:
          name: compliance-audit-${{ github.run_number }}
          path: audit-report.md

      - name: Create issue if non-compliant
        if: steps.audit.outcome != 'success'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('audit-report.md', 'utf8');

            github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: 'Compliance Audit Failed',
              body: `## Compliance Audit Report\n\n${report}\n\n---\nGenerated by weekly compliance audit workflow.`,
              labels: ['compliance', 'audit']
            });
EOF
}

create_org_sync_dispatch_workflow() {
    cat << 'EOF'
name: Dispatch Org Config Updates

on:
  push:
    branches: [main]
    paths:
      - '.claude/**'
  workflow_dispatch:

jobs:
  dispatch:
    runs-on: ubuntu-latest

    steps:
      - name: Trigger sync across org
        uses: actions/github-script@v7
        with:
          github-token: ${{ secrets.ORG_ADMIN_TOKEN }}
          script: |
            const repos = await github.paginate(
              github.rest.repos.listForOrg,
              {
                org: 'raibid-labs',
                type: 'all',
                per_page: 100
              }
            );

            for (const repo of repos) {
              if (repo.archived || repo.name === 'claude-org-config') {
                continue;
              }

              try {
                await github.rest.repos.createDispatchEvent({
                  owner: 'raibid-labs',
                  repo: repo.name,
                  event_type: 'org-config-updated'
                });
                console.log(`Dispatched to ${repo.name}`);
              } catch (error) {
                console.error(`Failed to dispatch to ${repo.name}:`, error.message);
              }
            }
EOF
}

#############################################################################
# Main Script
#############################################################################

main() {
    local repo_path="${1:-.}"
    local create_validation=false
    local create_sync=false
    local create_audit=false
    local force=false

    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                print_usage
                exit 0
                ;;
            --validation-only)
                create_validation=true
                shift
                ;;
            --sync-only)
                create_sync=true
                shift
                ;;
            --audit-only)
                create_audit=true
                shift
                ;;
            --all)
                create_validation=true
                create_sync=true
                create_audit=true
                shift
                ;;
            -f|--force)
                force=true
                shift
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

    # Default to all if none specified
    if [[ "$create_validation" == false ]] && [[ "$create_sync" == false ]] && [[ "$create_audit" == false ]]; then
        create_validation=true
        create_sync=true
        create_audit=true
    fi

    # Resolve absolute path
    repo_path=$(cd "$repo_path" 2>/dev/null && pwd || echo "$repo_path")

    if [[ ! -d "$repo_path" ]]; then
        log_error "Repository path does not exist: $repo_path"
        exit 1
    fi

    log_info "Setting up GitHub Actions in: $repo_path"

    # Create .github/workflows directory
    local workflows_dir="$repo_path/.github/workflows"
    mkdir -p "$workflows_dir"

    # Create workflows
    if [[ "$create_validation" == true ]]; then
        local validation_file="$workflows_dir/validate-compliance.yml"
        if [[ -f "$validation_file" ]] && [[ "$force" != true ]]; then
            log_warn "Validation workflow already exists, skipping"
        else
            log_info "Creating validation workflow..."
            create_validation_workflow > "$validation_file"
            log_success "Created: validate-compliance.yml"
        fi
    fi

    if [[ "$create_sync" == true ]]; then
        local sync_file="$workflows_dir/sync-org-config.yml"
        if [[ -f "$sync_file" ]] && [[ "$force" != true ]]; then
            log_warn "Sync workflow already exists, skipping"
        else
            log_info "Creating sync workflow..."
            create_sync_workflow > "$sync_file"
            log_success "Created: sync-org-config.yml"
        fi
    fi

    if [[ "$create_audit" == true ]]; then
        local audit_file="$workflows_dir/audit-compliance.yml"
        if [[ -f "$audit_file" ]] && [[ "$force" != true ]]; then
            log_warn "Audit workflow already exists, skipping"
        else
            log_info "Creating audit workflow..."
            create_audit_workflow > "$audit_file"
            log_success "Created: audit-compliance.yml"
        fi
    fi

    # Create dispatch workflow for org config repo
    if [[ "$(basename "$repo_path")" == "claude-org-config" ]]; then
        local dispatch_file="$workflows_dir/dispatch-updates.yml"
        if [[ -f "$dispatch_file" ]] && [[ "$force" != true ]]; then
            log_warn "Dispatch workflow already exists, skipping"
        else
            log_info "Creating dispatch workflow for org config repo..."
            create_org_sync_dispatch_workflow > "$dispatch_file"
            log_success "Created: dispatch-updates.yml"
        fi
    fi

    log_success "GitHub Actions setup completed!"
    echo ""
    log_info "Next steps:"
    echo "  1. Review the created workflow files in .github/workflows/"
    echo "  2. Commit and push the workflows to enable them"
    echo "  3. For org-wide sync, set up ORG_ADMIN_TOKEN secret"
    echo ""
    log_info "Workflows created:"
    ls -1 "$workflows_dir"/*.yml
}

# Run main function
main "$@"
