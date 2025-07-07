#!/bin/bash
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
NAMESPACE="orchestrator"
PVC_NAME="shared-workspace"
WORKER_NODE="talos-a43-ee1"
ORCHESTRATOR_API_URL="http://orchestrator.orchestrator.svc.cluster.local/api/v1"
AGENT_TEMPLATE_REPO="https://github.com/5dlabs/agent-template"
TEST_REPO_NAME="${TEST_REPO_NAME:-todo-api-test}"
GITHUB_ORG="5dlabs"
GITHUB_USER="swe-1-5dlabs"
TASK_ID="1"

echo -e "${GREEN}=== Claude Workspace Clean Test Script ===${NC}"
echo "This script will:"
echo "1. Clean the PVC completely"
echo "2. Create a fresh test repository from agent-template"
echo "3. Wait for GitHub Actions build to complete"
echo "4. Restart the orchestrator"
echo "5. Submit the test task"
echo ""

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"
command -v kubectl >/dev/null 2>&1 || { echo -e "${RED}kubectl is required but not installed.${NC}" >&2; exit 1; }
command -v gh >/dev/null 2>&1 || { echo -e "${RED}GitHub CLI (gh) is required but not installed.${NC}" >&2; exit 1; }
command -v orchestrator >/dev/null 2>&1 || { echo -e "${RED}orchestrator CLI is required but not installed.${NC}" >&2; exit 1; }

# Check if authenticated to GitHub
gh auth status >/dev/null 2>&1 || { echo -e "${RED}Not authenticated to GitHub. Run 'gh auth login' first.${NC}" >&2; exit 1; }

echo -e "${GREEN}Prerequisites check passed!${NC}"
echo ""

# No confirmation - just show what will happen
echo -e "${YELLOW}Starting clean test:${NC}"
echo " - Cleaning PVC ${PVC_NAME}"
echo " - Recreating repository ${TEST_REPO_NAME}"
echo " - Restarting orchestrator"
echo ""

# Function to wait for GitHub Actions
wait_for_github_build() {
    local repo=$1
    echo -e "${YELLOW}Waiting for GitHub Actions build to complete for ${repo}...${NC}"
    
    # Get the latest workflow run
    local run_id=$(gh run list --repo "${GITHUB_ORG}/${repo}" --workflow=build.yml --limit=1 --json databaseId --jq '.[0].databaseId' 2>/dev/null || echo "")
    
    if [ -z "$run_id" ]; then
        echo -e "${YELLOW}No build workflow found, continuing...${NC}"
        return 0
    fi
    
    # Wait for the workflow to complete
    gh run watch "$run_id" --repo "${GITHUB_ORG}/${repo}" --exit-status || {
        echo -e "${RED}Build failed or was cancelled${NC}"
        return 1
    }
    
    echo -e "${GREEN}Build completed successfully!${NC}"
}

# Step 1: Clean the PVC
echo -e "${GREEN}Step 1: Cleaning PVC ${PVC_NAME}...${NC}"
CLEAN_JOB_NAME="clean-pvc-$(date +%s)"
cat <<EOF | kubectl apply -f -
apiVersion: batch/v1
kind: Job
metadata:
  name: ${CLEAN_JOB_NAME}
  namespace: orchestrator
