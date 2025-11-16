#!/bin/bash
# Check if issue is in draft state for enrichment workflow
# Draft issues get enriched by an agent before implementation begins

set -e

ISSUE_NUM="${ISSUE_NUMBER}"

echo "Checking draft status for issue #${ISSUE_NUM}..."

# Fetch issue details
ISSUE_JSON=$(gh issue view "$ISSUE_NUM" --json labels)

# Check if issue has "draft" label
IS_DRAFT=$(echo "$ISSUE_JSON" | jq -r '.labels[] | select(.name == "draft" or .name == "status:draft") | .name' | head -1)

if [ -n "$IS_DRAFT" ]; then
  echo "📝 Issue is in DRAFT state"
  echo "is_draft=true" >> $GITHUB_OUTPUT
  echo "draft_label=$IS_DRAFT" >> $GITHUB_OUTPUT
else
  echo "✅ Issue is NOT in draft state"
  echo "is_draft=false" >> $GITHUB_OUTPUT
fi
