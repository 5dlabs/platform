#!/bin/bash
# Example script to submit tasks to the orchestrator

# Configuration
ORCHESTRATOR_URL="${ORCHESTRATOR_URL:-http://orchestrator.local}"
API_BASE="${ORCHESTRATOR_URL}/api/v1"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

SERVICE="todo-api"
NAMESPACE="orchestrator"

echo -e "${BLUE}Starting Todo API Example Task Submission${NC}"
echo "This script demonstrates how to submit tasks to the AI agent platform"
echo "Orchestrator URL: ${ORCHESTRATOR_URL}"
echo ""

# Check if orchestrator is reachable
echo -n "Checking orchestrator connectivity... "
if curl -s "${ORCHESTRATOR_URL}/health" > /dev/null; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}FAILED${NC}"
    echo "Make sure:"
    echo "  1. Add '192.168.1.72 orchestrator.local' to /etc/hosts"
    echo "  2. Orchestrator is running in the cluster"
    exit 1
fi
echo ""

# Function to submit a task
submit_task() {
    local task_file=$1
    local task_name=$2
    
    echo -e "${BLUE}Submitting task: ${task_name}${NC}"
    
    # In a real implementation, this would use the orchestrator CLI
    # For now, we'll show the command that would be run
    echo "Command: orchestrator-cli task submit \\"
    echo "  --service ${SERVICE} \\"
    echo "  --namespace ${NAMESPACE} \\"
    echo "  --task-file ${task_file}"
    echo ""
    
    # Placeholder for actual submission
    # orchestrator-cli task submit --service ${SERVICE} --task-file ${task_file}
}

# Function to check task status
check_status() {
    echo -e "${BLUE}Checking task status for service: ${SERVICE}${NC}"
    echo "Command: orchestrator-cli task status --service ${SERVICE}"
    echo ""
    
    # Placeholder for actual status check
    # orchestrator-cli task status --service ${SERVICE}
}

# Function to submit via API (until CLI is ready)
submit_task_api() {
    local task_file=$1
    local task_name=$2
    
    echo -e "${BLUE}Submitting task via API: ${task_name}${NC}"
    
    # Read the task file content
    if [ ! -f "$task_file" ]; then
        echo -e "${RED}Error: Task file not found: $task_file${NC}"
        return 1
    fi
    
    # Create JSON payload
    local task_content=$(cat "$task_file" | jq -Rs .)
    local payload=$(cat <<EOF
{
  "service_name": "${SERVICE}",
  "markdown_files": [
    {
      "filename": "task.md",
      "content": ${task_content}
    }
  ]
}
EOF
)
    
    echo "Submitting to: ${API_BASE}/pm/tasks"
    
    # Submit via curl
    response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$payload" \
        "${API_BASE}/pm/tasks")
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}Task submitted successfully!${NC}"
        echo "Response: $response"
        
        # Extract task_id if available
        task_id=$(echo "$response" | jq -r '.task_id // .id // empty' 2>/dev/null)
        if [ -n "$task_id" ]; then
            echo "Task ID: $task_id"
            echo "Monitor with: kubectl get taskruns -n orchestrator"
        fi
    else
        echo -e "${RED}Failed to submit task${NC}"
    fi
    echo ""
}

# Main flow
echo "=== Task Submission Flow ==="
echo ""

# Choose method
echo "Since the CLI is not built yet, we'll demonstrate the API approach:"
echo ""

# Task 1: Setup Project (via API)
# Uncomment to actually submit:
# submit_task_api "tasks/01-setup-project.md" "Initialize Project Structure"

# Show the command that would be used
echo -e "${YELLOW}Example API submission:${NC}"
echo "submit_task_api \"tasks/01-setup-project.md\" \"Initialize Project Structure\""
echo ""

echo -e "${YELLOW}Example CLI submission (once built):${NC}"
submit_task "tasks/01-setup-project.md" "Initialize Project Structure"
echo "Wait for completion before submitting next task..."
echo ""

# Task 2: Create API
# submit_task "tasks/02-create-api.md" "Implement Todo CRUD API"

# Task 3: Add Database
# submit_task "tasks/03-add-database.md" "Add SQLite Database"

# Task 4: Add Tests
# submit_task "tasks/04-add-tests.md" "Add Test Suite"

echo -e "${GREEN}Example task submission complete!${NC}"
echo ""
echo "To actually submit tasks, you need to:"
echo "1. Build and install the orchestrator CLI"
echo "2. Ensure the orchestrator is running in your cluster"
echo "3. Uncomment the actual CLI commands in this script"
echo ""
echo "You can also submit tasks manually:"
echo "  orchestrator-cli task submit --service todo-api --task-file tasks/01-setup-project.md"