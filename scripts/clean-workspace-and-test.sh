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
TASK_ID="9999"

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

# Confirmation prompt
echo -e "${YELLOW}WARNING: This will:${NC}"
echo " - Delete ALL data in PVC ${PVC_NAME}"
echo " - Delete and recreate repository ${TEST_REPO_NAME}"
echo " - Restart the orchestrator deployment"
echo ""
read -p "Are you sure you want to continue? (yes/no): " -r
if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
    echo "Aborted."
    exit 0
fi
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
    
    cat > .taskmaster/docs/design-spec.md << 'EOFD'
# Design Specification - Task 9999

## Objective
Verify workspace setup and create test results.

## Requirements
1. Verify git repository is properly cloned
2. Create test results file
3. Create workspace snapshot
4. Commit and push changes
EOFD

    cat > .taskmaster/docs/prompt.md << 'EOFP'
# Task 9999 - Workspace Verification

Please verify the workspace setup and create appropriate test files.
EOFP

    cat > .taskmaster/docs/acceptance-criteria.md << 'EOFA'
# Acceptance Criteria

- [ ] Workspace is properly configured
- [ ] Test results created
- [ ] Changes committed to repository
EOFA
fi

# Submit the task
echo "Submitting task to orchestrator..."
# Note: This assumes kubectl port-forward is running on localhost:8080
# Or run this script from within a Kubernetes pod to use the service URL directly
if command -v kubectl &> /dev/null && kubectl auth can-i get pods &> /dev/null; then
    # We're running outside the cluster, use port-forward
    echo "Running outside cluster - using localhost:8080 (ensure port-forward is active)"
    SUBMIT_URL="http://localhost:8080/api/v1"
else
    # We're running inside the cluster, use service URL
    echo "Running inside cluster - using service URL"
    SUBMIT_URL="$ORCHESTRATOR_API_URL"
fi

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
echo "1. Monitor the task with: kubectl logs -n $NAMESPACE -l task-id=$TASK_ID -f"
if [ -n "$SUBMITTED_TASK_ID" ]; then
    echo "2. Check task status with: orchestrator task status ${SUBMITTED_TASK_ID}"
else
    echo "2. Check task status with: orchestrator task status <task-id>"
fi
echo "3. View the agent workspace: kubectl exec -it -n $NAMESPACE <pod-name> -- bash"
echo "4. Watch the job: kubectl get jobs -n $NAMESPACE -w"
echo ""
echo -e "${GREEN}Repository URL: https://github.com/${GITHUB_ORG}/${TEST_REPO_NAME}${NC}"