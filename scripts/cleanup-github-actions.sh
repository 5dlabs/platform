#!/bin/bash

# GitHub Actions Run Cleanup Script
# Deletes old workflow runs to clean up repository history

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default configuration
REPO="5dlabs/platform"
DAYS_TO_KEEP=2
DRY_RUN=false
FORCE=false
WORKFLOW_NAME=""
STATUS_FILTER=""
BATCH_SIZE=100
KEEP_SUCCESSFUL=false

# Usage function
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Clean up old GitHub Actions workflow runs

OPTIONS:
    -r, --repo REPO         Repository in format 'owner/repo' (default: $REPO)
    -d, --days DAYS         Keep runs from last N days (default: $DAYS_TO_KEEP)
    -w, --workflow NAME     Only clean specific workflow (e.g., 'build-rust-image.yml')
    -s, --status STATUS     Only clean runs with specific status (completed, failure, cancelled, etc.)
    -b, --batch-size SIZE   Process runs in batches of SIZE (default: $BATCH_SIZE)
    -k, --keep-successful   Keep successful runs, only delete failed/cancelled ones
    --dry-run               Show what would be deleted without actually deleting
    --force                 Skip confirmation prompts
    -h, --help              Show this help message

EXAMPLES:
    $0 --days 7                              # Keep last 7 days
    $0 --days 1 --workflow build-rust-image.yml  # Clean specific workflow, keep 1 day
    $0 --status failure --days 30            # Clean only failed runs older than 30 days
    $0 --keep-successful --days 3            # Keep successful runs, delete failed ones older than 3 days
    $0 --dry-run --days 2                    # Preview what would be deleted
    $0 --force --days 0                      # Delete ALL runs without confirmation

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -r|--repo)
            REPO="$2"
            shift 2
            ;;
        -d|--days)
            DAYS_TO_KEEP="$2"
            shift 2
            ;;
        -w|--workflow)
            WORKFLOW_NAME="$2"
            shift 2
            ;;
        -s|--status)
            STATUS_FILTER="$2"
            shift 2
            ;;
        -b|--batch-size)
            BATCH_SIZE="$2"
            shift 2
            ;;
        -k|--keep-successful)
            KEEP_SUCCESSFUL=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --force)
            FORCE=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            exit 1
            ;;
    esac
done

# Validate prerequisites
if ! command -v gh &> /dev/null; then
    echo -e "${RED}Error: GitHub CLI (gh) is required but not installed${NC}"
    echo "Install with: brew install gh (macOS) or apt install gh (Ubuntu)"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo -e "${RED}Error: jq is required but not installed${NC}"
    echo "Install with: brew install jq (macOS) or apt install jq (Ubuntu)"
    exit 1
fi

# Check GitHub authentication
if ! gh auth status &>/dev/null; then
    echo -e "${RED}Error: Not authenticated with GitHub${NC}"
    echo "Run: gh auth login"
    exit 1
fi

# Calculate cutoff date
if [ "$DAYS_TO_KEEP" -eq 0 ]; then
    CUTOFF_DATE="9999-12-31T23:59:59Z"  # Far future date to delete everything
    CUTOFF_DISPLAY="ALL RUNS"
else
    # Handle both macOS and Linux date commands
    if date -v-1d >/dev/null 2>&1; then
        # macOS date command
        CUTOFF_DATE=$(date -u -v-${DAYS_TO_KEEP}d +%Y-%m-%dT%H:%M:%SZ)
    else
        # Linux date command
        CUTOFF_DATE=$(date -u -d "$DAYS_TO_KEEP days ago" +%Y-%m-%dT%H:%M:%SZ)
    fi
    CUTOFF_DISPLAY="runs older than $DAYS_TO_KEEP days (before $CUTOFF_DATE)"
fi

echo -e "${BLUE}GitHub Actions Cleanup Script${NC}"
echo "=================================="
echo -e "Repository: ${CYAN}$REPO${NC}"
echo -e "Target: ${YELLOW}$CUTOFF_DISPLAY${NC}"
if [ -n "$WORKFLOW_NAME" ]; then
    echo -e "Workflow: ${CYAN}$WORKFLOW_NAME${NC}"
fi
if [ -n "$STATUS_FILTER" ]; then
    echo -e "Status filter: ${CYAN}$STATUS_FILTER${NC}"
fi
if [ "$KEEP_SUCCESSFUL" = true ]; then
    echo -e "Mode: ${GREEN}Keep successful runs, delete failed/cancelled only${NC}"
fi
if [ "$DRY_RUN" = true ]; then
    echo -e "Mode: ${YELLOW}DRY RUN - No runs will be deleted${NC}"
fi
echo ""

# Function to get workflow runs
get_workflow_runs() {
    local workflow_filter=""
    local status_filter=""

    if [ -n "$WORKFLOW_NAME" ]; then
        workflow_filter="--workflow=$WORKFLOW_NAME"
    fi

    if [ -n "$STATUS_FILTER" ]; then
        status_filter="--status=$STATUS_FILTER"
    fi

    gh run list \
        --repo "$REPO" \
        $workflow_filter \
        $status_filter \
        --limit "$BATCH_SIZE" \
        --json databaseId,status,conclusion,createdAt,workflowName,displayTitle,headBranch,event
}

