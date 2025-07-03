#!/bin/bash

# Monitor GitHub Actions workflow runs
REPO="5dlabs/platform"
WORKFLOW_FILE="unified-ci.yml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "🔍 Monitoring GitHub Actions for $REPO..."
echo "Workflow: $WORKFLOW_FILE"
echo ""

# Function to get latest workflow runs
get_workflow_runs() {
    curl -s \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/$REPO/actions/workflows/$WORKFLOW_FILE/runs?per_page=5"
}

# Function to get workflow run details
get_run_details() {
    local run_id=$1
    curl -s \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/$REPO/actions/runs/$run_id"
}

# Function to get jobs for a run
get_run_jobs() {
    local run_id=$1
    curl -s \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/$REPO/actions/runs/$run_id/jobs"
}

# Track seen failures
declare -A seen_failures

# Main monitoring loop
while true; do
    clear
    echo "🔍 GitHub Actions Monitor - $(date)"
    echo "================================"
    echo ""

    # Get latest runs
    runs_json=$(get_workflow_runs)

    # Check if we got valid JSON
    if ! echo "$runs_json" | jq empty 2>/dev/null; then
        echo -e "${RED}Error: Invalid JSON response from GitHub API${NC}"
        echo "Response: $runs_json"
        sleep 5
        continue
    fi

    # Check if we have workflow runs
    if [ "$(echo "$runs_json" | jq -r '.workflow_runs | length')" -eq 0 ]; then
        echo "No workflow runs found yet..."
        sleep 5
        continue
    fi

    # Parse and display runs
    echo "$runs_json" | jq -r '.workflow_runs[0:3] | .[] |
        "Run #\(.run_number) - \(.status) (\(.conclusion // "in progress"))\n" +
        "  Branch: \(.head_branch)\n" +
        "  Commit: \(.head_sha[0:7]) - \(.head_commit.message | split("\n")[0])\n" +
        "  Started: \(.created_at)\n" +
        "  ID: \(.id)\n"' 2>/dev/null || echo "Error parsing workflow runs"

    # Get details of the latest run
    latest_run_id=$(echo "$runs_json" | jq -r '.workflow_runs[0].id' 2>/dev/null || echo "null")
    latest_run_status=$(echo "$runs_json" | jq -r '.workflow_runs[0].status' 2>/dev/null || echo "null")

    if [ "$latest_run_id" != "null" ] && [ "$latest_run_id" != "" ]; then
        echo "📊 Latest Run Details (ID: $latest_run_id)"
        echo "================================"

        # Get jobs for the latest run
        jobs_json=$(get_run_jobs "$latest_run_id")

        # Check if we got valid jobs JSON
        if ! echo "$jobs_json" | jq empty 2>/dev/null; then
            echo -e "${RED}Error: Invalid JSON response for jobs${NC}"
            sleep 5
            continue
        fi

        # Display all jobs with their current status
        if [ "$(echo "$jobs_json" | jq -r '.jobs | length')" -gt 0 ]; then
            echo "$jobs_json" | jq -r '.jobs[] |
                "Job: \(.name)\n" +
                "  Status: \(.status) (\(.conclusion // "running"))\n" +
                "  Started: \(.started_at // "pending")\n"' 2>/dev/null || echo "Error parsing jobs"

            # Check for ANY failures (including in-progress failures)
            failed_jobs=$(echo "$jobs_json" | jq -r '.jobs[] | select(.conclusion == "failure") | "\(.id):\(.name)"' 2>/dev/null || echo "")

            if [ -n "$failed_jobs" ]; then
                echo -e "\n${RED}❌ FAILED JOBS DETECTED:${NC}"

                while IFS= read -r job_info; do
                    job_id="${job_info%%:*}"
                    job_name="${job_info#*:}"

                    if [ -z "${seen_failures[$job_id]}" ]; then
                        seen_failures[$job_id]=1
                        echo -e "${RED}NEW FAILURE: $job_name${NC}"

                        # Get detailed failure info
                        echo "$jobs_json" | jq -r --arg name "$job_name" '.jobs[] | select(.name == $name) |
                            "Failed Steps:\n" +
                            (.steps[]? | select(.conclusion == "failure") | "  - \(.name)\n    Status: \(.conclusion)")' 2>/dev/null || echo "  Unable to get step details"

                        echo -e "\n${YELLOW}🔧 IMMEDIATE ACTION REQUIRED!${NC}"
                        echo "View logs at: https://github.com/$REPO/actions/runs/$latest_run_id"
                        echo ""

                        # Break out to handle the failure
                        echo -e "${YELLOW}Failure detected! Breaking monitoring loop to investigate...${NC}"
                        exit 1
                    fi
                done <<< "$failed_jobs"
            fi

            # Also check for jobs that are failing (not yet concluded)
            failing_steps=$(echo "$jobs_json" | jq -r '.jobs[] |
                select(.status == "in_progress") |
                .steps[]? |
                select(.status == "in_progress" and .conclusion == null) |
                "\(.name)"' 2>/dev/null || echo "")

            if [ -n "$failing_steps" ]; then
                echo -e "\n${YELLOW}⚠️  Currently Running Steps:${NC}"
                echo "$failing_steps" | while read -r step; do
                    [ -n "$step" ] && echo "  - $step"
                done
            fi
        else
            echo "No jobs found for this run yet..."
        fi
    fi

    # Check workflow conclusion
    if [ "$latest_run_status" = "completed" ]; then
        conclusion=$(echo "$runs_json" | jq -r '.workflow_runs[0].conclusion' 2>/dev/null || echo "null")
        if [ "$conclusion" = "success" ]; then
            echo -e "\n${GREEN}✅ Workflow completed successfully!${NC}"
            break
        elif [ "$conclusion" = "failure" ]; then
            echo -e "\n${RED}❌ Workflow failed!${NC}"
            exit 1
        fi
    fi

    echo ""
    echo "Refreshing in 5 seconds... (Press Ctrl+C to exit)"
    sleep 5
done