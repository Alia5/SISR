#!/bin/bash
set -euo pipefail

# Usage: generate-changelog.sh <output_file> [tag_or_range]
# If tag_or_range is empty, generates from latest tag to HEAD

OUTPUT_FILE="$1"
TAG_OR_RANGE="${2:-}"

mkdir -p "$(dirname "$OUTPUT_FILE")"

if [[ -z "$TAG_OR_RANGE" ]]; then
  # Dev version - from latest tag to HEAD
  LATEST_TAG=$(git describe --tags --abbrev=0 --match "v*.*.*" 2>/dev/null || echo "")
  if [[ -z "$LATEST_TAG" ]]; then
    LOG_RANGE=""
    CONTEXT_MSG="All unreleased changes:"
    VERSION_TITLE="Development Version"
  else
    LOG_RANGE="$LATEST_TAG..HEAD"
    CONTEXT_MSG="Changes since $LATEST_TAG:"
    VERSION_TITLE="Development Version"
  fi
else
  # Tagged version
  VERSION="$TAG_OR_RANGE"
  PREVIOUS_TAG=$(git describe --tags --abbrev=0 --match "v*.*.*" "$TAG_OR_RANGE^" 2>/dev/null || echo "")
  if [[ -z "$PREVIOUS_TAG" ]]; then
    LOG_RANGE="$TAG_OR_RANGE"
    CONTEXT_MSG="All changes in this release:"
  else
    LOG_RANGE="$PREVIOUS_TAG..$TAG_OR_RANGE"
    CONTEXT_MSG="Changes since $PREVIOUS_TAG:"
  fi
  VERSION_TITLE="Version ${VERSION#v}"
fi

# Collect commits
mapfile -t COMMITS < <(git log --pretty=format:'%H' $LOG_RANGE)
FEATURES=""
FIXES=""
MISC=""

for commit_hash in "${COMMITS[@]}"; do
  commit_msg=$(git log -1 --pretty=format:'%s' "$commit_hash")
  commit_body=$(git log -1 --pretty=format:'%b' "$commit_hash")
  # Extract changelog type from subject or body (case-insensitive, robust)
  changelog_type=""
  # Check subject first
  if echo "$commit_msg" | grep -iqE 'changelog[(: ]'; then
    if echo "$commit_msg" | grep -iqE 'changelog\((feature|feat)\)'; then
      changelog_type="feature"
    elif echo "$commit_msg" | grep -iqE 'changelog\((fix)\)'; then
      changelog_type="fix"
    elif echo "$commit_msg" | grep -iqE 'changelog\((misc)\)'; then
      changelog_type="misc"
    fi
  fi
  # If not found in subject, check body
  if [ -z "$changelog_type" ]; then
    if echo "$commit_body" | grep -iqE 'changelog[(: ]'; then
      if echo "$commit_body" | grep -iqE 'changelog\((feature|feat)\)'; then
        changelog_type="feature"
      elif echo "$commit_body" | grep -iqE 'changelog\((fix)\)'; then
        changelog_type="fix"
      elif echo "$commit_body" | grep -iqE 'changelog\((misc)\)'; then
        changelog_type="misc"
      fi
    fi
  fi
  if [ -n "$changelog_type" ]; then
    body_content=$(echo "$commit_body" | awk 'BEGIN{IGNORECASE=1} !/changelog[(: ]/ && NF')
    entry="- $commit_msg"
    if [ -n "$body_content" ]; then
      entry=$(printf "%s\n%s" "$entry" "$(echo "$body_content" | sed 's/^/  /')")
    fi
    if [ "$changelog_type" = "feature" ]; then
      FEATURES=$(printf "%s\n%s" "$FEATURES" "$entry")
    elif [ "$changelog_type" = "fix" ]; then
      FIXES=$(printf "%s\n%s" "$FIXES" "$entry")
    else
      MISC=$(printf "%s\n%s" "$MISC" "$entry")
    fi
  fi
done

# Write changelog
{
  echo "# Changelog for $VERSION_TITLE"
  echo ""
  if [[ -z "$TAG_OR_RANGE" ]]; then
    echo "This page shows unreleased changes in the development version."
  else
    echo "Release Date: $(date +'%Y-%m-%d')"
  fi
  echo ""
  echo "$CONTEXT_MSG"
  echo ""
  
  if [[ -n "$FEATURES" ]]; then
    echo "## ✨ New Features"
    echo ""
    echo "$FEATURES"
    echo ""
  fi
  
  if [[ -n "$FIXES" ]]; then
    echo "## 🐛 Fixes"
    echo ""
    echo "$FIXES"
    echo ""
  fi
  
  if [[ -n "$MISC" ]]; then
    echo "## 🔧 Miscellaneous"
    echo ""
    echo "$MISC"
    echo ""
  fi
  
  if [[ -z "$FEATURES" && -z "$FIXES" && -z "$MISC" ]]; then
    if [[ -z "$TAG_OR_RANGE" ]]; then
      echo "No changes yet."
    else
      echo "No changes."
    fi
  fi
} > "$OUTPUT_FILE"

echo "Changelog generated: $OUTPUT_FILE"