spec:
  template:
    spec:
      restartPolicy: Never
      nodeSelector:
        kubernetes.io/hostname: talos-a43-ee1
      containers:
      - name: cleaner
        image: busybox
        command: 
        - sh
        - -c
        - |
          echo "Cleaning workspace..."
          if [ -d /workspace ]; then
            # List what we're about to delete
            echo "Current contents:"
            ls -la /workspace/
            
            # Also show service directories
            for dir in /workspace/*/; do
              if [ -d "\$dir" ]; then
                echo "Service directory: \$dir"
                ls -la "\$dir" | head -5
              fi
            done
            
            # Remove everything including hidden files
            find /workspace -mindepth 1 -delete 2>/dev/null || true
            
            echo "Workspace cleaned"
            echo "Verification:"
            ls -la /workspace/
          else
            echo "Workspace directory doesn't exist"
          fi
        volumeMounts:
        - name: workspace
          mountPath: /workspace
      volumes:
      - name: workspace
        persistentVolumeClaim:
          claimName: shared-workspace
EOF

# Wait for cleaning job to complete
echo "Waiting for cleaning job to complete..."
kubectl wait --for=condition=complete job/${CLEAN_JOB_NAME} -n $NAMESPACE --timeout=60s || {
    echo -e "${RED}Cleaning job failed or timed out${NC}"
    exit 1
}

# Show job logs for debugging
echo "Cleaning job output:"
kubectl logs job/${CLEAN_JOB_NAME} -n $NAMESPACE

# Delete the cleaning job
kubectl delete job/${CLEAN_JOB_NAME} -n $NAMESPACE

# Step 2: Create fresh repository from template
echo -e "${GREEN}Step 2: Creating fresh repository from template...${NC}"

# Check if repo exists and delete if it does
if gh repo view "${GITHUB_ORG}/${TEST_REPO_NAME}" &>/dev/null; then
    echo "Repository ${TEST_REPO_NAME} exists, attempting to delete..."
    if gh repo delete "${GITHUB_ORG}/${TEST_REPO_NAME}" --yes 2>/dev/null; then
        echo "Repository deleted successfully"
        sleep 5  # Give GitHub a moment
    else
        echo -e "${YELLOW}Warning: Could not delete repository (may need admin rights)${NC}"
        echo "You may need to manually delete it or use a different test repo name"
        echo "Continuing anyway..."
    fi
fi

# Create new repo from template
echo "Creating new repository from template..."
gh repo create "${GITHUB_ORG}/${TEST_REPO_NAME}" \
    --template="${GITHUB_ORG}/agent-template" \
    --private \
    --description="Test repository for Claude agent testing"

# Step 3: Wait for any GitHub Actions to complete
echo -e "${GREEN}Step 3: Checking for GitHub Actions...${NC}"
sleep 10  # Give GitHub Actions time to trigger
wait_for_github_build "$TEST_REPO_NAME"

# Step 4: Restart orchestrator
echo -e "${GREEN}Step 4: Restarting orchestrator...${NC}"
kubectl rollout restart deployment/orchestrator -n $NAMESPACE
kubectl rollout status deployment/orchestrator -n $NAMESPACE --timeout=300s

# Wait for orchestrator to be ready
echo "Waiting for orchestrator to be ready..."
sleep 10

# Step 5: Submit test task
echo -e "${GREEN}Step 5: Submitting test task...${NC}"

# Ensure .taskmaster directory exists with test files
if [ ! -d ".taskmaster/docs" ]; then
    echo -e "${YELLOW}Creating .taskmaster/docs directory with test files...${NC}"
    mkdir -p .taskmaster/docs
    
    # Don't override if files already exist with our test content
    echo "Using existing .taskmaster/docs files"
fi

# Clean up any existing TaskRuns and Jobs for this task
echo "Cleaning up any existing runs for task $TASK_ID..."
kubectl delete taskrun -n $NAMESPACE "task-$TASK_ID" 2>/dev/null || true
kubectl delete jobs -n $NAMESPACE -l "task-id=$TASK_ID" 2>/dev/null || true

# Submit the task
echo "Submitting task to orchestrator..."
# Always use the Kubernetes service URL - we have Twingate VPN
SUBMIT_URL="http://orchestrator.orchestrator.svc.cluster.local/api/v1"
echo "Using Kubernetes service URL: $SUBMIT_URL"

TASK_RESPONSE=$(ORCHESTRATOR_API_URL="$SUBMIT_URL" orchestrator task submit \
    --service "$TEST_REPO_NAME" \
    --repo "https://github.com/${GITHUB_ORG}/${TEST_REPO_NAME}" \
    --github-user "$GITHUB_USER" \
    "$TASK_ID" 2>&1)

echo "$TASK_RESPONSE"

# Extract task ID from response if available
SUBMITTED_TASK_ID=$(echo "$TASK_RESPONSE" | grep -oP 'Task ID: \K[a-zA-Z0-9-]+' || echo "")

echo -e "${GREEN}=== Test setup complete! ===${NC}"
echo ""
echo "Summary:"
echo " - PVC cleaned: ${PVC_NAME}"
echo " - Repository created: ${GITHUB_ORG}/${TEST_REPO_NAME}"
echo " - Task submitted: ${TASK_ID}"
if [ -n "$SUBMITTED_TASK_ID" ]; then
    echo " - TaskRun ID: ${SUBMITTED_TASK_ID}"
fi
echo ""
echo "Next steps:"
echo "1. Watch both prep and Claude jobs: kubectl get jobs -n $NAMESPACE -l task-id=$TASK_ID -w"
echo "2. Monitor prep job logs: kubectl logs -n $NAMESPACE -l task-id=$TASK_ID,job-type=prep -f"
echo "3. Monitor Claude job logs: kubectl logs -n $NAMESPACE -l task-id=$TASK_ID,job-type!=prep -f"
if [ -n "$SUBMITTED_TASK_ID" ]; then
    echo "4. Check task status with: orchestrator task status ${SUBMITTED_TASK_ID}"
else
    echo "4. Check task status with: orchestrator task status <task-id>"
fi
echo "5. View the agent workspace: kubectl exec -it -n $NAMESPACE <pod-name> -- bash"
echo ""
echo -e "${GREEN}Repository URL: https://github.com/${GITHUB_ORG}/${TEST_REPO_NAME}${NC}"