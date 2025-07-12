#!/bin/bash

echo "🧪 Testing Docs Generation Hook Functionality"
echo "=============================================="

# Test if we can create a minimal TaskRun that just tests the hook
echo "This script tests the docs generation hook without running Claude to save tokens."
echo ""

# Set up test parameters
SERVICE_NAME="docs-generator"
TASK_ID="999999"
ATTEMPT="1"
TARGET_BRANCH="hook-test-$(date +%Y%m%d-%H%M%S)"
SOURCE_BRANCH="feature/example-project-and-cli"
WORKING_DIR="example"
GITHUB_USER="pm0-5dlabs"

echo "Test parameters:"
echo "  SERVICE_NAME: $SERVICE_NAME"
echo "  TARGET_BRANCH: $TARGET_BRANCH"
echo "  SOURCE_BRANCH: $SOURCE_BRANCH"
echo "  WORKING_DIR: $WORKING_DIR"
echo "  GITHUB_USER: $GITHUB_USER"
echo ""

# Check if we can submit a test job
echo "Submitting hook test job..."

# Use orchestrator CLI to submit a test docs job with minimal token usage
orchestrator submit-job \
  --service="$SERVICE_NAME" \
  --task-id="$TASK_ID" \
  --version="1" \
  --model="claude-sonnet-3-5-20241022" \
  --repository="https://github.com/5dlabs/agent-platform.git" \
  --github-user="$GITHUB_USER" \
  --source-branch="$SOURCE_BRANCH" \
  --target-branch="$TARGET_BRANCH" \
  --working-directory="$WORKING_DIR" \
  --env="TEST_MODE=true" \
  --env="HOOK_TEST_ONLY=true"

if [ $? -eq 0 ]; then
  echo "✅ Test job submitted successfully!"
  echo ""
  echo "To monitor the test job:"
  echo "  kubectl -n orchestrator get taskrun -l task-id=$TASK_ID"
  echo "  kubectl -n orchestrator logs -f job/claude-sonnet-docs-generator-task${TASK_ID}-attempt1"
  echo ""
  echo "The test will:"
  echo "  1. Set up the repository and environment"
  echo "  2. Create dummy documentation files"
  echo "  3. Test the hook script (.stop-hook-docs-pr.sh)"
  echo "  4. Verify PR creation works"
  echo "  5. Skip the expensive Claude docs generation"
else
  echo "❌ Failed to submit test job"
  exit 1
fi