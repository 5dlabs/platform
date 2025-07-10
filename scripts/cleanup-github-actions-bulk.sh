#!/bin/bash

# Enhanced GitHub Actions Run Cleanup Script
# Handles large numbers of runs with pagination and bulk deletion

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
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
MAX_RUNS=1000
PARALLEL_JOBS=5

# Usage function
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Enhanced GitHub Actions workflow run cleanup with pagination and bulk operations

OPTIONS:
    -r, --repo REPO         Repository in format 'owner/repo' (default: $REPO)
    -d, --days DAYS         Keep runs from last N days (default: $DAYS_TO_KEEP)
    -w, --workflow NAME     Only clean specific workflow (e.g., 'build-rust-image.yml')
    -s, --status STATUS     Only clean runs with specific status (completed, failure, cancelled, etc.)
    -b, --batch-size SIZE   Process runs in batches of SIZE (default: $BATCH_SIZE)
    -m, --max-runs MAX      Maximum number of runs to process (default: $MAX_RUNS)
    -j, --parallel JOBS     Number of parallel deletion jobs (default: $PARALLEL_JOBS)
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
    $0 --force --days 0 --max-runs 500       # Delete ALL runs (up to 500) without confirmation
    $0 --parallel 10 --days 7                # Use 10 parallel jobs for faster deletion

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
        -m|--max-runs)
            MAX_RUNS="$2"
            shift 2
            ;;
        -j|--parallel)
            PARALLEL_JOBS="$2"
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

echo -e "${BLUE}Enhanced GitHub Actions Cleanup Script${NC}"
echo "========================================"
echo -e "Repository: ${CYAN}$REPO${NC}"
echo -e "Target: ${YELLOW}$CUTOFF_DISPLAY${NC}"
echo -e "Max runs to process: ${CYAN}$MAX_RUNS${NC}"
echo -e "Parallel jobs: ${CYAN}$PARALLEL_JOBS${NC}"
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

# Function to get workflow runs with pagination
get_all_workflow_runs() {
    local workflow_filter=""
    local status_filter=""
    local all_runs="[]"
    local page=1
    local runs_fetched=0

    if [ -n "$WORKFLOW_NAME" ]; then
        workflow_filter="--workflow=$WORKFLOW_NAME"
    fi

    if [ -n "$STATUS_FILTER" ]; then
        status_filter="--status=$STATUS_FILTER"
    fi

    echo "Fetching workflow runs (pagination enabled)..."

    while [ $runs_fetched -lt $MAX_RUNS ]; do
        local limit=$((MAX_RUNS - runs_fetched))
        if [ $limit -gt $BATCH_SIZE ]; then
            limit=$BATCH_SIZE
        fi

        echo -ne "\rFetching page $page (runs: $runs_fetched/$MAX_RUNS)..."

        local batch_runs=$(gh run list \
            --repo "$REPO" \
            $workflow_filter \
            $status_filter \
            --limit "$limit" \
            --json databaseId,status,conclusion,createdAt,workflowName,displayTitle,headBranch,event \
            2>/dev/null || echo "[]")

        if [ "$batch_runs" = "[]" ] || [ -z "$batch_runs" ]; then
            break
        fi

        local batch_count=$(echo "$batch_runs" | jq '. | length')
        if [ "$batch_count" -eq 0 ]; then
            break
        fi

        all_runs=$(echo "$all_runs" "$batch_runs" | jq -s 'add')
        runs_fetched=$((runs_fetched + batch_count))
        page=$((page + 1))

        # If we got fewer runs than requested, we've reached the end
        if [ "$batch_count" -lt "$limit" ]; then
            break
        fi

        # Small delay to avoid rate limiting
        sleep 0.1
    done

    echo -e "\rFetched $runs_fetched runs total"
    echo "$all_runs"
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

# Function to delete a run (used in parallel)
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

# Function to delete runs in parallel
delete_runs_parallel() {
    local runs_to_delete="$1"
    local temp_dir=$(mktemp -d)
    local job_count=0
    local total_jobs=0

    echo -e "${BLUE}Starting parallel deletion (${PARALLEL_JOBS} jobs)...${NC}"
    echo ""

    # Create job files
    echo "$runs_to_delete" | jq -c '.[]' | while read -r run; do
        local run_id=$(echo "$run" | jq -r '.databaseId')
        local run_info=$(format_run_info "$run")

        # Create job file
        local job_file="$temp_dir/job_$total_jobs.sh"
        cat > "$job_file" << EOF
#!/bin/bash
$(declare -f delete_run)
delete_run "$run_id" "$run_info"
EOF
        chmod +x "$job_file"

        total_jobs=$((total_jobs + 1))
    done

    echo "Created $total_jobs deletion jobs"

    # Execute jobs in parallel
    for job_file in "$temp_dir"/job_*.sh; do
        if [ -f "$job_file" ]; then
            # Wait if we have too many background jobs
            while [ $(jobs -r | wc -l) -ge $PARALLEL_JOBS ]; do
                sleep 0.1
            done

            # Start job in background
            "$job_file" &
            job_count=$((job_count + 1))

            # Progress indicator
            if [ $((job_count % 10)) -eq 0 ]; then
                echo -ne "\rProgress: $job_count/$total_jobs jobs started..."
            fi
        fi
    done

    # Wait for all jobs to complete
    wait
    echo -e "\rAll deletion jobs completed!"

    # Clean up temp directory
    rm -rf "$temp_dir"
}

# Main cleanup logic
echo -e "${BLUE}Scanning for runs to delete...${NC}"
echo ""

# Get all workflow runs
RUNS_JSON=$(get_all_workflow_runs)

if [ -z "$RUNS_JSON" ] || [ "$RUNS_JSON" = "null" ] || [ "$RUNS_JSON" = "[]" ]; then
    echo -e "${YELLOW}No workflow runs found matching criteria${NC}"
    exit 0
fi

# Count total runs
TOTAL_RUNS=$(echo "$RUNS_JSON" | jq '. | length')
echo ""
echo "Analyzing $TOTAL_RUNS runs..."

# Separate runs to delete vs keep
RUNS_TO_DELETE=$(echo "$RUNS_JSON" | jq '[.[] | select(. as $run |
    (if "'$DAYS_TO_KEEP'" != "0" then ($run.createdAt <= "'$CUTOFF_DATE'") else true end) and
    (if "'$KEEP_SUCCESSFUL'" == "true" then ($run.conclusion != "success") else true end)
)]')

