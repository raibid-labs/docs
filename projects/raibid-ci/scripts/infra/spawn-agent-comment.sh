#!/bin/bash
# Post spawn trigger comment for orchestrator to detect
# This comment signals that an issue is ready for agent assignment

set -e

ISSUE_NUM="${ISSUE_NUMBER}"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

echo "Posting spawn trigger comment for issue #${ISSUE_NUM}..."

# Fetch issue details to determine agent type
ISSUE_JSON=$(gh issue view "$ISSUE_NUM" --json title,labels)
ISSUE_TITLE=$(echo "$ISSUE_JSON" | jq -r '.title')
LABELS=$(echo "$ISSUE_JSON" | jq -r '.labels[].name')

# Determine agent type based on issue title or labels
AGENT_TYPE="rust-pro"  # Default for raibid-ci

if echo "$ISSUE_TITLE" | grep -qi "API"; then
  AGENT_TYPE="rust-pro"
elif echo "$ISSUE_TITLE" | grep -qi "TUI"; then
  AGENT_TYPE="rust-pro"
elif echo "$ISSUE_TITLE" | grep -qi "CLI"; then
  AGENT_TYPE="rust-pro"
elif echo "$LABELS" | grep -qi "backend"; then
  AGENT_TYPE="rust-pro"
elif echo "$LABELS" | grep -qi "frontend"; then
  AGENT_TYPE="rust-pro"
fi

# Extract issue ID from title (e.g., "CLI-001" from "CLI-001: Project Scaffolding")
ISSUE_ID=$(echo "$ISSUE_TITLE" | grep -oP '^[A-Z]+-[0-9]+' || echo "ISSUE-$ISSUE_NUM")

# Post spawn trigger comment
gh issue comment "$ISSUE_NUM" --body \
"🤖 **ORCHESTRATOR-SPAWN-AGENT**

**Issue**: #${ISSUE_NUM}
**Issue ID**: ${ISSUE_ID}
**Type**: ${AGENT_TYPE}
**Status**: ready
**Timestamp**: ${TIMESTAMP}

**Agent Instructions:**
1. Review this issue and all comments thoroughly
2. Follow TDD workflow (tests first, then implementation)
3. Create feature branch: \`${ISSUE_ID,,}-implementation\`
4. Commit frequently with clear messages
5. Submit PR when complete, referencing this issue

---
<!-- ORCHESTRATOR-STATE
{
  \"issue\": ${ISSUE_NUM},
  \"issue_id\": \"${ISSUE_ID}\",
  \"agent_type\": \"${AGENT_TYPE}\",
  \"status\": \"ready\",
  \"spawned_at\": \"${TIMESTAMP}\"
}
-->

*This comment triggers agent spawning. Orchestrator will detect and spawn the development agent.*

---
*Orchestrator Event-Driven System v1.0*"

echo "✅ Spawn trigger comment posted successfully"
echo "Issue #${ISSUE_NUM} is now ready for orchestrator to spawn agent"
