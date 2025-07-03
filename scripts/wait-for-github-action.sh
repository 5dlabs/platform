#!/bin/bash
# Script to wait for a GitHub Actions run to complete

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
POLL_INTERVAL=10
WORKFLOW_NAME=""
RUN_ID=""
TIMEOUT=1800  # 30 minutes default

# Usage function
usage() {
    echo "Usage: $0 [-w workflow_name] [-r run_id] [-i interval] [-t timeout]"
    echo "  -w: Workflow name (e.g., build-orchestrator.yml)"
    echo "  -r: Specific run ID to monitor"
    echo "  -i: Polling interval in seconds (default: 10)"
    echo "  -t: Timeout in seconds (default: 1800)"
    echo ""
    echo "Examples:"
    echo "  $0 -w build-orchestrator.yml          # Monitor latest run of workflow"
    echo "  $0 -r 1234567890                      # Monitor specific run ID"
    echo "  $0 -w build-orchestrator.yml -i 5     # Check every 5 seconds"
    exit 1
}

# Parse command line arguments
while getopts "w:r:i:t:h" opt; do
    case $opt in
        w) WORKFLOW_NAME="$OPTARG" ;;
        r) RUN_ID="$OPTARG" ;;
        i) POLL_INTERVAL="$OPTARG" ;;
        t) TIMEOUT="$OPTARG" ;;
        h) usage ;;
        *) usage ;;
    esac
done

# Validate inputs
if [[ -z "$WORKFLOW_NAME" && -z "$RUN_ID" ]]; then
    echo -e "${RED}Error: Either workflow name (-w) or run ID (-r) must be specified${NC}"
    usage
fi

# Function to get the latest run ID for a workflow
get_latest_run_id() {
    local workflow=$1
    gh run list --workflow="$workflow" --limit=1 --json databaseId --jq '.[0].databaseId' 2>/dev/null || echo ""
}

# Function to get run status
get_run_status() {
    local run_id=$1
    gh run view "$run_id" --json status,conclusion --jq '.status + "|" + .conclusion' 2>/dev/null || echo "error|error"
}

# Function to get run details
get_run_details() {
    local run_id=$1
    gh run view "$run_id" --json name,displayTitle,event,headBranch,startedAt 2>/dev/null || echo "{}"
}

# Function to format duration
format_duration() {
    local seconds=$1
    local hours=$((seconds / 3600))
    local minutes=$(( (seconds % 3600) / 60 ))
    local secs=$((seconds % 60))
    
    if [[ $hours -gt 0 ]]; then
        printf "%dh %dm %ds" $hours $minutes $secs
    elif [[ $minutes -gt 0 ]]; then
        printf "%dm %ds" $minutes $secs
    else
        printf "%ds" $secs
    fi
}

# Main monitoring logic
main() {
    # If workflow name provided, get the latest run ID
    if [[ -n "$WORKFLOW_NAME" && -z "$RUN_ID" ]]; then
        echo -e "${BLUE}Finding latest run for workflow: $WORKFLOW_NAME${NC}"
        RUN_ID=$(get_latest_run_id "$WORKFLOW_NAME")
        
        if [[ -z "$RUN_ID" ]]; then
            echo -e "${RED}Error: No runs found for workflow $WORKFLOW_NAME${NC}"
            exit 1
        fi
    fi
    
    echo -e "${BLUE}Monitoring GitHub Actions run: $RUN_ID${NC}"
    
    # Get initial run details
    local run_details=$(get_run_details "$RUN_ID")
    local run_name=$(echo "$run_details" | jq -r '.name // "Unknown"')
    local run_title=$(echo "$run_details" | jq -r '.displayTitle // "Unknown"')
    local run_branch=$(echo "$run_details" | jq -r '.headBranch // "Unknown"')
    
    echo -e "${BLUE}Workflow: ${NC}$run_name"
    echo -e "${BLUE}Title: ${NC}$run_title"
    echo -e "${BLUE}Branch: ${NC}$run_branch"
    echo -e "${BLUE}Run URL: ${NC}https://github.com/${GITHUB_REPOSITORY:-5dlabs/platform}/actions/runs/$RUN_ID"
    echo ""
    
    local start_time=$(date +%s)
    local elapsed=0
    local status=""
    local conclusion=""
    local last_status=""
    
    # Monitoring loop
    while true; do
        # Get current status
        local status_result=$(get_run_status "$RUN_ID")
        status=$(echo "$status_result" | cut -d'|' -f1)
        conclusion=$(echo "$status_result" | cut -d'|' -f2)
        
        # Calculate elapsed time
        local current_time=$(date +%s)
        elapsed=$((current_time - start_time))
        
        # Update status if changed
        if [[ "$status" != "$last_status" ]]; then
            echo -e "\n${YELLOW}Status changed to: $status${NC}"
            last_status="$status"
        fi
        
        # Check if completed
        if [[ "$status" == "completed" ]]; then
            echo ""
            if [[ "$conclusion" == "success" ]]; then
                echo -e "${GREEN}✅ Run completed successfully!${NC}"
                echo -e "${GREEN}Total duration: $(format_duration $elapsed)${NC}"
                exit 0
            else
                echo -e "${RED}❌ Run failed with conclusion: $conclusion${NC}"
                echo -e "${RED}Total duration: $(format_duration $elapsed)${NC}"
                echo -e "${RED}View logs: gh run view $RUN_ID --log-failed${NC}"
                exit 1
            fi
        fi
        
        # Check timeout
        if [[ $elapsed -gt $TIMEOUT ]]; then
            echo -e "\n${RED}❌ Timeout reached after $(format_duration $TIMEOUT)${NC}"
            echo -e "${RED}Run is still in status: $status${NC}"
            exit 1
        fi
        
        # Show progress
        printf "\r${YELLOW}⏳ Waiting... [$(format_duration $elapsed)] Status: $status${NC}"
        
        # Wait before next check
        sleep "$POLL_INTERVAL"
    done
}

# Check if gh CLI is installed and authenticated
if ! command -v gh &> /dev/null; then
    echo -e "${RED}Error: GitHub CLI (gh) is not installed${NC}"
    echo "Install with: brew install gh"
    exit 1
fi

# Check authentication
if ! gh auth status &> /dev/null; then
    echo -e "${RED}Error: Not authenticated with GitHub CLI${NC}"
    echo "Run: gh auth login"
    exit 1
fi

# Run main logic
main