RUNS_TO_KEEP=$(echo "$RUNS_JSON" | jq '[.[] | select(. as $run |
    (if "'$DAYS_TO_KEEP'" != "0" then ($run.createdAt > "'$CUTOFF_DATE'") else false end) or
    (if "'$KEEP_SUCCESSFUL'" == "true" then ($run.conclusion == "success") else false end)
)]')

DELETE_COUNT=$(echo "$RUNS_TO_DELETE" | jq '. | length')
KEEP_COUNT=$(echo "$RUNS_TO_KEEP" | jq '. | length')

echo ""
echo -e "${BLUE}Analysis Results:${NC}"
echo -e "  Total runs analyzed: $TOTAL_RUNS"
echo -e "  Runs to delete: ${RED}$DELETE_COUNT${NC}"
echo -e "  Runs to keep: ${GREEN}$KEEP_COUNT${NC}"

# Show sample of runs to delete
if [ "$DELETE_COUNT" -gt 0 ]; then
    echo ""
    echo -e "${YELLOW}Sample of runs to delete:${NC}"
    echo "$RUNS_TO_DELETE" | jq -c '.[:5][]' | while read -r run; do
        run_info=$(format_run_info "$run")
        echo -e "${RED}🗑️  $run_info${NC}"
    done

    if [ "$DELETE_COUNT" -gt 5 ]; then
        echo -e "${YELLOW}... and $((DELETE_COUNT - 5)) more${NC}"
    fi
fi

# Show sample of runs to keep
if [ "$KEEP_COUNT" -gt 0 ]; then
    echo ""
    echo -e "${GREEN}Sample of runs to keep:${NC}"
    echo "$RUNS_TO_KEEP" | jq -c '.[:3][]' | while read -r run; do
        run_info=$(format_run_info "$run")
        echo -e "${GREEN}✓ $run_info${NC}"
    done

    if [ "$KEEP_COUNT" -gt 3 ]; then
        echo -e "${GREEN}... and $((KEEP_COUNT - 3)) more${NC}"
    fi
fi

echo ""

if [ "$DELETE_COUNT" -eq 0 ]; then
    echo -e "${GREEN}No runs to delete!${NC}"
    exit 0
fi

# Confirmation prompt
if [ "$FORCE" != true ] && [ "$DRY_RUN" != true ]; then
    echo -e "${YELLOW}Are you sure you want to delete $DELETE_COUNT workflow runs?${NC}"
    echo -e "This will use ${PARALLEL_JOBS} parallel deletion jobs for faster processing."
    echo -n "Type 'yes' to confirm: "
    read -r response
    if [ "$response" != "yes" ]; then
        echo "Cleanup cancelled."
        exit 0
    fi
    echo ""
fi

# Delete runs (in parallel if not dry run)
if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}DRY RUN - Showing all runs that would be deleted:${NC}"
    echo "$RUNS_TO_DELETE" | jq -c '.[]' | while read -r run; do
        run_info=$(format_run_info "$run")
        echo -e "${YELLOW}[DRY RUN] Would delete: $run_info${NC}"
    done
else
    delete_runs_parallel "$RUNS_TO_DELETE"
fi

echo ""
echo -e "${BLUE}Cleanup Summary:${NC}"
echo -e "  Total runs processed: $TOTAL_RUNS"
echo -e "  Runs deleted: ${RED}$DELETE_COUNT${NC}"
echo -e "  Runs kept: ${GREEN}$KEEP_COUNT${NC}"
echo ""

if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}This was a dry run. No runs were actually deleted.${NC}"
    echo -e "Remove --dry-run flag to perform actual deletion."
else
    echo -e "${GREEN}Cleanup completed!${NC}"
    echo -e "Used ${PARALLEL_JOBS} parallel jobs for faster processing."
fi