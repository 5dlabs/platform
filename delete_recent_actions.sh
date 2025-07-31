#!/bin/bash

# Script to delete GitHub Actions runs from the last 2 days
# Requires GitHub CLI (gh) to be installed and authenticated

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🔍 Fetching GitHub Actions runs from the last 2 days...${NC}"

# Calculate the date 2 days ago in ISO format
two_days_ago=$(date -v-2d +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date -d "2 days ago" +"%Y-%m-%dT%H:%M:%SZ")

echo -e "${YELLOW}📅 Looking for runs created after: ${two_days_ago}${NC}"

# Get workflow runs from the last 2 days
runs=$(gh run list --created ">${two_days_ago}" --json databaseId --jq '.[].databaseId')

if [ -z "$runs" ]; then
    echo -e "${GREEN}✅ No workflow runs found from the last 2 days to delete.${NC}"
    exit 0
fi

# Count the runs
run_count=$(echo "$runs" | wc -l | xargs)

echo -e "${YELLOW}📊 Found ${run_count} workflow runs to delete${NC}"

# Ask for confirmation
echo -e "${RED}⚠️  This will permanently delete ${run_count} GitHub Actions runs.${NC}"
read -p "Are you sure you want to continue? (y/N): " -n 1 -r
echo

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}❌ Operation cancelled.${NC}"
    exit 0
fi

echo -e "${YELLOW}🗑️  Deleting workflow runs...${NC}"

# Delete each run
deleted_count=0
failed_count=0

for run_id in $runs; do
    echo -n "Deleting run ${run_id}... "
    if gh run delete "$run_id" --confirm >/dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        ((deleted_count++))
    else
        echo -e "${RED}✗${NC}"
        ((failed_count++))
    fi
done

echo
echo -e "${GREEN}✅ Deletion complete!${NC}"
echo -e "${GREEN}   Successfully deleted: ${deleted_count} runs${NC}"
if [ $failed_count -gt 0 ]; then
    echo -e "${RED}   Failed to delete: ${failed_count} runs${NC}"
fi

echo -e "${YELLOW}💡 Note: Some runs might fail to delete if they are still in progress or protected.${NC}"