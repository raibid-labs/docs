#!/bin/bash
# Sparky Data Collection Script (Zero Cost)
# Uses GitHub CLI (gh) which doesn't count against API rate limits

set -euo pipefail

DATE=$(date +%Y-%m-%d)
OUTPUT_DIR="data/raw"
mkdir -p "$OUTPUT_DIR"

echo "🚀 Sparky Data Collection - $DATE"
echo "Using GitHub CLI (free, no API limits)"
echo ""

# Get all raibid-labs repositories
echo "📦 Discovering repositories..."
gh repo list raibid-labs --limit 100 --json name,description,updatedAt,pushedAt > "$OUTPUT_DIR/$DATE-repos.json"
REPO_COUNT=$(jq length "$OUTPUT_DIR/$DATE-repos.json")
echo "   Found $REPO_COUNT repositories"

# Filter repos updated in last 7 days for efficiency
ACTIVE_REPOS=$(jq -r '.[] | select(.pushedAt >= (now - 7*86400 | strftime("%Y-%m-%dT%H:%M:%SZ"))) | .name' "$OUTPUT_DIR/$DATE-repos.json")
ACTIVE_COUNT=$(echo "$ACTIVE_REPOS" | wc -l)
echo "   $ACTIVE_COUNT active in last 7 days"

# Collect commits from active repos
echo ""
echo "📝 Collecting commits (last 24 hours)..."
echo "[]" > "$OUTPUT_DIR/$DATE-commits.json"

while IFS= read -r repo; do
  echo "   Processing: $repo"

  # Get commits from last 24 hours
  gh api "repos/raibid-labs/$repo/commits" \
    --jq ".[] | select(.commit.author.date >= \"$(date -u -d '24 hours ago' '+%Y-%m-%dT%H:%M:%SZ')\") | {
      repo: \"$repo\",
      sha: .sha,
      author: .commit.author.name,
      email: .commit.author.email,
      date: .commit.author.date,
      message: .commit.message
    }" 2>/dev/null || true
done <<< "$ACTIVE_REPOS" | jq -s '.' > "$OUTPUT_DIR/$DATE-commits.json"

COMMIT_COUNT=$(jq length "$OUTPUT_DIR/$DATE-commits.json")
echo "   Collected $COMMIT_COUNT commits"

# Collect merged PRs (last 24 hours)
echo ""
echo "🔀 Collecting merged pull requests..."
echo "[]" > "$OUTPUT_DIR/$DATE-prs.json"

while IFS= read -r repo; do
  echo "   Processing: $repo"

  gh pr list --repo "raibid-labs/$repo" \
    --state merged \
    --limit 50 \
    --json number,title,author,mergedAt,mergedBy,url \
    --jq ".[] | select(.mergedAt >= \"$(date -u -d '24 hours ago' '+%Y-%m-%dT%H:%M:%SZ')\") | . + {repo: \"$repo\"}" 2>/dev/null || true
done <<< "$ACTIVE_REPOS" | jq -s '.' > "$OUTPUT_DIR/$DATE-prs.json"

PR_COUNT=$(jq length "$OUTPUT_DIR/$DATE-prs.json")
echo "   Collected $PR_COUNT merged PRs"

# Collect issues created/updated (last 24 hours)
echo ""
echo "📋 Collecting issue activity..."
echo "[]" > "$OUTPUT_DIR/$DATE-issues.json"

while IFS= read -r repo; do
  echo "   Processing: $repo"

  gh issue list --repo "raibid-labs/$repo" \
    --limit 50 \
    --json number,title,author,createdAt,updatedAt,url,state \
    --jq ".[] | select(.updatedAt >= \"$(date -u -d '24 hours ago' '+%Y-%m-%dT%H:%M:%SZ')\") | . + {repo: \"$repo\"}" 2>/dev/null || true
done <<< "$ACTIVE_REPOS" | jq -s '.' > "$OUTPUT_DIR/$DATE-issues.json"

ISSUE_COUNT=$(jq length "$OUTPUT_DIR/$DATE-issues.json")
echo "   Collected $ISSUE_COUNT issue activities"

# Calculate basic statistics
echo ""
echo "📊 Calculating statistics..."

# Top contributors
TOP_CONTRIBUTORS=$(jq -r '.[].author' "$OUTPUT_DIR/$DATE-commits.json" | sort | uniq -c | sort -rn | head -5)

# Active repos
ACTIVE_REPOS_LIST=$(jq -r '.[].repo' "$OUTPUT_DIR/$DATE-commits.json" | sort -u)

# Create summary
cat > "$OUTPUT_DIR/$DATE-summary.json" <<EOF
{
  "date": "$DATE",
  "commits": $COMMIT_COUNT,
  "pull_requests": $PR_COUNT,
  "issues": $ISSUE_COUNT,
  "active_repositories": $(echo "$ACTIVE_REPOS_LIST" | wc -l),
  "contributors": $(jq -r '.[].author' "$OUTPUT_DIR/$DATE-commits.json" | sort -u | wc -l),
  "top_contributors": $(echo "$TOP_CONTRIBUTORS" | awk '{print "{\"name\": \""$2"\", \"commits\": "$1"}"}' | jq -s '.'),
  "active_repos": $(echo "$ACTIVE_REPOS_LIST" | jq -R . | jq -s '.')
}
EOF

# Display summary
echo ""
echo "✅ Collection Complete!"
echo ""
echo "Summary:"
cat "$OUTPUT_DIR/$DATE-summary.json" | jq .
echo ""
echo "Data files created:"
echo "  - $OUTPUT_DIR/$DATE-repos.json"
echo "  - $OUTPUT_DIR/$DATE-commits.json"
echo "  - $OUTPUT_DIR/$DATE-prs.json"
echo "  - $OUTPUT_DIR/$DATE-issues.json"
echo "  - $OUTPUT_DIR/$DATE-summary.json"
echo ""
echo "💰 Cost: $0 (GitHub CLI is free!)"
