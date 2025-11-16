#!/bin/bash
# Find the next ready issue and assign it to an agent
# Priority: Critical > High > Medium > Low
# Filter: Only issues with ready:work label

set -e

echo "Finding next ready issue..."

# Fetch all open issues with ready:work label, sorted by priority
READY_ISSUES=$(gh issue list \
  --state open \
  --label "ready:work" \
  --json number,title,labels,createdAt \
  --jq 'sort_by(.createdAt) | .[]')

if [ -z "$READY_ISSUES" ]; then
  echo "No ready issues found"
  echo "issue_number=" >> $GITHUB_OUTPUT
  exit 0
fi

# Find highest priority issue
# Priority order: critical > high > medium > low
NEXT_ISSUE=""

# Check for critical priority
CRITICAL=$(echo "$READY_ISSUES" | jq -s '[.[] | select(.labels[].name | contains("priority:critical"))] | .[0].number' 2>/dev/null || echo "null")
if [ "$CRITICAL" != "null" ]; then
  NEXT_ISSUE="$CRITICAL"
  echo "Found critical priority issue: #$NEXT_ISSUE"
fi

# Check for high priority if no critical found
if [ -z "$NEXT_ISSUE" ] || [ "$NEXT_ISSUE" == "null" ]; then
  HIGH=$(echo "$READY_ISSUES" | jq -s '[.[] | select(.labels[].name | contains("priority:high"))] | .[0].number' 2>/dev/null || echo "null")
  if [ "$HIGH" != "null" ]; then
    NEXT_ISSUE="$HIGH"
    echo "Found high priority issue: #$NEXT_ISSUE"
  fi
fi

# Check for medium priority if no high found
if [ -z "$NEXT_ISSUE" ] || [ "$NEXT_ISSUE" == "null" ]; then
  MEDIUM=$(echo "$READY_ISSUES" | jq -s '[.[] | select(.labels[].name | contains("priority:medium"))] | .[0].number' 2>/dev/null || echo "null")
  if [ "$MEDIUM" != "null" ]; then
    NEXT_ISSUE="$MEDIUM"
    echo "Found medium priority issue: #$NEXT_ISSUE"
  fi
fi

# Default to oldest ready issue if no priority labels
if [ -z "$NEXT_ISSUE" ] || [ "$NEXT_ISSUE" == "null" ]; then
  NEXT_ISSUE=$(echo "$READY_ISSUES" | jq -s '.[0].number' 2>/dev/null || echo "")
  if [ -n "$NEXT_ISSUE" ] && [ "$NEXT_ISSUE" != "null" ]; then
    echo "Found oldest ready issue (no priority label): #$NEXT_ISSUE"
  fi
fi

# Output result
if [ -n "$NEXT_ISSUE" ] && [ "$NEXT_ISSUE" != "null" ]; then
  echo "issue_number=$NEXT_ISSUE" >> $GITHUB_OUTPUT
  echo "✅ Next issue selected: #$NEXT_ISSUE"
else
  echo "issue_number=" >> $GITHUB_OUTPUT
  echo "No ready issues available"
fi