# Function to check if a run should be deleted
should_delete_run() {
    local run_data="$1"
    local created_at=$(echo "$run_data" | jq -r '.createdAt')
    local status=$(echo "$run_data" | jq -r '.status')
    local conclusion=$(echo "$run_data" | jq -r '.conclusion')

    # Check if run is older than cutoff date
    if [ "$DAYS_TO_KEEP" -ne 0 ]; then
        if [[ "$created_at" > "$CUTOFF_DATE" ]]; then
            return 1  # Don't delete - too recent
        fi
    fi

    # If keeping successful runs, check conclusion
    if [ "$KEEP_SUCCESSFUL" = true ]; then
        if [ "$conclusion" = "success" ]; then
            return 1  # Don't delete - successful run
        fi
    fi

    return 0  # Delete this run
}

# Function to delete a run
delete_run() {
    local run_id="$1"
    local run_info="$2"

    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[DRY RUN] Would delete: $run_info${NC}"
        return 0
    fi

    if gh run delete "$run_id" --repo "$REPO" &>/dev/null; then
        echo -e "${GREEN}✓ Deleted: $run_info${NC}"
        return 0
    else
        echo -e "${RED}✗ Failed to delete: $run_info${NC}"
        return 1
    fi
}

# Function to format run info for display
format_run_info() {
    local run_data="$1"
    local run_id=$(echo "$run_data" | jq -r '.databaseId')
    local workflow=$(echo "$run_data" | jq -r '.workflowName')
    local title=$(echo "$run_data" | jq -r '.displayTitle')
    local branch=$(echo "$run_data" | jq -r '.headBranch')
    local status=$(echo "$run_data" | jq -r '.status')
    local conclusion=$(echo "$run_data" | jq -r '.conclusion')
    local created_at=$(echo "$run_data" | jq -r '.createdAt')

    # Format date for display (handle both macOS and Linux)
    local date_display
    if date -v-1d >/dev/null 2>&1; then
        # macOS date command
        date_display=$(date -j -f "%Y-%m-%dT%H:%M:%SZ" "$created_at" +"%Y-%m-%d %H:%M" 2>/dev/null || echo "$created_at")
    else
        # Linux date command
        date_display=$(date -d "$created_at" +"%Y-%m-%d %H:%M" 2>/dev/null || echo "$created_at")
    fi

    echo "Run #$run_id [$workflow] $title ($branch) - $status/$conclusion - $date_display"
}

# Main cleanup logic
echo -e "${BLUE}Scanning for runs to delete...${NC}"
echo ""

# Get all workflow runs
echo "Fetching workflow runs..."
RUNS_JSON=$(get_workflow_runs)

if [ -z "$RUNS_JSON" ] || [ "$RUNS_JSON" = "null" ] || [ "$RUNS_JSON" = "[]" ]; then
    echo -e "${YELLOW}No workflow runs found matching criteria${NC}"
    exit 0
fi

# Count total runs
TOTAL_RUNS=$(echo "$RUNS_JSON" | jq '. | length')
echo "Found $TOTAL_RUNS runs to analyze"
echo ""

# Process runs and collect those to delete
TO_DELETE=()
TO_KEEP=()

echo "$RUNS_JSON" | jq -c '.[]' | while read -r run; do
    run_info=$(format_run_info "$run")

    if should_delete_run "$run"; then
        TO_DELETE+=("$run")
        echo -e "${RED}🗑️  $run_info${NC}"
    else
        TO_KEEP+=("$run")
        echo -e "${GREEN}✓ $run_info${NC}"
    fi
done

# Count runs to delete vs keep
DELETE_COUNT=$(echo "$RUNS_JSON" | jq -c '.[]' | while read -r run; do
    if should_delete_run "$run"; then
        echo "delete"
    fi
done | wc -l)

KEEP_COUNT=$((TOTAL_RUNS - DELETE_COUNT))

echo ""
echo -e "${BLUE}Summary:${NC}"
echo -e "  Total runs found: $TOTAL_RUNS"
echo -e "  Runs to delete: ${RED}$DELETE_COUNT${NC}"
echo -e "  Runs to keep: ${GREEN}$KEEP_COUNT${NC}"
echo ""

if [ "$DELETE_COUNT" -eq 0 ]; then
    echo -e "${GREEN}No runs to delete!${NC}"
    exit 0
fi

# Confirmation prompt
if [ "$FORCE" != true ] && [ "$DRY_RUN" != true ]; then
    echo -e "${YELLOW}Are you sure you want to delete $DELETE_COUNT workflow runs?${NC}"
    echo -n "Type 'yes' to confirm: "
    read -r response
    if [ "$response" != "yes" ]; then
        echo "Cleanup cancelled."
        exit 0
    fi
    echo ""
fi

# Delete runs
echo -e "${BLUE}Deleting runs...${NC}"
DELETED_COUNT=0
FAILED_COUNT=0

echo "$RUNS_JSON" | jq -c '.[]' | while read -r run; do
    if should_delete_run "$run"; then
        run_id=$(echo "$run" | jq -r '.databaseId')
        run_info=$(format_run_info "$run")

        if delete_run "$run_id" "$run_info"; then
            ((DELETED_COUNT++))
        else
            ((FAILED_COUNT++))
        fi

        # Add small delay to avoid rate limiting
        sleep 0.1
    fi
done

echo ""
echo -e "${BLUE}Cleanup Summary:${NC}"
echo -e "  Successfully deleted: ${GREEN}$DELETED_COUNT${NC}"
if [ "$FAILED_COUNT" -gt 0 ]; then
    echo -e "  Failed to delete: ${RED}$FAILED_COUNT${NC}"
fi
echo ""

if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}This was a dry run. No runs were actually deleted.${NC}"
    echo -e "Remove --dry-run flag to perform actual deletion."
else
    echo -e "${GREEN}Cleanup completed!${NC}"
fi